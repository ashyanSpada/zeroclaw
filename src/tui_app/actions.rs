use crate::{config::Config, memory, onboard::shared, providers};

use super::state::MenuItem;

pub fn run(item: MenuItem, config: &Config) -> Vec<String> {
    match item {
        MenuItem::Home => vec![
            "ZeroClaw TUI Dashboard".to_string(),
            "".to_string(),
            "This dashboard provides read-only command views.".to_string(),
            "Select a menu item and press Enter.".to_string(),
        ],
        MenuItem::Status => status_lines(config),
        MenuItem::Providers => provider_lines(config),
        MenuItem::ModelsList => models_list_lines(config),
        MenuItem::ModelsStatus => models_status_lines(config),
        MenuItem::Doctor => doctor_lines(config),
        MenuItem::MemoryStats => memory_stats_lines(config),
    }
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

    let models = shared::curated_models_for_provider(&provider);

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
        .unwrap_or_else(|| shared::default_model_for_provider(&provider));

    vec![
        "Models Status".to_string(),
        "".to_string(),
        format!("Provider: {provider}"),
        format!("Configured model: {selected}"),
        format!(
            "Curated entries: {}",
            shared::curated_models_for_provider(&provider).len()
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
            MenuItem::ModelsList,
            MenuItem::ModelsStatus,
            MenuItem::Doctor,
            MenuItem::MemoryStats,
        ] {
            let lines = run(item, &config);
            assert!(!lines.is_empty());
        }
    }
}
