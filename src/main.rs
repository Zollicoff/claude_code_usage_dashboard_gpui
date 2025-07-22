mod analytics;
mod app;
mod theme;
mod ui;
mod utils;

use gpui::*;
use app::views::root::RootView;

fn main() {
    println!("🎯 Initializing Claude Code Usage Dashboard (GPUI)...");
    
    // Initialize GPUI application
    Application::new().run(|cx: &mut App| {
        println!("🚀 Creating dashboard window...");
        
        // Set up window bounds - larger size for dashboard
        let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);
        
        // Create the main window with proper window management
        let _window = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Claude Code Usage Dashboard".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                // Set window to be focusable and able to become key window
                is_movable: true,
                ..Default::default()
            },
            |_window, cx| {
                cx.new(|cx| RootView::new(cx))
            }
        )
        .unwrap();
        
        // Handle application quit properly
        let _quit_subscription = cx.on_app_quit(|_| async move {
            println!("🔴 Application quit requested");
        });
        
        // Activate the application
        cx.activate(true);
        
        println!("✅ Claude Code Usage Dashboard started successfully!");
    });
}