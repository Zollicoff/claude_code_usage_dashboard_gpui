// Theme system inspired by Zed editor
// Provides light/dark mode support with GPUI color system

pub mod colors;
pub mod registry;
pub mod settings;

pub use registry::*;
// Unused wildcard exports removed during cleanup