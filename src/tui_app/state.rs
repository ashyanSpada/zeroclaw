use ratatui::widgets::ListState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuItem {
    Home,
    Status,
    Providers,
    ModelsList,
    ModelsStatus,
    Doctor,
    MemoryStats,
}

impl MenuItem {
    pub fn title(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Status => "Status",
            Self::Providers => "Providers",
            Self::ModelsList => "Models List",
            Self::ModelsStatus => "Models Status",
            Self::Doctor => "Doctor (readonly)",
            Self::MemoryStats => "Memory Stats",
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
                MenuItem::ModelsList,
                MenuItem::ModelsStatus,
                MenuItem::Doctor,
                MenuItem::MemoryStats,
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
