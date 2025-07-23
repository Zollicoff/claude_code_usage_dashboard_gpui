mod analytics;
mod app;
mod theme;
mod ui;
mod utils;

use gpui::*;
use app::views::root::RootView;

fn main() {
    println!("🎯 Initializing Claude Code Usage Dashboard (GPUI)...");
    
    // Initialize GPUI application with proper quit behavior
    let app = Application::new();
    app.run(|cx: &mut App| {
        println!("🚀 Creating dashboard window...");
        
        // Set up window bounds - larger size for dashboard
        let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);
        
        // Create the main window with proper window management
        let window_handle = cx.open_window(
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
        
        // Store window handle for potential reuse
        let _window_id = window_handle.window_id();
        
        // Handle application quit properly - quit when last window closes
        let _quit_subscription = cx.on_app_quit(|_| async move {
            println!("🔴 Application quit requested");
        });
        
        // Handle window close events - let the window close normally without quitting
        // This allows the app to stay in the dock (standard macOS behavior)
        let _window_close_subscription = cx.on_window_closed(|_closed_window_id| {
            println!("🪟 Window closed - app remains in dock");
            // Don't force quit - let macOS handle the app lifecycle
        });
        
        // Activate the application
        cx.activate(true);
        
        // Simple approach: Handle Cmd+Q and window close at the OS level
        
        println!("✅ Claude Code Usage Dashboard started successfully!");
    });
}