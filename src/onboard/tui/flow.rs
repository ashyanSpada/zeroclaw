use crate::{
    config::{
        schema::{
            default_nostr_relays, CloudflareTunnelConfig, CustomTunnelConfig, DingTalkConfig,
            FeishuConfig, IMessageConfig, IrcConfig, LarkConfig, LarkReceiveMode, LinqConfig,
            MatrixConfig, NextcloudTalkConfig, NgrokTunnelConfig, NostrConfig, QQConfig,
            SignalConfig, TailscaleTunnelConfig, WhatsAppConfig,
        },
        DiscordConfig, SlackConfig, StreamMode, TelegramConfig, WebhookConfig,
    },
    memory::{memory_backend_profile, selectable_memory_backends},
    onboard::shared,
};
use std::collections::BTreeSet;

use super::state::{
    App, ChannelChoice, OnboardingMode, ToolModeChoice, TunnelChoice, WizardStep,
    CUSTOM_MODEL_SENTINEL,
};

impl App<'_> {
    fn parse_list_csv(value: &str) -> Vec<String> {
        value
            .split(',')
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .map(ToString::to_string)
            .collect()
    }

    pub fn prepare_models(&mut self) {
        self.loading = true;
        self.status_message = format!("Fetching models for {}...", self.provider);

        let mut candidates: BTreeSet<String> = shared::curated_models_for_provider(&self.provider)
            .into_iter()
            .map(|(id, _)| id)
            .collect();

        match shared::fetch_live_models_for_provider(
            &self.provider,
            &self.api_key,
            self.api_url.as_deref(),
        ) {
            Ok(models) => {
                for model in models {
                    let trimmed = model.trim();
                    if !trimmed.is_empty() {
                        candidates.insert(trimmed.to_string());
                    }
                }
                self.status_message = "Loaded live + curated model catalog".to_string();
            }
            Err(error) => {
                self.status_message = format!("Live fetch unavailable: {error}");
            }
        }

        if candidates.is_empty() {
            candidates.insert(shared::default_model_for_provider(&self.provider));
        }

        let mut merged: Vec<String> = candidates.into_iter().collect();
        merged.push(CUSTOM_MODEL_SENTINEL.to_string());
        self.available_models = merged;
        self.model_list.select(Some(0));
        self.loading = false;
    }

    pub fn project_style_text(&self) -> String {
        match self.project_style_list.selected().unwrap_or(1) {
            0 => "Be direct and concise. Skip pleasantries. Get to the point.".to_string(),
            1 => "Be friendly, human, and conversational. Show warmth and empathy while staying efficient. Use natural contractions.".to_string(),
            2 => "Be professional and polished. Stay calm, structured, and respectful. Use occasional tone-setting emojis only when appropriate.".to_string(),
            3 => "Be expressive and playful when appropriate. Use relevant emojis naturally (0-2 max), and keep serious topics emoji-light.".to_string(),
            4 => "Be technical and detailed. Thorough explanations, code-first.".to_string(),
            5 => "Adapt to the situation. Default to warm and clear communication; be concise when needed, thorough when it matters.".to_string(),
            _ => Self::text_value(&self.project_style_custom_input),
        }
    }

    fn apply_channel_choice(&mut self) {
        let selected = self.channel_list.selected().unwrap_or(0);
        self.channel_choice = match selected {
            1 => ChannelChoice::Telegram,
            2 => ChannelChoice::Discord,
            3 => ChannelChoice::Slack,
            4 => ChannelChoice::IMessage,
            5 => ChannelChoice::Matrix,
            6 => ChannelChoice::Signal,
            7 => ChannelChoice::WhatsApp,
            8 => ChannelChoice::Linq,
            9 => ChannelChoice::Irc,
            10 => ChannelChoice::Webhook,
            11 => ChannelChoice::NextcloudTalk,
            12 => ChannelChoice::DingTalk,
            13 => ChannelChoice::QqOfficial,
            14 => ChannelChoice::Lark,
            15 => ChannelChoice::Feishu,
            16 => ChannelChoice::Nostr,
            _ => ChannelChoice::CliOnly,
        };

        self.channels_config = crate::config::ChannelsConfig::default();
        self.channels_config.cli = true;
    }

    fn apply_channel_token(&mut self) {
        let token = Self::text_value(&self.channel_token_input);
        let aux = Self::text_value(&self.channel_aux_input);
        let allowed_users = Self::parse_list_csv(&aux);

        match self.channel_choice {
            ChannelChoice::Telegram => {
                if !token.is_empty() {
                    self.channels_config.telegram = Some(TelegramConfig {
                        bot_token: token,
                        allowed_users,
                        stream_mode: StreamMode::default(),
                        draft_update_interval_ms: 1000,
                        interrupt_on_new_message: false,
                        mention_only: false,
                    });
                }
            }
            ChannelChoice::Discord => {
                if !token.is_empty() {
                    self.channels_config.discord = Some(DiscordConfig {
                        bot_token: token,
                        guild_id: None,
                        allowed_users,
                        listen_to_bots: false,
                        mention_only: false,
                    });
                }
            }
            ChannelChoice::Slack => {
                if !token.is_empty() {
                    self.channels_config.slack = Some(SlackConfig {
                        bot_token: token,
                        app_token: None,
                        channel_id: None,
                        allowed_users,
                    });
                }
            }
            ChannelChoice::Webhook => {
                let port = token.parse::<u16>().unwrap_or(8081);
                self.channels_config.webhook = Some(WebhookConfig {
                    port,
                    secret: if aux.is_empty() { None } else { Some(aux) },
                });
            }
            ChannelChoice::IMessage => {
                self.channels_config.imessage = Some(IMessageConfig {
                    allowed_contacts: allowed_users,
                });
            }
            ChannelChoice::Matrix => {
                self.channels_config.matrix = Some(MatrixConfig {
                    homeserver: if aux.is_empty() {
                        "https://matrix.org".to_string()
                    } else {
                        aux.clone()
                    },
                    access_token: token,
                    user_id: None,
                    device_id: None,
                    room_id: "!zeroclaw:matrix.org".to_string(),
                    allowed_users: vec!["*".to_string()],
                });
            }
            ChannelChoice::Signal => {
                self.channels_config.signal = Some(SignalConfig {
                    http_url: "http://127.0.0.1:8686".to_string(),
                    account: token,
                    group_id: if aux.is_empty() { None } else { Some(aux.clone()) },
                    allowed_from: vec!["*".to_string()],
                    ignore_attachments: false,
                    ignore_stories: true,
                });
            }
            ChannelChoice::WhatsApp => {
                self.channels_config.whatsapp = Some(WhatsAppConfig {
                    access_token: if token.is_empty() {
                        None
                    } else {
                        Some(token)
                    },
                    phone_number_id: if aux.is_empty() { None } else { Some(aux.clone()) },
                    verify_token: Some("zeroclaw".to_string()),
                    app_secret: None,
                    session_path: None,
                    pair_phone: None,
                    pair_code: None,
                    allowed_numbers: vec!["*".to_string()],
                });
            }
            ChannelChoice::Linq => {
                self.channels_config.linq = Some(LinqConfig {
                    api_token: token,
                    from_phone: if aux.is_empty() {
                        "+10000000000".to_string()
                    } else {
                        aux.clone()
                    },
                    signing_secret: None,
                    allowed_senders: vec!["*".to_string()],
                });
            }
            ChannelChoice::Irc => {
                self.channels_config.irc = Some(IrcConfig {
                    server: if token.is_empty() {
                        "irc.libera.chat".to_string()
                    } else {
                        token
                    },
                    port: 6697,
                    nickname: if aux.is_empty() {
                        "zeroclaw".to_string()
                    } else {
                        aux.clone()
                    },
                    username: None,
                    channels: vec!["#general".to_string()],
                    allowed_users: vec!["*".to_string()],
                    server_password: None,
                    nickserv_password: None,
                    sasl_password: None,
                    verify_tls: Some(true),
                });
            }
            ChannelChoice::NextcloudTalk => {
                self.channels_config.nextcloud_talk = Some(NextcloudTalkConfig {
                    base_url: token,
                    app_token: aux,
                    webhook_secret: None,
                    allowed_users: vec!["*".to_string()],
                });
            }
            ChannelChoice::DingTalk => {
                self.channels_config.dingtalk = Some(DingTalkConfig {
                    client_id: token,
                    client_secret: aux,
                    allowed_users: vec!["*".to_string()],
                });
            }
            ChannelChoice::QqOfficial => {
                self.channels_config.qq = Some(QQConfig {
                    app_id: token,
                    app_secret: aux,
                    allowed_users: vec!["*".to_string()],
                });
            }
            ChannelChoice::Lark => {
                self.channels_config.lark = Some(LarkConfig {
                    app_id: token,
                    app_secret: aux,
                    encrypt_key: None,
                    verification_token: None,
                    allowed_users: vec!["*".to_string()],
                    mention_only: false,
                    use_feishu: false,
                    receive_mode: LarkReceiveMode::Websocket,
                    port: None,
                });
            }
            ChannelChoice::Feishu => {
                self.channels_config.feishu = Some(FeishuConfig {
                    app_id: token,
                    app_secret: aux,
                    encrypt_key: None,
                    verification_token: None,
                    allowed_users: vec!["*".to_string()],
                    receive_mode: LarkReceiveMode::Websocket,
                    port: None,
                });
            }
            ChannelChoice::Nostr => {
                self.channels_config.nostr = Some(NostrConfig {
                    private_key: token,
                    relays: default_nostr_relays(),
                    allowed_pubkeys: if aux.is_empty() {
                        vec!["*".to_string()]
                    } else {
                        Self::parse_list_csv(&aux)
                    },
                });
            }
            ChannelChoice::CliOnly => {}
        }
    }

    fn apply_tunnel_choice(&mut self) {
        self.tunnel_choice = match self.tunnel_list.selected().unwrap_or(0) {
            1 => TunnelChoice::Cloudflare,
            2 => TunnelChoice::Tailscale,
            3 => TunnelChoice::Ngrok,
            4 => TunnelChoice::Custom,
            _ => TunnelChoice::None,
        };
    }

    pub fn next_step(&mut self) {
        match self.step {
            WizardStep::Welcome => {
                if self.has_existing_config() && !self.force {
                    self.step = WizardStep::ConfigModeSelection;
                } else {
                    self.mode = OnboardingMode::FullOnboarding;
                    self.step = WizardStep::WorkspaceSetup;
                }
            }
            WizardStep::ConfigModeSelection => {
                self.mode = if self.mode_list.selected().unwrap_or(1) == 0 {
                    OnboardingMode::FullOnboarding
                } else {
                    OnboardingMode::UpdateProviderOnly
                };
                self.step = WizardStep::WorkspaceSetup;
            }
            WizardStep::WorkspaceSetup => self.step = WizardStep::ProviderTierSelection,
            WizardStep::ProviderTierSelection => {
                let tier_idx = self.provider_tier_list.selected().unwrap_or(0);
                self.current_tier_providers = shared::get_providers_for_tier(tier_idx);
                if self.current_tier_providers.is_empty() {
                    self.step = WizardStep::CustomProviderUrlEntry;
                } else {
                    self.step = WizardStep::ProviderSelection;
                }
            }
            WizardStep::ProviderSelection => {
                let idx = self.provider_list.selected().unwrap_or(0);
                if let Some((id, _)) = self.current_tier_providers.get(idx) {
                    self.provider = id.to_string();
                    self.api_url = None;
                    if Self::needs_provider_endpoint(&self.provider) {
                        self.step = WizardStep::ProviderEndpointEntry;
                    } else {
                        self.step = WizardStep::ApiKeyEntry;
                    }
                }
            }
            WizardStep::CustomProviderUrlEntry => {
                let input = Self::text_value(&self.custom_provider_url_input);
                let normalized = input.trim_end_matches('/');
                if !normalized.is_empty() {
                    self.provider = format!("custom:{normalized}");
                    self.api_url = None;
                    self.step = WizardStep::ApiKeyEntry;
                }
            }
            WizardStep::ProviderEndpointEntry => {
                let input = Self::text_value(&self.provider_endpoint_input);
                let normalized = input.trim_end_matches('/').to_string();
                if !normalized.is_empty() {
                    self.api_url = Some(normalized);
                    self.step = WizardStep::ApiKeyEntry;
                }
            }
            WizardStep::ApiKeyEntry => {
                self.api_key = Self::text_value(&self.api_key_input);
                self.prepare_models();
                self.step = WizardStep::ModelSelection;
            }
            WizardStep::ModelSelection => {
                let idx = self.model_list.selected().unwrap_or(0);
                if let Some(selected) = self.available_models.get(idx) {
                    if selected == CUSTOM_MODEL_SENTINEL {
                        self.step = WizardStep::ModelCustomEntry;
                    } else {
                        self.model = selected.clone();
                        self.step = WizardStep::ChannelSelection;
                    }
                }
            }
            WizardStep::ModelCustomEntry => {
                let typed = Self::text_value(&self.model_custom_input);
                if !typed.is_empty() {
                    self.model = typed;
                    self.step = WizardStep::ChannelSelection;
                }
            }
            WizardStep::ChannelSelection => {
                self.apply_channel_choice();
                match self.channel_choice {
                    ChannelChoice::CliOnly => self.step = WizardStep::TunnelSelection,
                    ChannelChoice::IMessage => self.step = WizardStep::ChannelAuxEntry,
                    ChannelChoice::Webhook => self.step = WizardStep::ChannelTokenEntry,
                    _ => self.step = WizardStep::ChannelTokenEntry,
                }
            }
            WizardStep::ChannelTokenEntry => match self.channel_choice {
                ChannelChoice::Webhook => {
                    self.step = WizardStep::ChannelAuxEntry;
                }
                ChannelChoice::CliOnly => self.step = WizardStep::TunnelSelection,
                _ => self.step = WizardStep::ChannelAuxEntry,
            },
            WizardStep::ChannelAuxEntry => {
                self.apply_channel_token();
                self.step = WizardStep::TunnelSelection;
            }
            WizardStep::TunnelSelection => {
                self.apply_tunnel_choice();
                self.step = match self.tunnel_choice {
                    TunnelChoice::None => WizardStep::ToolModeSelection,
                    _ => WizardStep::TunnelPrimaryEntry,
                };
            }
            WizardStep::TunnelPrimaryEntry => {
                self.step = match self.tunnel_choice {
                    TunnelChoice::Cloudflare => WizardStep::ToolModeSelection,
                    _ => WizardStep::TunnelSecondaryEntry,
                };
            }
            WizardStep::TunnelSecondaryEntry => self.step = WizardStep::ToolModeSelection,
            WizardStep::ToolModeSelection => {
                self.tool_mode_choice = if self.tool_mode_list.selected().unwrap_or(0) == 1 {
                    ToolModeChoice::Composio
                } else {
                    ToolModeChoice::Sovereign
                };
                self.step = match self.tool_mode_choice {
                    ToolModeChoice::Composio => WizardStep::ComposioApiKeyEntry,
                    ToolModeChoice::Sovereign => WizardStep::SecretsEncryptChoice,
                };
            }
            WizardStep::ComposioApiKeyEntry => self.step = WizardStep::SecretsEncryptChoice,
            WizardStep::SecretsEncryptChoice => self.step = WizardStep::HardwareSelection,
            WizardStep::HardwareSelection => {
                self.hardware_choice = self.hardware_list.selected().unwrap_or(3);
                self.step = WizardStep::MemorySelection;
            }
            WizardStep::MemorySelection => {
                self.memory_choice = self.memory_list.selected().unwrap_or(0);
                let backend = selectable_memory_backends()
                    .get(self.memory_choice)
                    .map_or("sqlite", |b| b.key);
                self.memory_auto_save = memory_backend_profile(backend).auto_save_default;
                self.step = WizardStep::ProjectUserEntry;
            }
            WizardStep::ProjectUserEntry => self.step = WizardStep::ProjectTimezoneEntry,
            WizardStep::ProjectTimezoneEntry => self.step = WizardStep::ProjectAgentEntry,
            WizardStep::ProjectAgentEntry => self.step = WizardStep::ProjectStyleSelection,
            WizardStep::ProjectStyleSelection => {
                if self.project_style_list.selected().unwrap_or(1) == 6 {
                    self.step = WizardStep::ProjectStyleCustomEntry;
                } else {
                    self.step = WizardStep::Confirmation;
                }
            }
            WizardStep::ProjectStyleCustomEntry => self.step = WizardStep::Confirmation,
            WizardStep::Confirmation => self.step = WizardStep::Done,
            WizardStep::Done => {}
        }
    }

    pub fn tunnel_config(&self) -> crate::config::TunnelConfig {
        let primary = Self::text_value(&self.tunnel_primary_input);
        let secondary = Self::text_value(&self.tunnel_secondary_input);
        match self.tunnel_choice {
            TunnelChoice::Cloudflare => crate::config::TunnelConfig {
                provider: "cloudflare".into(),
                cloudflare: Some(CloudflareTunnelConfig { token: primary }),
                tailscale: None,
                ngrok: None,
                custom: None,
            },
            TunnelChoice::Tailscale => crate::config::TunnelConfig {
                provider: "tailscale".into(),
                cloudflare: None,
                tailscale: Some(TailscaleTunnelConfig {
                    funnel: self.tunnel_toggle,
                    hostname: if secondary.is_empty() {
                        None
                    } else {
                        Some(secondary)
                    },
                }),
                ngrok: None,
                custom: None,
            },
            TunnelChoice::Ngrok => crate::config::TunnelConfig {
                provider: "ngrok".into(),
                cloudflare: None,
                tailscale: None,
                ngrok: Some(NgrokTunnelConfig {
                    auth_token: primary,
                    domain: if secondary.is_empty() {
                        None
                    } else {
                        Some(secondary)
                    },
                }),
                custom: None,
            },
            TunnelChoice::Custom => crate::config::TunnelConfig {
                provider: "custom".into(),
                cloudflare: None,
                tailscale: None,
                ngrok: None,
                custom: Some(CustomTunnelConfig {
                    start_command: primary,
                    health_url: if secondary.is_empty() {
                        None
                    } else {
                        Some(secondary)
                    },
                    url_pattern: None,
                }),
            },
            TunnelChoice::None => crate::config::TunnelConfig::default(),
        }
    }
}
