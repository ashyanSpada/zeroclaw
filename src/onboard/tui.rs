use crate::config::Config;
use crate::onboard::wizard;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::{io, path::PathBuf, time::Duration};
use tui_textarea::{Input, TextArea};

// ── App State ────────────────────────────────────────────────────────

#[derive(PartialEq)]
enum WizardStep {
    Welcome,
    WorkspaceSetup,
    ProviderTierSelection,
    ProviderSelection,
    ApiKeyEntry,
    ModelSelection,
    Confirmation,
    Done,
}

struct App<'a> {
    step: WizardStep,
    /// Whether we are waiting for an async task (like fetching models)
    loading: bool,
    /// Status message to show during loading or as feedback
    status_message: String,

    // Data collected
    config_dir: PathBuf,
    workspace_dir: PathBuf,
    provider: String,
    api_key: String,
    api_url: Option<String>,
    model: String,

    // UI States
    workspace_input: TextArea<'a>,
    use_default_workspace: bool,

    provider_tier_list: ListState,
    provider_list: ListState,
    model_list: ListState,

    api_key_input: TextArea<'a>,

    // Data for lists
    provider_tiers: Vec<&'static str>,
    current_tier_providers: Vec<(&'static str, &'static str)>,
    available_models: Vec<String>,
}

impl<'a> App<'a> {
    fn new() -> Self {
        let mut workspace_input = TextArea::default();
        workspace_input.set_cursor_line_style(RatatuiStyle::default());
        workspace_input.set_placeholder_text("Enter custom workspace path...");

        let mut api_key_input = TextArea::default();
        api_key_input.set_cursor_line_style(RatatuiStyle::default());
        api_key_input.set_placeholder_text("sk-...");
        api_key_input.set_mask_char('•');

        Self {
            step: WizardStep::Welcome,
            loading: false,
            status_message: String::new(),
            config_dir: PathBuf::new(),
            workspace_dir: PathBuf::new(),
            provider: String::new(),
            api_key: String::new(),
            api_url: None,
            model: String::new(),
            workspace_input,
            use_default_workspace: true,
            provider_tier_list: ListState::default(),
            provider_list: ListState::default(),
            model_list: ListState::default(),
            api_key_input,
            provider_tiers: wizard::get_provider_tiers(),
            current_tier_providers: Vec::new(),
            available_models: Vec::new(),
        }
    }

    fn next_step(&mut self) {
        match self.step {
            WizardStep::Welcome => self.step = WizardStep::WorkspaceSetup,
            WizardStep::WorkspaceSetup => self.step = WizardStep::ProviderTierSelection,
            WizardStep::ProviderTierSelection => {
                let tier_idx = self.provider_tier_list.selected().unwrap_or(0);
                self.current_tier_providers = wizard::get_providers_for_tier(tier_idx);
                if self.current_tier_providers.is_empty() {
                    // Custom provider flow - not fully implemented in TUI yet
                    self.provider = "custom".to_string();
                    self.step = WizardStep::ApiKeyEntry;
                } else {
                    self.step = WizardStep::ProviderSelection;
                }
            }
            WizardStep::ProviderSelection => {
                let idx = self.provider_list.selected().unwrap_or(0);
                if let Some((id, _)) = self.current_tier_providers.get(idx) {
                    self.provider = id.to_string();
                    self.step = WizardStep::ApiKeyEntry;
                }
            }
            WizardStep::ApiKeyEntry => {
                self.api_key = self.api_key_input.lines()[0].trim().to_string();
                // Trigger model fetch
                self.loading = true;
                self.status_message = format!("Fetching models for {}...", self.provider);
                // In a real async event loop we would dispatch this.
                // For now, we'll do it synchronously in the update loop (blocking UI briefly)
                // or transition to a loading state that polls.
                self.step = WizardStep::ModelSelection;
            }
            WizardStep::ModelSelection => {
                let idx = self.model_list.selected().unwrap_or(0);
                if let Some(model) = self.available_models.get(idx) {
                    self.model = model.clone();
                    self.step = WizardStep::Confirmation;
                }
            }
            WizardStep::Confirmation => self.step = WizardStep::Done,
            WizardStep::Done => {}
        }
    }
}

pub async fn run_wizard(force: bool) -> Result<Config> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let mut app = App::new();

    // Pre-calculate default workspace
    let (default_config, default_workspace) =
        crate::config::schema::resolve_runtime_dirs_for_onboarding().await?;
    app.config_dir = default_config;
    app.workspace_dir = default_workspace;

    let res = run_app_loop(&mut terminal, &mut app, force).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        return Err(err);
    }

    // Build final config (mockup for now, reusing quick setup logic if possible or manual build)
    // For now, falling back to quick setup with the collected values
    wizard::run_quick_setup(
        Some(&app.api_key),
        Some(&app.provider),
        Some(&app.model),
        None, // Default memory
        force,
    )
    .await
}

async fn run_app_loop<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App<'_>,
    _force: bool,
) -> Result<()>
where
    B::Error: Send + Sync + 'static,
{
    loop {
        // Handle async tasks (model fetching)
        if app.step == WizardStep::ModelSelection && app.available_models.is_empty() && !app.loading
        {
            // Initial load for models step
            // Fetch models synchronously for now to keep it simple, blocking UI briefly
            // In production code, use a channel to send results back
            let models = wizard::fetch_live_models_for_provider(&app.provider, &app.api_key, None);
            match models {
                Ok(m) => {
                    app.available_models = m;
                    if app.available_models.is_empty() {
                        app.available_models
                            .push(wizard::default_model_for_provider(&app.provider));
                    }
                }
                Err(e) => {
                    app.status_message = format!("Failed to fetch models: {}", e);
                    app.available_models
                        .push(wizard::default_model_for_provider(&app.provider));
                }
            }
            app.loading = false;
        }

        terminal.draw(|f| ui(f, app))?;

        if crossterm::event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.code == KeyCode::Esc {
                        return Ok(());
                    }

                    match app.step {
                        WizardStep::Welcome => {
                            if key.code == KeyCode::Enter {
                                app.next_step();
                            }
                        }
                        WizardStep::WorkspaceSetup => {
                            if key.code == KeyCode::Enter {
                                if !app.use_default_workspace {
                                    let input = app.workspace_input.lines()[0].trim();
                                    if !input.is_empty() {
                                        let expanded = shellexpand::tilde(input).to_string();
                                        let (_, ws) =
                                            crate::config::schema::resolve_config_dir_for_workspace(
                                                &PathBuf::from(expanded),
                                            );
                                        app.workspace_dir = ws;
                                    }
                                }
                                app.next_step();
                            } else if key.code == KeyCode::Tab {
                                app.use_default_workspace = !app.use_default_workspace;
                            } else if !app.use_default_workspace {
                                app.workspace_input.input(Input::from(key));
                            }
                        }
                        WizardStep::ProviderTierSelection => {
                            if key.code == KeyCode::Enter {
                                app.next_step();
                            } else if key.code == KeyCode::Down {
                                let i = app.provider_tier_list.selected().unwrap_or(0);
                                if i < app.provider_tiers.len() - 1 {
                                    app.provider_tier_list.select(Some(i + 1));
                                }
                            } else if key.code == KeyCode::Up {
                                let i = app.provider_tier_list.selected().unwrap_or(0);
                                if i > 0 {
                                    app.provider_tier_list.select(Some(i - 1));
                                }
                            }
                        }
                        WizardStep::ProviderSelection => {
                            if key.code == KeyCode::Enter {
                                app.next_step();
                            } else if key.code == KeyCode::Down {
                                let i = app.provider_list.selected().unwrap_or(0);
                                if i < app.current_tier_providers.len() - 1 {
                                    app.provider_list.select(Some(i + 1));
                                }
                            } else if key.code == KeyCode::Up {
                                let i = app.provider_list.selected().unwrap_or(0);
                                if i > 0 {
                                    app.provider_list.select(Some(i - 1));
                                }
                            }
                        }
                        WizardStep::ApiKeyEntry => {
                            if key.code == KeyCode::Enter {
                                app.next_step();
                            } else {
                                app.api_key_input.input(Input::from(key));
                            }
                        }
                        WizardStep::ModelSelection => {
                            if key.code == KeyCode::Enter {
                                app.next_step();
                            } else if key.code == KeyCode::Down {
                                let i = app.model_list.selected().unwrap_or(0);
                                if i < app.available_models.len() - 1 {
                                    app.model_list.select(Some(i + 1));
                                }
                            } else if key.code == KeyCode::Up {
                                let i = app.model_list.selected().unwrap_or(0);
                                if i > 0 {
                                    app.model_list.select(Some(i - 1));
                                }
                            }
                        }
                        WizardStep::Confirmation => {
                            if key.code == KeyCode::Enter {
                                return Ok(()); // Done
                            }
                        }
                        WizardStep::Done => return Ok(()),
                    }
                }
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3), // Title
                Constraint::Min(10),   // Content
                Constraint::Length(3), // Footer
            ]
            .as_ref(),
        )
        .split(f.area());

    let title = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan))
        .title(" ZeroClaw Setup Wizard ");
    f.render_widget(title, chunks[0]);

    // Footer
    let footer_text = match app.step {
        WizardStep::Welcome => "Press <Enter> to start • <Esc> to quit",
        WizardStep::WorkspaceSetup => "Press <Tab> to toggle custom path • <Enter> to confirm",
        WizardStep::ProviderTierSelection
        | WizardStep::ProviderSelection
        | WizardStep::ModelSelection => "Use <Up/Down> to select • <Enter> to confirm",
        WizardStep::ApiKeyEntry => "Paste/Type key • <Enter> to confirm",
        WizardStep::Confirmation => "Press <Enter> to finish setup",
        WizardStep::Done => "Setup Complete",
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);

    // Content area
    let inner_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(chunks[1]);

    match app.step {
        WizardStep::Welcome => draw_welcome(f, inner_chunks[0]),
        WizardStep::WorkspaceSetup => draw_workspace_setup(f, app, inner_chunks[0]),
        WizardStep::ProviderTierSelection => draw_provider_tier(f, app, inner_chunks[0]),
        WizardStep::ProviderSelection => draw_provider_select(f, app, inner_chunks[0]),
        WizardStep::ApiKeyEntry => draw_api_key(f, app, inner_chunks[0]),
        WizardStep::ModelSelection => draw_model_select(f, app, inner_chunks[0]),
        WizardStep::Confirmation => draw_confirmation(f, app, inner_chunks[0]),
        WizardStep::Done => {}
    }
}

fn draw_welcome(f: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(Span::styled(
            "Welcome to ZeroClaw",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Green),
        )),
        Line::from(""),
        Line::from("This wizard will help you configure your AI assistant."),
        Line::from(""),
        Line::from("Steps:"),
        Line::from(" 1. Workspace Setup"),
        Line::from(" 2. Provider Selection"),
        Line::from(" 3. API Key & Model"),
        Line::from(""),
        Line::from("Let's get started!"),
    ];
    let p = Paragraph::new(text)
        .block(Block::default().borders(Borders::NONE))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(p, area);
}

fn draw_workspace_setup(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(area);

    let default_style = if app.use_default_workspace {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let custom_style = if !app.use_default_workspace {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    f.render_widget(
        Paragraph::new(format!("Default: {}", app.workspace_dir.display())).block(
            Block::default()
                .title(" Option 1: Default ")
                .borders(Borders::ALL)
                .border_style(default_style),
        ),
        chunks[0],
    );

    app.workspace_input.set_block(
        Block::default()
            .title(" Option 2: Custom Path ")
            .borders(Borders::ALL)
            .border_style(custom_style),
    );
    f.render_widget(&app.workspace_input, chunks[1]);
}

fn draw_provider_tier(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .provider_tiers
        .iter()
        .map(|t| ListItem::new(Line::from(*t)))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Select Provider Category ")
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    if app.provider_tier_list.selected().is_none() {
        app.provider_tier_list.select(Some(0));
    }
    f.render_stateful_widget(list, area, &mut app.provider_tier_list);
}

fn draw_provider_select(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .current_tier_providers
        .iter()
        .map(|(id, desc)| {
            ListItem::new(vec![
                Line::from(Span::styled(
                    *id,
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    format!("  {}", desc),
                    Style::default().fg(Color::DarkGray),
                )),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Select Provider ")
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    if app.provider_list.selected().is_none() {
        app.provider_list.select(Some(0));
    }
    f.render_stateful_widget(list, area, &mut app.provider_list);
}

fn draw_api_key(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(area);

    app.api_key_input.set_block(
        Block::default()
            .title(" Enter API Key ")
            .borders(Borders::ALL),
    );
    f.render_widget(&app.api_key_input, chunks[0]);

    let help = Paragraph::new("Your API key will be stored securely.")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[1]);
}

fn draw_model_select(f: &mut Frame, app: &mut App, area: Rect) {
    if app.available_models.is_empty() {
        let p = Paragraph::new("Loading models...").alignment(ratatui::layout::Alignment::Center);
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = app
        .available_models
        .iter()
        .map(|m| ListItem::new(Line::from(m.as_str())))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(" Select Model ({}) ", app.provider))
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    if app.model_list.selected().is_none() {
        app.model_list.select(Some(0));
    }
    f.render_stateful_widget(list, area, &mut app.model_list);
}

fn draw_confirmation(f: &mut Frame, app: &mut App, area: Rect) {
    let text = vec![
        Line::from(vec![Span::styled(
            "Configuration Summary",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Workspace: "),
            Span::styled(
                app.workspace_dir.display().to_string(),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw("Provider:  "),
            Span::styled(&app.provider, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("Model:     "),
            Span::styled(&app.model, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from("Ready to initialize!"),
    ];

    let p = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
    f.render_widget(p, area);
}
