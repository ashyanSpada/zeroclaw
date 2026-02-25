use ratatui::widgets::ListState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuItem {
    Home,
    Status,
    Providers,
    ConfigSchema,
    EstopStatus,
    Channels,
    ChannelDoctor,
    AuthProfiles,
    ModelsList,
    ModelsStatus,
    ModelsRefresh,
    DoctorFull,
    DoctorModels,
    Doctor,
    MemoryList,
    MemoryStats,
    HardwareDiscover,
    PeripheralList,
}

impl MenuItem {
    pub fn title(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Status => "Status",
            Self::Providers => "Providers",
            Self::ConfigSchema => "Config Schema",
            Self::EstopStatus => "Estop Status",
            Self::Channels => "Channels",
            Self::ChannelDoctor => "Channel Doctor (run)",
            Self::AuthProfiles => "Auth Profiles",
            Self::ModelsList => "Models List",
            Self::ModelsStatus => "Models Status",
            Self::ModelsRefresh => "Models Refresh (run)",
            Self::DoctorFull => "Doctor (run)",
            Self::DoctorModels => "Doctor Models (run)",
            Self::Doctor => "Doctor (readonly)",
            Self::MemoryList => "Memory List (run)",
            Self::MemoryStats => "Memory Stats",
            Self::HardwareDiscover => "Hardware Discover (run)",
            Self::PeripheralList => "Peripheral List (run)",
        }
    }
}

pub struct AppState {
    pub menu: ListState,
    pub items: Vec<MenuItem>,
    pub output: Vec<String>,
}

impl AppState {
    pub fn new() -> Self {
        let mut menu = ListState::default();
        menu.select(Some(0));

        Self {
            menu,
            items: vec![
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
            ],
            output: vec![
                "ZeroClaw TUI Dashboard".to_string(),
                "".to_string(),
                "Use ↑/↓ to select, Enter to run, q to quit.".to_string(),
            ],
        }
    }

    pub fn selected_item(&self) -> MenuItem {
        let index = self.menu.selected().unwrap_or(0);
        self.items.get(index).copied().unwrap_or(MenuItem::Home)
    }
}
