mod analytics;
mod app;
mod theme;
mod ui;
mod utils;

use gpui::*;
use app::views::root::RootView;

fn main() {
    println!("🎯 Initializing Usage Dashboard (GPUI)...");
    
    // Initialize GPUI application
    Application::new().run(|cx: &mut App| {
        println!("🚀 Creating dashboard window...");
        
        // Set up window bounds - larger size for dashboard
        let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);
        
        // Create the main window
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Usage Dashboard".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                ..Default::default()
            },
            |_window, cx| {
                cx.new(|cx| RootView::new(cx))
            }
        )
        .unwrap();
        
        // Activate the application
        cx.activate(true);
        
        println!("✅ Usage Dashboard started successfully!");
    });
}