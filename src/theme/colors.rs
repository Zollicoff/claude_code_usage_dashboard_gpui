// Color definitions based on Zed's theme system
// Provides semantic color roles for light and dark modes

use gpui::{hsla, Hsla};

#[derive(Debug, Clone, Copy)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone)]
pub struct ThemeColors {
    // Background colors
    pub background: Hsla,
    pub surface: Hsla,
    pub elevated_surface: Hsla,
    
    // Text colors
    pub text: Hsla,
    pub text_muted: Hsla,
    pub text_accent: Hsla,
    
    // Border colors
    pub border: Hsla,
    
    // Status colors
    pub success: Hsla,
    
    // Metric colors for dashboard cards
    pub metric_primary: Hsla,
    pub metric_secondary: Hsla,
    pub metric_tertiary: Hsla,
    pub metric_quaternary: Hsla,
    
    // Interactive colors - removed unused colors during cleanup
}

impl ThemeColors {
    /// Light theme colors based on Zed's light theme
    pub fn light() -> Self {
        Self {
            // Light theme backgrounds
            background: hsla(0.0, 0.0, 0.98, 1.0),        // #f9f9f8
            surface: hsla(0.0, 0.0, 0.94, 1.0),           // #f0f0ef  
            elevated_surface: hsla(0.0, 0.0, 1.0, 1.0),   // #ffffff
            
            // Light theme text
            text: hsla(0.0, 0.0, 0.1, 1.0),               // #191918
            text_muted: hsla(0.0, 0.0, 0.42, 1.0),        // #6c6b69
            text_accent: hsla(210.0 / 360.0, 1.0, 0.5, 1.0),      // #0090ff
            
            // Light theme borders
            border: hsla(0.0, 0.0, 0.89, 1.0),            // #e3e2e0
            
            // Status colors (same for light/dark)
            success: hsla(145.0 / 360.0, 0.53, 0.42, 1.0),        // #30a46c - green
            
            // Metric colors for dashboard cards
            metric_primary: hsla(210.0 / 360.0, 1.0, 0.5, 1.0),     // Blue
            metric_secondary: hsla(145.0 / 360.0, 0.53, 0.42, 1.0), // Green
            metric_tertiary: hsla(260.0 / 360.0, 0.6, 0.55, 1.0),   // Purple
            metric_quaternary: hsla(35.0 / 360.0, 0.91, 0.55, 1.0), // Orange
            
            // Interactive elements removed during cleanup
        }
    }
    
    /// Dark theme colors based on Zed's dark theme
    pub fn dark() -> Self {
        Self {
            // Dark theme backgrounds  
            background: hsla(0.0, 0.0, 0.1, 1.0),         // #191918
            surface: hsla(0.0, 0.0, 0.13, 1.0),           // #222221
            elevated_surface: hsla(0.0, 0.0, 0.16, 1.0),  // #2a2a28
            
            // Dark theme text
            text: hsla(0.0, 0.0, 0.99, 1.0),              // #fcfcfc
            text_muted: hsla(0.0, 0.0, 0.71, 1.0),        // #b5b4b3
            text_accent: hsla(210.0 / 360.0, 1.0, 0.62, 1.0),     // #3b9eff
            
            // Dark theme borders
            border: hsla(0.0, 0.0, 0.23, 1.0),            // #3a3a37
            
            // Status colors (adjusted for dark mode)
            success: hsla(145.0 / 360.0, 0.53, 0.47, 1.0),        // #33b074 - green
            
            // Metric colors for dashboard cards (adjusted for dark mode)
            metric_primary: hsla(210.0 / 360.0, 1.0, 0.62, 1.0),    // Lighter blue
            metric_secondary: hsla(145.0 / 360.0, 0.53, 0.47, 1.0), // Lighter green
            metric_tertiary: hsla(260.0 / 360.0, 0.6, 0.65, 1.0),   // Lighter purple
            metric_quaternary: hsla(35.0 / 360.0, 0.91, 0.6, 1.0),  // Lighter orange
            
            // Interactive elements removed during cleanup
        }
    }
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self::light()
    }
}