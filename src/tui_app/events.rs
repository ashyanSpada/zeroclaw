use anyhow::Result;
use ratatui::{
    backend::Backend,
    crossterm::event::{self, Event, KeyCode},
    Terminal,
};
use std::time::Duration;

use crate::config::Config;

use super::{actions, render::ui, state::AppState};

pub async fn run_app_loop<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut AppState,
    config: &Config,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Up | KeyCode::Char('k') => {
                        let current = app.menu.selected().unwrap_or(0);
                        app.menu.select(Some(current.saturating_sub(1)));
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let current = app.menu.selected().unwrap_or(0);
                        let next = (current + 1).min(app.items.len().saturating_sub(1));
                        app.menu.select(Some(next));
                    }
                    KeyCode::Enter => {
                        app.output = actions::run(app.selected_item(), config).await;
                    }
                    _ => {}
                }
            }
        }
    }
}
