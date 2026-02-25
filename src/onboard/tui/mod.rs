mod events;
mod finalize;
mod flow;
mod render;
mod state;

use crate::config::Config;
use anyhow::Result;
use ratatui::crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use self::events::run_app_loop;
use self::finalize::finalize_config;
use self::state::App;

pub async fn run_wizard(force: bool) -> Result<Config> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(force);

    let (default_config, default_workspace) =
        crate::config::schema::resolve_runtime_dirs_for_onboarding().await?;
    app.config_dir = default_config;
    app.config_path = app.config_dir.join("config.toml");
    app.workspace_dir = default_workspace;

    let loop_result = run_app_loop(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    loop_result?;

    finalize_config(&app).await
}
