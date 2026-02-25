use crate::{
    auth::AuthService, channels, config::Config, doctor, hardware, memory, onboard, peripherals,
    security, providers,
};
use anyhow::Context;

use super::state::MenuItem;

pub async fn run(item: MenuItem, config: &Config) -> Vec<String> {
    match item {
        MenuItem::Home => vec![
            "ZeroClaw TUI Dashboard".to_string(),
            "".to_string(),
            "This dashboard provides read-only command views.".to_string(),
            "Select a menu item and press Enter.".to_string(),
        ],
        MenuItem::Status => status_lines(config),
        MenuItem::Providers => provider_lines(config),
        MenuItem::ConfigSchema => config_schema_lines(),
        MenuItem::EstopStatus => estop_status_lines(config),
        MenuItem::Channels => channel_lines(config),
        MenuItem::ChannelDoctor => channel_doctor_lines(config).await,
        MenuItem::AuthProfiles => auth_profile_lines(config).await,
        MenuItem::ModelsList => models_list_lines(config),
        MenuItem::ModelsStatus => models_status_lines(config),
        MenuItem::ModelsRefresh => models_refresh_lines(config).await,
        MenuItem::DoctorFull => doctor_full_lines(config),
        MenuItem::DoctorModels => doctor_models_lines(config).await,
        MenuItem::Doctor => doctor_lines(config),
        MenuItem::MemoryList => memory_list_lines(config).await,
        MenuItem::MemoryStats => memory_stats_lines(config),
        MenuItem::HardwareDiscover => hardware_discover_lines(config),
        MenuItem::PeripheralList => peripheral_list_lines(config).await,
    }
}

fn config_schema_lines() -> Vec<String> {
    let schema = schemars::schema_for!(crate::config::Config);
    let pretty = serde_json::to_string_pretty(&schema)
        .unwrap_or_else(|error| format!("<schema serialization failed: {error}>"));
    let mut lines = vec![
        "Config Schema".to_string(),
        "".to_string(),
        "Previewing first 120 lines: ".to_string(),
    ];
    lines.extend(pretty.lines().take(120).map(ToString::to_string));
    lines
}

fn estop_status_lines(config: &Config) -> Vec<String> {
    if !config.security.estop.enabled {
        return vec![
            "Estop Status".to_string(),
            "".to_string(),
            "Emergency stop is disabled in config.".to_string(),
        ];
    }

    let result = (|| -> anyhow::Result<Vec<String>> {
        let config_dir = config
            .config_path
            .parent()
            .context("Config path must have a parent directory")?;
        let manager = security::EstopManager::load(&config.security.estop, config_dir)?;
        let state = manager.status();

        let mut lines = vec!["Estop Status".to_string(), "".to_string()];
        lines.push(format!("engaged: {}", if state.is_engaged() { "yes" } else { "no" }));
        lines.push(format!(
            "kill_all: {}",
            if state.kill_all { "active" } else { "inactive" }
        ));
        lines.push(format!(
            "network_kill: {}",
            if state.network_kill { "active" } else { "inactive" }
        ));
        lines.push(format!(
            "domain_blocks: {}",
            if state.blocked_domains.is_empty() {
                "(none)".to_string()
            } else {
                state.blocked_domains.join(", ")
            }
        ));
        lines.push(format!(
            "tool_freeze: {}",
            if state.frozen_tools.is_empty() {
                "(none)".to_string()
            } else {
                state.frozen_tools.join(", ")
            }
        ));
        if let Some(updated_at) = state.updated_at {
            lines.push(format!("updated_at: {updated_at}"));
        }
        Ok(lines)
    })();

    match result {
        Ok(lines) => lines,
        Err(error) => vec![
            "Estop Status".to_string(),
            "".to_string(),
            format!("Failed to load estop status: {error}"),
        ],
    }
}

fn doctor_full_lines(config: &Config) -> Vec<String> {
    match doctor::run(config) {
        Ok(()) => vec![
            "Doctor (run)".to_string(),
            "".to_string(),
            "Doctor run completed.".to_string(),
            "Detailed diagnostics were emitted to terminal output.".to_string(),
        ],
        Err(error) => vec![
            "Doctor (run)".to_string(),
            "".to_string(),
            format!("Doctor run failed: {error}"),
        ],
    }
}

async fn doctor_models_lines(config: &Config) -> Vec<String> {
    match doctor::run_models(config, None, true).await {
        Ok(()) => vec![
            "Doctor Models (run)".to_string(),
            "".to_string(),
            "Model doctor probe completed (cache-first).".to_string(),
            "Detailed probe output was emitted to terminal output.".to_string(),
        ],
        Err(error) => vec![
            "Doctor Models (run)".to_string(),
            "".to_string(),
            format!("Model doctor probe failed: {error}"),
        ],
    }
}

async fn memory_list_lines(config: &Config) -> Vec<String> {
    match memory::cli::handle_command(
        crate::MemoryCommands::List {
            category: None,
            session: None,
            limit: 20,
            offset: 0,
        },
        config,
    )
    .await
    {
        Ok(()) => vec![
            "Memory List (run)".to_string(),
            "".to_string(),
            "Memory list command completed (limit=20, offset=0).".to_string(),
            "Detailed entries were emitted to terminal output.".to_string(),
        ],
        Err(error) => vec![
            "Memory List (run)".to_string(),
            "".to_string(),
            format!("Memory list failed: {error}"),
        ],
    }
}

fn hardware_discover_lines(config: &Config) -> Vec<String> {
    match hardware::handle_command(crate::HardwareCommands::Discover, config) {
        Ok(()) => vec![
            "Hardware Discover (run)".to_string(),
            "".to_string(),
            "Hardware discovery completed.".to_string(),
            "Detailed hardware output was emitted to terminal output.".to_string(),
        ],
        Err(error) => vec![
            "Hardware Discover (run)".to_string(),
            "".to_string(),
            format!("Hardware discovery failed: {error}"),
        ],
    }
}

async fn peripheral_list_lines(config: &Config) -> Vec<String> {
    match peripherals::handle_command(crate::PeripheralCommands::List, config).await {
        Ok(()) => vec![
            "Peripheral List (run)".to_string(),
            "".to_string(),
            "Peripheral listing completed.".to_string(),
            "Detailed peripheral output was emitted to terminal output.".to_string(),
        ],
        Err(error) => vec![
            "Peripheral List (run)".to_string(),
            "".to_string(),
            format!("Peripheral listing failed: {error}"),
        ],
    }
}

async fn channel_doctor_lines(config: &Config) -> Vec<String> {
    match channels::doctor_channels(config.clone()).await {
        Ok(()) => vec![
            "Channel Doctor (run)".to_string(),
            "".to_string(),
            "Channel doctor completed successfully.".to_string(),
            "Detailed probe logs are emitted to terminal output.".to_string(),
        ],
        Err(error) => vec![
            "Channel Doctor (run)".to_string(),
            "".to_string(),
            format!("Channel doctor failed: {error}"),
        ],
    }
}

async fn models_refresh_lines(config: &Config) -> Vec<String> {
    let provider = config.default_provider.as_deref();
    let provider_label = provider.unwrap_or("default");
    match onboard::run_models_refresh(config, provider, false).await {
        Ok(()) => vec![
            "Models Refresh (run)".to_string(),
            "".to_string(),
            format!("Model refresh completed for provider: {provider_label}"),
            "Detailed refresh logs are emitted to terminal output.".to_string(),
        ],
        Err(error) => vec![
            "Models Refresh (run)".to_string(),
            "".to_string(),
            format!("Model refresh failed for provider {provider_label}: {error}"),
        ],
    }
}

fn channel_lines(config: &Config) -> Vec<String> {
    let mut lines = vec![
        "Channels".to_string(),
        "".to_string(),
        "CLI: configured".to_string(),
    ];

    for (channel, configured) in config.channels_config.channels() {
        lines.push(format!(
            "{}: {}",
            channel.name(),
            if configured {
                "configured"
            } else {
                "not configured"
            }
        ));
    }

    lines
}

async fn auth_profile_lines(config: &Config) -> Vec<String> {
    let service = AuthService::from_config(config);
    let data = match service.load_profiles().await {
        Ok(data) => data,
        Err(error) => {
            return vec![
                "Auth Profiles".to_string(),
                "".to_string(),
                format!("Failed to load auth profiles: {error}"),
            ];
        }
    };

    let mut lines = vec![
        "Auth Profiles".to_string(),
        "".to_string(),
        format!("Total profiles: {}", data.profiles.len()),
    ];

    if data.profiles.is_empty() {
        lines.push("No auth profiles configured.".to_string());
        return lines;
    }

    for (profile_id, profile) in &data.profiles {
        let is_active = data
            .active_profiles
            .get(&profile.provider)
            .is_some_and(|active| active == profile_id);
        let marker = if is_active { " [active]" } else { "" };
        lines.push(format!(
            "- {} ({}){}",
            profile_id,
            profile.provider,
            marker
        ));
    }

    lines
}

fn status_lines(config: &Config) -> Vec<String> {
    let effective_memory_backend = memory::effective_memory_backend_name(
        &config.memory.backend,
        Some(&config.storage.provider.config),
    );

    vec![
        "Status".to_string(),
        "".to_string(),
        format!("Version: {}", env!("CARGO_PKG_VERSION")),
        format!("Workspace: {}", config.workspace_dir.display()),
        format!("Config: {}", config.config_path.display()),
        format!(
            "Provider: {}",
            config.default_provider.as_deref().unwrap_or("openrouter")
        ),
        format!(
            "Model: {}",
            config.default_model.as_deref().unwrap_or("(default)")
        ),
        format!("Memory backend: {effective_memory_backend}"),
        format!(
            "Auto-save: {}",
            if config.memory.auto_save { "on" } else { "off" }
        ),
    ]
}

fn provider_lines(config: &Config) -> Vec<String> {
    let providers = providers::list_providers();
    let active = config
        .default_provider
        .as_deref()
        .unwrap_or("openrouter")
        .trim()
        .to_ascii_lowercase();

    let mut lines = vec![
        "Providers".to_string(),
        "".to_string(),
        format!("Total providers: {}", providers.len()),
    ];

    for provider in providers {
        let is_active = provider.name.eq_ignore_ascii_case(&active)
            || provider
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(&active));
        let marker = if is_active { " [active]" } else { "" };
        let local_tag = if provider.local { " [local]" } else { "" };
        lines.push(format!(
            "- {}: {}{}{}",
            provider.name, provider.display_name, local_tag, marker
        ));
    }

    lines
}

fn models_list_lines(config: &Config) -> Vec<String> {
    let provider = config
        .default_provider
        .as_deref()
        .unwrap_or("openrouter")
        .trim()
        .to_string();

    let models = onboard::shared::curated_models_for_provider(&provider);

    let mut lines = vec![
        "Models List (curated)".to_string(),
        "".to_string(),
        format!("Provider: {provider}"),
        format!("Curated models: {}", models.len()),
    ];

    for (index, (model_id, description)) in models.into_iter().take(20).enumerate() {
        lines.push(format!("{}. {} — {}", index + 1, model_id, description));
    }

    if lines.len() == 4 {
        lines.push("No curated models available.".to_string());
    }

    lines
}

fn models_status_lines(config: &Config) -> Vec<String> {
    let provider = config
        .default_provider
        .as_deref()
        .unwrap_or("openrouter")
        .trim()
        .to_string();
    let selected = config
        .default_model
        .clone()
        .unwrap_or_else(|| onboard::shared::default_model_for_provider(&provider));

    vec![
        "Models Status".to_string(),
        "".to_string(),
        format!("Provider: {provider}"),
        format!("Configured model: {selected}"),
        format!(
            "Curated entries: {}",
            onboard::shared::curated_models_for_provider(&provider).len()
        ),
    ]
}

fn doctor_lines(config: &Config) -> Vec<String> {
    let configured_channels = config
        .channels_config
        .channels()
        .iter()
        .filter(|(_, enabled)| *enabled)
        .count();

    vec![
        "Doctor (readonly quick checks)".to_string(),
        "".to_string(),
        format!(
            "Config file exists: {}",
            if config.config_path.exists() {
                "yes"
            } else {
                "no"
            }
        ),
        format!(
            "Workspace exists: {}",
            if config.workspace_dir.exists() {
                "yes"
            } else {
                "no"
            }
        ),
        format!(
            "API key configured: {}",
            if config.api_key.is_some() { "yes" } else { "no" }
        ),
        format!("Configured channels: {configured_channels}"),
        format!("OTP enabled: {}", config.security.otp.enabled),
        format!("E-stop enabled: {}", config.security.estop.enabled),
    ]
}

fn memory_stats_lines(config: &Config) -> Vec<String> {
    vec![
        "Memory Stats".to_string(),
        "".to_string(),
        format!("Backend: {}", config.memory.backend),
        format!(
            "Auto-save: {}",
            if config.memory.auto_save { "on" } else { "off" }
        ),
        format!("Retention days: {}", config.memory.conversation_retention_days),
        format!(
            "Embedding: {}/{} (dim={})",
            config.memory.embedding_provider,
            config.memory.embedding_model,
            config.memory.embedding_dimensions
        ),
        format!("Workspace: {}", config.workspace_dir.display()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_items_produce_output() {
        let config = Config::default();
        for item in [
            MenuItem::Home,
            MenuItem::Status,
            MenuItem::Providers,
            MenuItem::ConfigSchema,
            MenuItem::EstopStatus,
            MenuItem::Channels,
            MenuItem::ChannelDoctor,
            MenuItem::AuthProfiles,
            MenuItem::ModelsList,
            MenuItem::ModelsStatus,
            MenuItem::ModelsRefresh,
            MenuItem::DoctorFull,
            MenuItem::DoctorModels,
            MenuItem::Doctor,
            MenuItem::MemoryList,
            MenuItem::MemoryStats,
            MenuItem::HardwareDiscover,
            MenuItem::PeripheralList,
        ] {
            let runtime = tokio::runtime::Runtime::new().expect("runtime should initialize");
            let lines = runtime.block_on(run(item, &config));
            assert!(!lines.is_empty());
        }
    }
}
