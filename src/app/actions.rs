// GPUI Actions
// These define the actions that can be performed in the application
// Used for keybindings and user interactions

use gpui::actions;

actions!(dashboard, [SwitchTab, Refresh, Export, Search]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DashboardTab {
    Overview,
    Models,
    Projects,
    Sessions,
    Timeline,
}

impl DashboardTab {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Overview,
            Self::Models,
            Self::Projects,
            Self::Sessions,
            Self::Timeline,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Models => "Models",
            Self::Projects => "Projects",
            Self::Sessions => "Sessions",
            Self::Timeline => "Timeline",
        }
    }
}