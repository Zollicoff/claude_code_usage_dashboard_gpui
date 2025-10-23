mod models;
mod api;
mod filters;
mod db;
mod config;
mod integration;
mod dashboard;

use gpui::*;
use log::info;

use config::{AppConfig, default_config_path};
use dashboard::DashboardView;

fn main() {
    env_logger::init();

    info!("🚀 Starting PropStream to Pipedrive Integration");

    // Initialize GPUI application
    let app = Application::new();

    app.run(|cx: &mut App| {
        info!("Initializing application window");

        // Load configuration
        let config_path = default_config_path();
        let config = match AppConfig::load(&config_path) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Failed to load config: {}. Using environment variables.", e);
                match AppConfig::from_env() {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        eprintln!("Failed to load config from environment: {}", e);
                        AppConfig::default()
                    }
                }
            }
        };

        info!("Configuration loaded");

        // Create window bounds
        let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);

        // Create main window
        let _window_handle = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("PropStream → Pipedrive Integration".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                is_movable: true,
                ..Default::default()
            },
            |_window, cx| {
                cx.new(|cx| DashboardView::new(cx, config))
            }
        )
        .unwrap();

        // Activate the application
        cx.activate(true);

        info!("✅ Application started successfully");
    });
}