use crate::{
    config::{ComposioConfig, Config, HardwareConfig, MemoryConfig, SecretsConfig},
    hardware,
    memory::{memory_backend_profile, selectable_memory_backends},
    onboard::shared,
};
use anyhow::{Context, Result};
use tokio::fs;

use super::state::{App, OnboardingMode, ToolModeChoice};

fn memory_config_defaults_for_backend(backend: &str) -> MemoryConfig {
    let profile = memory_backend_profile(backend);

    MemoryConfig {
        backend: backend.to_string(),
        auto_save: profile.auto_save_default,
        hygiene_enabled: profile.uses_sqlite_hygiene,
        archive_after_days: if profile.uses_sqlite_hygiene { 7 } else { 0 },
        purge_after_days: if profile.uses_sqlite_hygiene { 30 } else { 0 },
        conversation_retention_days: 30,
        embedding_provider: "none".to_string(),
        embedding_model: "text-embedding-3-small".to_string(),
        embedding_dimensions: 1536,
        vector_weight: 0.7,
        keyword_weight: 0.3,
        min_relevance_score: 0.4,
        embedding_cache_size: if profile.uses_sqlite_hygiene { 10000 } else { 0 },
        chunk_max_tokens: 512,
        response_cache_enabled: false,
        response_cache_ttl_minutes: 60,
        response_cache_max_entries: 5_000,
        snapshot_enabled: false,
        snapshot_on_hygiene: false,
        auto_hydrate: true,
        sqlite_open_timeout_secs: None,
        qdrant: crate::config::QdrantConfig::default(),
    }
}

pub async fn finalize_config(app: &App<'_>) -> Result<Config> {
    let provider = if app.provider.trim().is_empty() {
        "openrouter".to_string()
    } else {
        app.provider.trim().to_string()
    };
    let model = if app.model.trim().is_empty() {
        shared::default_model_for_provider(&provider)
    } else {
        app.model.trim().to_string()
    };

    let mut config = if app.mode == OnboardingMode::UpdateProviderOnly && app.config_path.exists() {
        let raw = fs::read_to_string(&app.config_path).await.with_context(|| {
            format!(
                "Failed to read existing config at {}",
                app.config_path.display()
            )
        })?;
        let mut loaded: Config = toml::from_str(&raw).with_context(|| {
            format!(
                "Failed to parse existing config at {}",
                app.config_path.display()
            )
        })?;
        loaded.workspace_dir = app.workspace_dir.clone();
        loaded.config_path = app.config_path.clone();
        loaded
    } else {
        let mut fresh = Config::default();
        fresh.workspace_dir = app.workspace_dir.clone();
        fresh.config_path = app.config_path.clone();
        fresh
    };

    config.default_provider = Some(provider);
    config.default_model = Some(model);
    config.api_url = app.api_url.clone();
    config.api_key = if app.api_key.trim().is_empty() {
        None
    } else {
        Some(app.api_key.trim().to_string())
    };

    if app.mode == OnboardingMode::FullOnboarding {
        config.channels_config = app.channels_config.clone();
        config.tunnel = app.tunnel_config();

        config.composio = match app.tool_mode_choice {
            ToolModeChoice::Composio => ComposioConfig {
                enabled: true,
                api_key: {
                    let key = App::text_value(&app.composio_key_input);
                    if key.is_empty() { None } else { Some(key) }
                },
                entity_id: "default".to_string(),
            },
            ToolModeChoice::Sovereign => ComposioConfig::default(),
        };
        config.secrets = SecretsConfig {
            encrypt: app.secrets_encrypt,
        };

        let devices = hardware::discover_hardware();
        let mut hardware_config: HardwareConfig =
            hardware::config_from_wizard_choice(app.hardware_choice, &devices);
        hardware_config.workspace_datasheets = app.hardware_datasheets;
        config.hardware = hardware_config;

        let backend = selectable_memory_backends()
            .get(app.memory_choice)
            .map_or("sqlite", |backend| backend.key);
        let mut memory = memory_config_defaults_for_backend(backend);
        memory.auto_save = app.memory_auto_save;
        config.memory = memory;
    }

    config.save().await?;

    let config_dir = config
        .config_path
        .parent()
        .context("Config path must have a parent directory")?;
    crate::config::schema::persist_active_workspace_config_dir(config_dir).await?;

    if app.mode == OnboardingMode::FullOnboarding {
        let default_ctx = shared::ProjectContext {
            user_name: {
                let typed = App::text_value(&app.project_user_input);
                if typed.is_empty() {
                    std::env::var("USER").unwrap_or_else(|_| "User".into())
                } else {
                    typed
                }
            },
            timezone: {
                let typed = App::text_value(&app.project_timezone_input);
                if typed.is_empty() {
                    "UTC".into()
                } else {
                    typed
                }
            },
            agent_name: {
                let typed = App::text_value(&app.project_agent_input);
                if typed.is_empty() {
                    "ZeroClaw".into()
                } else {
                    typed
                }
            },
            communication_style: app.project_style_text(),
        };
        shared::scaffold_workspace(&config.workspace_dir, &default_ctx).await?;

        let has_channels = config
            .channels_config
            .channels_except_webhook()
            .iter()
            .any(|(_, ok)| *ok);
        if has_channels && config.api_key.is_some() {
            std::env::set_var("ZEROCLAW_AUTOSTART_CHANNELS", "1");
        }
    }

    Ok(config)
}
