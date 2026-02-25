use crate::{config::ChannelsConfig, onboard::wizard};
use ratatui::widgets::ListState;
use std::path::PathBuf;
use tui_textarea::TextArea;

pub const CUSTOM_MODEL_SENTINEL: &str = "__custom_model__";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnboardingMode {
    FullOnboarding,
    UpdateProviderOnly,
}

#[derive(PartialEq)]
pub enum WizardStep {
    Welcome,
    ConfigModeSelection,
    WorkspaceSetup,
    ProviderTierSelection,
    ProviderSelection,
    CustomProviderUrlEntry,
    ProviderEndpointEntry,
    ApiKeyEntry,
    ModelSelection,
    ModelCustomEntry,
    ChannelSelection,
    ChannelTokenEntry,
    ChannelAuxEntry,
    TunnelSelection,
    TunnelPrimaryEntry,
    TunnelSecondaryEntry,
    ToolModeSelection,
    ComposioApiKeyEntry,
    SecretsEncryptChoice,
    HardwareSelection,
    MemorySelection,
    ProjectUserEntry,
    ProjectTimezoneEntry,
    ProjectAgentEntry,
    ProjectStyleSelection,
    ProjectStyleCustomEntry,
    Confirmation,
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelChoice {
    CliOnly,
    Telegram,
    Discord,
    Slack,
    IMessage,
    Matrix,
    Signal,
    WhatsApp,
    Linq,
    Irc,
    Webhook,
    NextcloudTalk,
    DingTalk,
    QqOfficial,
    Lark,
    Feishu,
    Nostr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TunnelChoice {
    None,
    Cloudflare,
    Tailscale,
    Ngrok,
    Custom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolModeChoice {
    Sovereign,
    Composio,
}

pub struct App<'a> {
    pub step: WizardStep,
    pub loading: bool,
    pub status_message: String,

    pub config_dir: PathBuf,
    pub config_path: PathBuf,
    pub workspace_dir: PathBuf,
    pub mode: OnboardingMode,
    pub force: bool,

    pub provider: String,
    pub api_key: String,
    pub api_url: Option<String>,
    pub model: String,

    pub workspace_input: TextArea<'a>,
    pub use_default_workspace: bool,
    pub custom_provider_url_input: TextArea<'a>,
    pub provider_endpoint_input: TextArea<'a>,

    pub provider_tier_list: ListState,
    pub provider_list: ListState,
    pub mode_list: ListState,
    pub model_list: ListState,

    pub api_key_input: TextArea<'a>,
    pub model_custom_input: TextArea<'a>,

    pub provider_tiers: Vec<&'static str>,
    pub current_tier_providers: Vec<(&'static str, &'static str)>,
    pub available_models: Vec<String>,

    pub channel_choice: ChannelChoice,
    pub channel_list: ListState,
    pub channel_token_input: TextArea<'a>,
    pub channel_aux_input: TextArea<'a>,
    pub channels_config: ChannelsConfig,

    pub tunnel_choice: TunnelChoice,
    pub tunnel_list: ListState,
    pub tunnel_primary_input: TextArea<'a>,
    pub tunnel_secondary_input: TextArea<'a>,
    pub tunnel_toggle: bool,

    pub tool_mode_choice: ToolModeChoice,
    pub tool_mode_list: ListState,
    pub composio_key_input: TextArea<'a>,
    pub secrets_encrypt: bool,

    pub hardware_choice: usize,
    pub hardware_list: ListState,
    pub hardware_datasheets: bool,

    pub memory_choice: usize,
    pub memory_list: ListState,
    pub memory_auto_save: bool,

    pub project_user_input: TextArea<'a>,
    pub project_timezone_input: TextArea<'a>,
    pub project_agent_input: TextArea<'a>,
    pub project_style_list: ListState,
    pub project_style_custom_input: TextArea<'a>,
}

impl<'a> App<'a> {
    pub fn new(force: bool) -> Self {
        let mut workspace_input = TextArea::default();
        workspace_input.set_placeholder_text("Enter custom workspace path...");

        let mut api_key_input = TextArea::default();
        api_key_input.set_placeholder_text("sk-...");
        api_key_input.set_mask_char('•');

        let mut custom_provider_url_input = TextArea::default();
        custom_provider_url_input.set_placeholder_text("https://your-openai-compatible-api/v1");

        let mut provider_endpoint_input = TextArea::default();
        provider_endpoint_input.set_placeholder_text("http://localhost:8000/v1");

        let mut model_custom_input = TextArea::default();
        model_custom_input.set_placeholder_text("gpt-5.2");

        let mut mode_list = ListState::default();
        mode_list.select(Some(1));

        let mut channel_token_input = TextArea::default();
        channel_token_input.set_placeholder_text("Token / API key");
        let mut channel_aux_input = TextArea::default();
        channel_aux_input.set_placeholder_text("Allowed users (comma-separated) or secret");

        let mut tunnel_primary_input = TextArea::default();
        tunnel_primary_input.set_placeholder_text("Primary tunnel field");
        let mut tunnel_secondary_input = TextArea::default();
        tunnel_secondary_input.set_placeholder_text("Secondary field (optional)");

        let mut composio_key_input = TextArea::default();
        composio_key_input.set_placeholder_text("Composio API key (optional)");

        let mut project_user_input = TextArea::default();
        project_user_input.set_placeholder_text("User");
        let mut project_timezone_input = TextArea::default();
        project_timezone_input.set_placeholder_text("UTC");
        let mut project_agent_input = TextArea::default();
        project_agent_input.set_placeholder_text("ZeroClaw");
        let mut project_style_custom_input = TextArea::default();
        project_style_custom_input.set_placeholder_text("Write custom communication style");

        let mut channel_list = ListState::default();
        channel_list.select(Some(0));
        let mut tunnel_list = ListState::default();
        tunnel_list.select(Some(0));
        let mut tool_mode_list = ListState::default();
        tool_mode_list.select(Some(0));
        let mut hardware_list = ListState::default();
        hardware_list.select(Some(3));
        let mut memory_list = ListState::default();
        memory_list.select(Some(0));
        let mut project_style_list = ListState::default();
        project_style_list.select(Some(1));

        Self {
            step: WizardStep::Welcome,
            loading: false,
            status_message: String::new(),
            config_dir: PathBuf::new(),
            config_path: PathBuf::new(),
            workspace_dir: PathBuf::new(),
            mode: OnboardingMode::FullOnboarding,
            force,
            provider: String::new(),
            api_key: String::new(),
            api_url: None,
            model: String::new(),
            workspace_input,
            use_default_workspace: true,
            custom_provider_url_input,
            provider_endpoint_input,
            provider_tier_list: ListState::default(),
            provider_list: ListState::default(),
            mode_list,
            model_list: ListState::default(),
            api_key_input,
            model_custom_input,
            provider_tiers: wizard::get_provider_tiers(),
            current_tier_providers: Vec::new(),
            available_models: Vec::new(),
            channel_choice: ChannelChoice::CliOnly,
            channel_list,
            channel_token_input,
            channel_aux_input,
            channels_config: ChannelsConfig::default(),
            tunnel_choice: TunnelChoice::None,
            tunnel_list,
            tunnel_primary_input,
            tunnel_secondary_input,
            tunnel_toggle: false,
            tool_mode_choice: ToolModeChoice::Sovereign,
            tool_mode_list,
            composio_key_input,
            secrets_encrypt: true,
            hardware_choice: 3,
            hardware_list,
            hardware_datasheets: false,
            memory_choice: 0,
            memory_list,
            memory_auto_save: true,
            project_user_input,
            project_timezone_input,
            project_agent_input,
            project_style_list,
            project_style_custom_input,
        }
    }

    pub fn has_existing_config(&self) -> bool {
        self.config_path.exists()
    }

    pub fn text_value(input: &TextArea<'_>) -> String {
        input
            .lines()
            .first()
            .map(|line| line.trim().to_string())
            .unwrap_or_default()
    }

    pub fn needs_provider_endpoint(provider: &str) -> bool {
        matches!(provider, "llamacpp" | "sglang" | "vllm" | "osaurus")
    }
}
