use anyhow::Result;
use ratatui::{
    backend::Backend,
    crossterm::event::{self, Event, KeyCode},
    Terminal,
};
use std::{path::PathBuf, time::Duration};
use tui_textarea::Input;

use super::render::ui;
use super::state::{App, ChannelChoice, WizardStep};

pub async fn run_app_loop<B: Backend>(terminal: &mut Terminal<B>, app: &mut App<'_>) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Esc {
                    return Ok(());
                }

                match app.step {
                    WizardStep::Welcome => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        }
                    }
                    WizardStep::ConfigModeSelection => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else if key.code == KeyCode::Down {
                            let index = app.mode_list.selected().unwrap_or(1);
                            if index < 1 {
                                app.mode_list.select(Some(index + 1));
                            }
                        } else if key.code == KeyCode::Up {
                            let index = app.mode_list.selected().unwrap_or(1);
                            if index > 0 {
                                app.mode_list.select(Some(index - 1));
                            }
                        }
                    }
                    WizardStep::WorkspaceSetup => {
                        if key.code == KeyCode::Enter {
                            if !app.use_default_workspace {
                                let input = App::text_value(&app.workspace_input);
                                if !input.is_empty() {
                                    let expanded = shellexpand::tilde(&input).to_string();
                                    let (config_dir, workspace_dir) =
                                        crate::config::schema::resolve_config_dir_for_workspace(
                                            &PathBuf::from(expanded),
                                        );
                                    app.config_dir = config_dir;
                                    app.workspace_dir = workspace_dir;
                                    app.config_path = app.config_dir.join("config.toml");
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
                            let index = app.provider_tier_list.selected().unwrap_or(0);
                            if index < app.provider_tiers.len().saturating_sub(1) {
                                app.provider_tier_list.select(Some(index + 1));
                            }
                        } else if key.code == KeyCode::Up {
                            let index = app.provider_tier_list.selected().unwrap_or(0);
                            if index > 0 {
                                app.provider_tier_list.select(Some(index - 1));
                            }
                        }
                    }
                    WizardStep::ProviderSelection => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else if key.code == KeyCode::Down {
                            let index = app.provider_list.selected().unwrap_or(0);
                            if index < app.current_tier_providers.len().saturating_sub(1) {
                                app.provider_list.select(Some(index + 1));
                            }
                        } else if key.code == KeyCode::Up {
                            let index = app.provider_list.selected().unwrap_or(0);
                            if index > 0 {
                                app.provider_list.select(Some(index - 1));
                            }
                        }
                    }
                    WizardStep::CustomProviderUrlEntry => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else {
                            app.custom_provider_url_input.input(Input::from(key));
                        }
                    }
                    WizardStep::ProviderEndpointEntry => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else {
                            app.provider_endpoint_input.input(Input::from(key));
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
                            let index = app.model_list.selected().unwrap_or(0);
                            if index < app.available_models.len().saturating_sub(1) {
                                app.model_list.select(Some(index + 1));
                            }
                        } else if key.code == KeyCode::Up {
                            let index = app.model_list.selected().unwrap_or(0);
                            if index > 0 {
                                app.model_list.select(Some(index - 1));
                            }
                        }
                    }
                    WizardStep::ModelCustomEntry => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else {
                            app.model_custom_input.input(Input::from(key));
                        }
                    }
                    WizardStep::ChannelSelection => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else if key.code == KeyCode::Down {
                            let index = app.channel_list.selected().unwrap_or(0);
                            if index < 16 {
                                app.channel_list.select(Some(index + 1));
                            }
                        } else if key.code == KeyCode::Up {
                            let index = app.channel_list.selected().unwrap_or(0);
                            if index > 0 {
                                app.channel_list.select(Some(index - 1));
                            }
                        }
                    }
                    WizardStep::ChannelTokenEntry => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else {
                            app.channel_token_input.input(Input::from(key));
                        }
                    }
                    WizardStep::ChannelAuxEntry => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else {
                            if app.channel_choice == ChannelChoice::IMessage {
                                app.channel_token_input.input(Input::from(key));
                            } else {
                                app.channel_aux_input.input(Input::from(key));
                            }
                        }
                    }
                    WizardStep::TunnelSelection => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else if key.code == KeyCode::Down {
                            let index = app.tunnel_list.selected().unwrap_or(0);
                            if index < 4 {
                                app.tunnel_list.select(Some(index + 1));
                            }
                        } else if key.code == KeyCode::Up {
                            let index = app.tunnel_list.selected().unwrap_or(0);
                            if index > 0 {
                                app.tunnel_list.select(Some(index - 1));
                            }
                        }
                    }
                    WizardStep::TunnelPrimaryEntry => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else if key.code == KeyCode::Tab {
                            app.tunnel_toggle = !app.tunnel_toggle;
                        } else {
                            app.tunnel_primary_input.input(Input::from(key));
                        }
                    }
                    WizardStep::TunnelSecondaryEntry => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else {
                            app.tunnel_secondary_input.input(Input::from(key));
                        }
                    }
                    WizardStep::ToolModeSelection => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else if key.code == KeyCode::Down {
                            let index = app.tool_mode_list.selected().unwrap_or(0);
                            if index < 1 {
                                app.tool_mode_list.select(Some(index + 1));
                            }
                        } else if key.code == KeyCode::Up {
                            let index = app.tool_mode_list.selected().unwrap_or(0);
                            if index > 0 {
                                app.tool_mode_list.select(Some(index - 1));
                            }
                        }
                    }
                    WizardStep::ComposioApiKeyEntry => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else {
                            app.composio_key_input.input(Input::from(key));
                        }
                    }
                    WizardStep::SecretsEncryptChoice => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else if key.code == KeyCode::Tab {
                            app.secrets_encrypt = !app.secrets_encrypt;
                        }
                    }
                    WizardStep::HardwareSelection => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else if key.code == KeyCode::Down {
                            let index = app.hardware_list.selected().unwrap_or(3);
                            if index < 3 {
                                app.hardware_list.select(Some(index + 1));
                            }
                        } else if key.code == KeyCode::Up {
                            let index = app.hardware_list.selected().unwrap_or(3);
                            if index > 0 {
                                app.hardware_list.select(Some(index - 1));
                            }
                        } else if key.code == KeyCode::Tab {
                            app.hardware_datasheets = !app.hardware_datasheets;
                        }
                    }
                    WizardStep::MemorySelection => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else if key.code == KeyCode::Down {
                            let max_index = crate::memory::selectable_memory_backends().len().saturating_sub(1);
                            let index = app.memory_list.selected().unwrap_or(0);
                            if index < max_index {
                                app.memory_list.select(Some(index + 1));
                            }
                        } else if key.code == KeyCode::Up {
                            let index = app.memory_list.selected().unwrap_or(0);
                            if index > 0 {
                                app.memory_list.select(Some(index - 1));
                            }
                        } else if key.code == KeyCode::Tab {
                            app.memory_auto_save = !app.memory_auto_save;
                        }
                    }
                    WizardStep::ProjectUserEntry => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else {
                            app.project_user_input.input(Input::from(key));
                        }
                    }
                    WizardStep::ProjectTimezoneEntry => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else {
                            app.project_timezone_input.input(Input::from(key));
                        }
                    }
                    WizardStep::ProjectAgentEntry => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else {
                            app.project_agent_input.input(Input::from(key));
                        }
                    }
                    WizardStep::ProjectStyleSelection => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else if key.code == KeyCode::Down {
                            let index = app.project_style_list.selected().unwrap_or(1);
                            if index < 6 {
                                app.project_style_list.select(Some(index + 1));
                            }
                        } else if key.code == KeyCode::Up {
                            let index = app.project_style_list.selected().unwrap_or(1);
                            if index > 0 {
                                app.project_style_list.select(Some(index - 1));
                            }
                        }
                    }
                    WizardStep::ProjectStyleCustomEntry => {
                        if key.code == KeyCode::Enter {
                            app.next_step();
                        } else {
                            app.project_style_custom_input.input(Input::from(key));
                        }
                    }
                    WizardStep::Confirmation => {
                        if key.code == KeyCode::Enter {
                            return Ok(());
                        }
                    }
                    WizardStep::Done => return Ok(()),
                }
            }
        }
    }
}
