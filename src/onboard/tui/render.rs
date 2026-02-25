use crate::memory::selectable_memory_backends;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::state::{App, ChannelChoice, OnboardingMode, WizardStep, CUSTOM_MODEL_SENTINEL};

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.area());

    let title = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan))
        .title(" ZeroClaw Setup Wizard (ratatui) ");
    f.render_widget(title, chunks[0]);

    let footer_text = match app.step {
        WizardStep::Welcome => "Press <Enter> to start • <Esc> to quit",
        WizardStep::ConfigModeSelection
        | WizardStep::ProviderTierSelection
        | WizardStep::ProviderSelection
        | WizardStep::ModelSelection
        | WizardStep::ChannelSelection
        | WizardStep::TunnelSelection
        | WizardStep::ToolModeSelection
        | WizardStep::HardwareSelection
        | WizardStep::MemorySelection
        | WizardStep::ProjectStyleSelection => "Use <Up/Down> to select • <Enter> to confirm",
        WizardStep::WorkspaceSetup => "Press <Tab> to toggle custom path • <Enter> to confirm",
        WizardStep::SecretsEncryptChoice => "Press <Tab> to toggle • <Enter> to continue",
        WizardStep::TunnelPrimaryEntry => "Type value • <Tab> toggle Funnel • <Enter> continue",
        WizardStep::CustomProviderUrlEntry
        | WizardStep::ProviderEndpointEntry
        | WizardStep::ApiKeyEntry
        | WizardStep::ModelCustomEntry
        | WizardStep::ChannelTokenEntry
        | WizardStep::ChannelAuxEntry
        | WizardStep::TunnelSecondaryEntry
        | WizardStep::ComposioApiKeyEntry
        | WizardStep::ProjectUserEntry
        | WizardStep::ProjectTimezoneEntry
        | WizardStep::ProjectAgentEntry
        | WizardStep::ProjectStyleCustomEntry => "Type value • <Enter> to confirm",
        WizardStep::Confirmation => "Press <Enter> to save and finish",
        WizardStep::Done => "Setup Complete",
    };

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);

    match app.step {
        WizardStep::Welcome => draw_welcome(f, chunks[1]),
        WizardStep::ConfigModeSelection => draw_mode_selection(f, app, chunks[1]),
        WizardStep::WorkspaceSetup => draw_workspace_setup(f, app, chunks[1]),
        WizardStep::ProviderTierSelection => draw_provider_tier(f, app, chunks[1]),
        WizardStep::ProviderSelection => draw_provider_select(f, app, chunks[1]),
        WizardStep::CustomProviderUrlEntry => draw_custom_provider_url(f, app, chunks[1]),
        WizardStep::ProviderEndpointEntry => draw_provider_endpoint(f, app, chunks[1]),
        WizardStep::ApiKeyEntry => draw_api_key(f, app, chunks[1]),
        WizardStep::ModelSelection => draw_model_select(f, app, chunks[1]),
        WizardStep::ModelCustomEntry => draw_custom_model(f, app, chunks[1]),
        WizardStep::ChannelSelection => draw_channel_select(f, app, chunks[1]),
        WizardStep::ChannelTokenEntry => draw_channel_token(f, app, chunks[1]),
        WizardStep::ChannelAuxEntry => draw_channel_aux(f, app, chunks[1]),
        WizardStep::TunnelSelection => draw_tunnel_select(f, app, chunks[1]),
        WizardStep::TunnelPrimaryEntry => draw_tunnel_primary(f, app, chunks[1]),
        WizardStep::TunnelSecondaryEntry => draw_tunnel_secondary(f, app, chunks[1]),
        WizardStep::ToolModeSelection => draw_tool_mode(f, app, chunks[1]),
        WizardStep::ComposioApiKeyEntry => draw_composio_key(f, app, chunks[1]),
        WizardStep::SecretsEncryptChoice => draw_secrets_encrypt(f, app, chunks[1]),
        WizardStep::HardwareSelection => draw_hardware_select(f, app, chunks[1]),
        WizardStep::MemorySelection => draw_memory_select(f, app, chunks[1]),
        WizardStep::ProjectUserEntry => draw_project_user(f, app, chunks[1]),
        WizardStep::ProjectTimezoneEntry => draw_project_timezone(f, app, chunks[1]),
        WizardStep::ProjectAgentEntry => draw_project_agent(f, app, chunks[1]),
        WizardStep::ProjectStyleSelection => draw_project_style_select(f, app, chunks[1]),
        WizardStep::ProjectStyleCustomEntry => draw_project_style_custom(f, app, chunks[1]),
        WizardStep::Confirmation => draw_confirmation(f, app, chunks[1]),
        WizardStep::Done => {}
    }
}

fn draw_welcome(f: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(Span::styled(
            "Welcome to ZeroClaw",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Green),
        )),
        Line::from(""),
        Line::from("This wizard covers provider, channels, tunnel, tools, hardware, memory, and context."),
    ];
    f.render_widget(Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center), area);
}

fn draw_mode_selection(f: &mut Frame, app: &mut App, area: Rect) {
    let items = vec![
        ListItem::new("Full onboarding (overwrite config with current wizard choices)"),
        ListItem::new("Update provider/model/api key only (preserve other settings)"),
    ];
    draw_list(f, area, " Existing config detected — choose mode ", items, &mut app.mode_list);
}

fn draw_workspace_setup(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(area);

    let default_style = if app.use_default_workspace {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let custom_style = if !app.use_default_workspace {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
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
    let items = app
        .provider_tiers
        .iter()
        .map(|t| ListItem::new(Line::from(*t)))
        .collect();
    draw_list(
        f,
        area,
        " Select Provider Category ",
        items,
        &mut app.provider_tier_list,
    );
}

fn draw_provider_select(f: &mut Frame, app: &mut App, area: Rect) {
    let items = app
        .current_tier_providers
        .iter()
        .map(|(id, desc)| {
            ListItem::new(vec![
                Line::from(Span::styled(*id, Style::default().add_modifier(Modifier::BOLD))),
                Line::from(Span::styled(
                    format!("  {}", desc),
                    Style::default().fg(Color::DarkGray),
                )),
            ])
        })
        .collect();
    draw_list(f, area, " Select Provider ", items, &mut app.provider_list);
}

fn draw_custom_provider_url(f: &mut Frame, app: &mut App, area: Rect) {
    app.custom_provider_url_input.set_block(
        Block::default()
            .title(" Enter custom OpenAI-compatible base URL ")
            .borders(Borders::ALL),
    );
    f.render_widget(&app.custom_provider_url_input, area);
}

fn draw_provider_endpoint(f: &mut Frame, app: &mut App, area: Rect) {
    app.provider_endpoint_input.set_block(
        Block::default()
            .title(format!(" Enter {} endpoint URL ", app.provider))
            .borders(Borders::ALL),
    );
    f.render_widget(&app.provider_endpoint_input, area);
}

fn draw_api_key(f: &mut Frame, app: &mut App, area: Rect) {
    app.api_key_input
        .set_block(Block::default().title(" Enter API Key ").borders(Borders::ALL));
    f.render_widget(&app.api_key_input, area);
}

fn draw_model_select(f: &mut Frame, app: &mut App, area: Rect) {
    if app.loading {
        f.render_widget(
            Paragraph::new("Loading models...").alignment(ratatui::layout::Alignment::Center),
            area,
        );
        return;
    }
    let items = app
        .available_models
        .iter()
        .map(|m| {
            if m == CUSTOM_MODEL_SENTINEL {
                ListItem::new("Custom model ID (type manually)")
            } else {
                ListItem::new(m.as_str())
            }
        })
        .collect();
    draw_list(
        f,
        area,
        &format!(" Select Model ({}) ", app.provider),
        items,
        &mut app.model_list,
    );
}

fn draw_custom_model(f: &mut Frame, app: &mut App, area: Rect) {
    app.model_custom_input.set_block(
        Block::default()
            .title(" Enter custom model ID ")
            .borders(Borders::ALL),
    );
    f.render_widget(&app.model_custom_input, area);
}

fn draw_channel_select(f: &mut Frame, app: &mut App, area: Rect) {
    let items = vec![
        ListItem::new("CLI only (default)"),
        ListItem::new("Telegram"),
        ListItem::new("Discord"),
        ListItem::new("Slack"),
        ListItem::new("iMessage"),
        ListItem::new("Matrix"),
        ListItem::new("Signal"),
        ListItem::new("WhatsApp"),
        ListItem::new("Linq"),
        ListItem::new("IRC"),
        ListItem::new("Webhook"),
        ListItem::new("Nextcloud Talk"),
        ListItem::new("DingTalk"),
        ListItem::new("QQ Official"),
        ListItem::new("Lark"),
        ListItem::new("Feishu"),
        ListItem::new("Nostr"),
    ];
    draw_list(f, area, " Select primary channel ", items, &mut app.channel_list);
}

fn draw_channel_token(f: &mut Frame, app: &mut App, area: Rect) {
    let title = match app.channel_choice {
        ChannelChoice::Telegram => " Telegram bot token ",
        ChannelChoice::Discord => " Discord bot token ",
        ChannelChoice::Slack => " Slack bot token ",
        ChannelChoice::Matrix => " Matrix access token ",
        ChannelChoice::Signal => " Signal account (phone) ",
        ChannelChoice::WhatsApp => " WhatsApp access token (optional) ",
        ChannelChoice::Linq => " Linq API token ",
        ChannelChoice::Irc => " IRC server (default irc.libera.chat) ",
        ChannelChoice::Webhook => " Webhook listen port (default 8081) ",
        ChannelChoice::NextcloudTalk => " Nextcloud base URL ",
        ChannelChoice::DingTalk => " DingTalk client ID ",
        ChannelChoice::QqOfficial => " QQ app ID ",
        ChannelChoice::Lark => " Lark app ID ",
        ChannelChoice::Feishu => " Feishu app ID ",
        ChannelChoice::Nostr => " Nostr private key ",
        _ => " Channel primary input ",
    };

    app.channel_token_input.set_block(
        Block::default()
            .title(title)
            .borders(Borders::ALL),
    );
    f.render_widget(&app.channel_token_input, area);
}

fn draw_channel_aux(f: &mut Frame, app: &mut App, area: Rect) {
    match app.channel_choice {
        ChannelChoice::IMessage => {
            app.channel_token_input.set_block(
                Block::default()
                    .title(" iMessage allowed contacts (comma-separated) ")
                    .borders(Borders::ALL),
            );
            f.render_widget(&app.channel_token_input, area);
        }
        _ => {
            let title = match app.channel_choice {
                ChannelChoice::Telegram
                | ChannelChoice::Discord
                | ChannelChoice::Slack
                | ChannelChoice::Nostr => " Allowed users/pubkeys (comma-separated) ",
                ChannelChoice::Webhook => " Webhook secret (optional) ",
                ChannelChoice::Matrix => " Matrix homeserver URL (optional) ",
                ChannelChoice::Signal => " Signal group ID (optional) ",
                ChannelChoice::WhatsApp => " WhatsApp phone_number_id (optional) ",
                ChannelChoice::Linq => " Linq from_phone (optional) ",
                ChannelChoice::Irc => " IRC nickname (optional) ",
                ChannelChoice::NextcloudTalk => " Nextcloud app token ",
                ChannelChoice::DingTalk => " DingTalk client secret ",
                ChannelChoice::QqOfficial => " QQ app secret ",
                ChannelChoice::Lark => " Lark app secret ",
                ChannelChoice::Feishu => " Feishu app secret ",
                _ => " Channel secondary input ",
            };

            app.channel_aux_input
                .set_block(Block::default().title(title).borders(Borders::ALL));
            f.render_widget(&app.channel_aux_input, area);
        }
    }
}

fn draw_tunnel_select(f: &mut Frame, app: &mut App, area: Rect) {
    let items = vec![
        ListItem::new("None (local only)"),
        ListItem::new("Cloudflare Tunnel"),
        ListItem::new("Tailscale"),
        ListItem::new("ngrok"),
        ListItem::new("Custom"),
    ];
    draw_list(f, area, " Select tunnel provider ", items, &mut app.tunnel_list);
}

fn draw_tunnel_primary(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(2)].as_ref())
        .split(area);

    app.tunnel_primary_input.set_block(
        Block::default()
            .title(" Tunnel primary field (token/command) ")
            .borders(Borders::ALL),
    );
    f.render_widget(&app.tunnel_primary_input, chunks[0]);

    f.render_widget(
        Paragraph::new(format!(
            "Tailscale Funnel: {} (Tab to toggle)",
            if app.tunnel_toggle { "on" } else { "off" }
        ))
        .style(Style::default().fg(Color::DarkGray)),
        chunks[1],
    );
}

fn draw_tunnel_secondary(f: &mut Frame, app: &mut App, area: Rect) {
    app.tunnel_secondary_input.set_block(
        Block::default()
            .title(" Tunnel secondary field (domain/hostname/health_url optional) ")
            .borders(Borders::ALL),
    );
    f.render_widget(&app.tunnel_secondary_input, area);
}

fn draw_tool_mode(f: &mut Frame, app: &mut App, area: Rect) {
    let items = vec![
        ListItem::new("Sovereign (local only)"),
        ListItem::new("Composio (managed OAuth)")
    ];
    draw_list(f, area, " Select tool mode ", items, &mut app.tool_mode_list);
}

fn draw_composio_key(f: &mut Frame, app: &mut App, area: Rect) {
    app.composio_key_input.set_block(
        Block::default()
            .title(" Composio API key (optional) ")
            .borders(Borders::ALL),
    );
    f.render_widget(&app.composio_key_input, area);
}

fn draw_secrets_encrypt(f: &mut Frame, app: &mut App, area: Rect) {
    f.render_widget(
        Paragraph::new(format!(
            "Encrypted secret storage: {}\n\nPress Tab to toggle.",
            if app.secrets_encrypt { "enabled" } else { "disabled" }
        ))
        .block(Block::default().title(" Secrets ").borders(Borders::ALL)),
        area,
    );
}

fn draw_hardware_select(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Length(2)].as_ref())
        .split(area);

    let items = vec![
        ListItem::new("Native GPIO"),
        ListItem::new("Serial tethered board"),
        ListItem::new("Probe (SWD/JTAG)"),
        ListItem::new("Software only"),
    ];
    draw_list(
        f,
        chunks[0],
        " Hardware mode ",
        items,
        &mut app.hardware_list,
    );

    f.render_widget(
        Paragraph::new(format!(
            "Workspace datasheets: {} (Tab to toggle)",
            if app.hardware_datasheets { "on" } else { "off" }
        ))
        .style(Style::default().fg(Color::DarkGray)),
        chunks[1],
    );
}

fn draw_memory_select(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Length(2)].as_ref())
        .split(area);

    let items = selectable_memory_backends()
        .iter()
        .map(|backend| ListItem::new(backend.label))
        .collect();
    draw_list(
        f,
        chunks[0],
        " Memory backend ",
        items,
        &mut app.memory_list,
    );

    f.render_widget(
        Paragraph::new(format!(
            "Auto-save: {} (Tab to toggle)",
            if app.memory_auto_save { "on" } else { "off" }
        ))
        .style(Style::default().fg(Color::DarkGray)),
        chunks[1],
    );
}

fn draw_project_user(f: &mut Frame, app: &mut App, area: Rect) {
    app.project_user_input.set_block(
        Block::default()
            .title(" Project Context: user name ")
            .borders(Borders::ALL),
    );
    f.render_widget(&app.project_user_input, area);
}

fn draw_project_timezone(f: &mut Frame, app: &mut App, area: Rect) {
    app.project_timezone_input.set_block(
        Block::default()
            .title(" Project Context: timezone ")
            .borders(Borders::ALL),
    );
    f.render_widget(&app.project_timezone_input, area);
}

fn draw_project_agent(f: &mut Frame, app: &mut App, area: Rect) {
    app.project_agent_input.set_block(
        Block::default()
            .title(" Project Context: agent name ")
            .borders(Borders::ALL),
    );
    f.render_widget(&app.project_agent_input, area);
}

fn draw_project_style_select(f: &mut Frame, app: &mut App, area: Rect) {
    let items = vec![
        ListItem::new("Direct & concise"),
        ListItem::new("Friendly & casual"),
        ListItem::new("Professional & polished"),
        ListItem::new("Expressive & playful"),
        ListItem::new("Technical & detailed"),
        ListItem::new("Balanced"),
        ListItem::new("Custom"),
    ];
    draw_list(
        f,
        area,
        " Communication style ",
        items,
        &mut app.project_style_list,
    );
}

fn draw_project_style_custom(f: &mut Frame, app: &mut App, area: Rect) {
    app.project_style_custom_input.set_block(
        Block::default()
            .title(" Custom communication style ")
            .borders(Borders::ALL),
    );
    f.render_widget(&app.project_style_custom_input, area);
}

fn draw_confirmation(f: &mut Frame, app: &mut App, area: Rect) {
    let mode = match app.mode {
        OnboardingMode::FullOnboarding => "Full onboarding",
        OnboardingMode::UpdateProviderOnly => "Update provider only",
    };

    let lines = vec![
        Line::from(Span::styled(
            "Configuration Summary",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Mode:      {mode}")),
        Line::from(format!("Workspace: {}", app.workspace_dir.display())),
        Line::from(format!("Provider:  {}", app.provider)),
        Line::from(format!("Model:     {}", app.model)),
        Line::from(format!("Status:    {}", app.status_message)),
    ];

    f.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL)),
        area,
    );
}

fn draw_list(
    f: &mut Frame,
    area: Rect,
    title: &str,
    items: Vec<ListItem<'_>>,
    state: &mut ratatui::widgets::ListState,
) {
    let list = List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    if state.selected().is_none() {
        state.select(Some(0));
    }

    f.render_stateful_widget(list, area, state);
}
