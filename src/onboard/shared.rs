use anyhow::Result;
use std::path::Path;

/// User-provided personalization baked into workspace MD files.
#[derive(Debug, Clone, Default)]
pub struct ProjectContext {
    pub user_name: String,
    pub timezone: String,
    pub agent_name: String,
    pub communication_style: String,
}

pub fn default_model_for_provider(provider: &str) -> String {
    super::wizard::default_model_for_provider(provider)
}

pub fn curated_models_for_provider(provider_name: &str) -> Vec<(String, String)> {
    super::wizard::curated_models_for_provider(provider_name)
}

pub fn fetch_live_models_for_provider(
    provider_name: &str,
    api_key: &str,
    provider_api_url: Option<&str>,
) -> Result<Vec<String>> {
    super::wizard::fetch_live_models_for_provider(provider_name, api_key, provider_api_url)
}

pub fn get_provider_tiers() -> Vec<&'static str> {
    super::wizard::get_provider_tiers()
}

pub fn get_providers_for_tier(tier_idx: usize) -> Vec<(&'static str, &'static str)> {
    super::wizard::get_providers_for_tier(tier_idx)
}

pub async fn scaffold_workspace(workspace_dir: &Path, ctx: &ProjectContext) -> Result<()> {
    super::wizard::scaffold_workspace(workspace_dir, ctx).await
}
