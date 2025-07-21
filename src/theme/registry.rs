// Theme registry for managing theme state and updates
// Provides reactive theme switching throughout the application

// Removed unused gpui import during cleanup
use std::sync::Arc;
use crate::theme::colors::{ThemeColors, ThemeMode};
use crate::theme::settings::ThemeSettings;

/// Global theme registry that manages theme state
#[derive(Clone)]
pub struct ThemeRegistry {
    settings: Arc<ThemeSettings>,
    current_colors: Arc<ThemeColors>,
}

impl ThemeRegistry {
    /// Create a new theme registry with loaded settings
    pub fn new() -> Self {
        let settings = ThemeSettings::load();
        let current_colors = Self::colors_for_mode(settings.effective_mode());
        
        Self {
            settings: Arc::new(settings),
            current_colors: Arc::new(current_colors),
        }
    }
    
    /// Get the current theme colors
    pub fn colors(&self) -> &ThemeColors {
        &self.current_colors
    }
    
    /// Get the current theme mode
    pub fn mode(&self) -> ThemeMode {
        self.settings.effective_mode()
    }
    
    /// Get colors for a specific theme mode
    fn colors_for_mode(mode: ThemeMode) -> ThemeColors {
        match mode {
            ThemeMode::Light | ThemeMode::System => ThemeColors::light(),
            ThemeMode::Dark => ThemeColors::dark(),
        }
    }
    
    // Removed unused set_mode method during cleanup
    
    /// Toggle between light and dark modes
    pub fn toggle_mode(&mut self) -> anyhow::Result<()> {
        let mut settings = (*self.settings).clone();
        settings.toggle_mode()?;
        
        let new_colors = Self::colors_for_mode(settings.effective_mode());
        
        self.settings = Arc::new(settings);
        self.current_colors = Arc::new(new_colors);
        
        Ok(())
    }
    
    /// Check if current theme is dark
    pub fn is_dark(&self) -> bool {
        matches!(self.mode(), ThemeMode::Dark)
    }
    
    // Removed unused is_light and refresh methods during cleanup
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Removed unused ThemeModel and ThemeAction during cleanup