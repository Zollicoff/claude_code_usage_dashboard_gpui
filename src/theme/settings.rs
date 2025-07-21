// Theme settings and persistence
// Handles user theme preferences and system theme detection

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::theme::colors::ThemeMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    pub mode: ThemeMode,
    pub auto_switch: bool, // Follow system theme
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            mode: ThemeMode::System,
            auto_switch: true,
        }
    }
}

impl ThemeSettings {
    /// Get the settings file path
    fn settings_path() -> anyhow::Result<PathBuf> {
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let config_dir = home_dir.join(".config").join("usage-dashboard");
        fs::create_dir_all(&config_dir)?;
        Ok(config_dir.join("theme.json"))
    }
    
    /// Load theme settings from disk
    pub fn load() -> Self {
        match Self::load_from_disk() {
            Ok(settings) => settings,
            Err(_) => {
                let default_settings = Self::default();
                // Try to save default settings
                let _ = default_settings.save();
                default_settings
            }
        }
    }
    
    fn load_from_disk() -> anyhow::Result<Self> {
        let path = Self::settings_path()?;
        let content = fs::read_to_string(path)?;
        let settings: ThemeSettings = serde_json::from_str(&content)?;
        Ok(settings)
    }
    
    /// Save theme settings to disk
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::settings_path()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    /// Get the effective theme mode (resolving System to Light/Dark)
    pub fn effective_mode(&self) -> ThemeMode {
        match self.mode {
            ThemeMode::System => {
                // Try to detect system theme, fallback to Light
                if self.auto_switch {
                    Self::detect_system_theme().unwrap_or(ThemeMode::Light)
                } else {
                    ThemeMode::Light
                }
            }
            mode => mode,
        }
    }
    
    /// Detect system theme preference (macOS implementation)
    fn detect_system_theme() -> Option<ThemeMode> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            
            let output = Command::new("defaults")
                .args(&["read", "-g", "AppleInterfaceStyle"])
                .output()
                .ok()?;
                
            if output.status.success() {
                let style = String::from_utf8_lossy(&output.stdout);
                if style.trim() == "Dark" {
                    Some(ThemeMode::Dark)
                } else {
                    Some(ThemeMode::Light)
                }
            } else {
                // If the command fails, it usually means light mode
                Some(ThemeMode::Light)
            }
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            // Default to light mode on other platforms
            Some(ThemeMode::Light)
        }
    }
    
    /// Update theme mode and save
    pub fn set_mode(&mut self, mode: ThemeMode) -> anyhow::Result<()> {
        self.mode = mode;
        self.save()
    }
    
    /// Toggle between light and dark mode
    pub fn toggle_mode(&mut self) -> anyhow::Result<()> {
        let new_mode = match self.effective_mode() {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::System => ThemeMode::Light, // If system, switch to explicit light
        };
        self.set_mode(new_mode)
    }
}

// Implement Serialize/Deserialize for ThemeMode
impl Serialize for ThemeMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark", 
            ThemeMode::System => "system",
        };
        serializer.serialize_str(s)
    }
}

impl<'de> Deserialize<'de> for ThemeMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "light" => Ok(ThemeMode::Light),
            "dark" => Ok(ThemeMode::Dark),
            "system" => Ok(ThemeMode::System),
            _ => Ok(ThemeMode::System), // Default fallback
        }
    }
}