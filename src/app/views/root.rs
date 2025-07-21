// Root View - Main application window view
// This is the top-level view that contains the entire application

use gpui::*;
// Unused FluentBuilder import removed
use crate::app::actions::DashboardTab;
// Simple loading state enum for root view
#[derive(Debug, Clone)]
pub enum LoadingState {
    LoadingInitial,
    LoadedFull,
    Error(String),
}
use crate::analytics::{UsageStats, ModelStats, ProjectStats, SessionStats, DailyUsage};
use crate::analytics::models::TimeRange;
use crate::analytics::processor::UsageProcessor;
use crate::analytics::aggregator::UsageAggregator;
use crate::theme::ThemeRegistry;
use std::collections::HashMap;
use std::sync::Arc;
// Unused chrono imports removed

#[derive(Debug, Clone, Copy)]
enum MetricType {
    Primary,
    Secondary,
    Tertiary,
    Quaternary,
}

#[derive(Debug, Clone)]
struct MonthlyUsage {
    month: String,
    total_cost: f64,
    total_tokens: u64,
    request_count: usize,
    days_count: usize,
}

// GPUI scrolling implementation using built-in overflow_scroll method

pub struct RootView {
    focus_handle: FocusHandle,
    active_tab: DashboardTab,
    loading_message: String,
    analytics_data: Option<Arc<UsageStats>>,
    full_analytics_data: Option<Arc<UsageStats>>, // Cache full unfiltered data
    loading_state: LoadingState,
    is_loading: bool,
    theme_registry: ThemeRegistry,
    current_time_range: TimeRange,
}

impl RootView {
    pub fn set_active_tab(&mut self, tab: DashboardTab, cx: &mut Context<Self>) {
        if self.active_tab != tab {
            println!("🔄 Switching to tab: {:?}", tab);
            self.active_tab = tab;
            cx.notify();
        }
    }

    pub fn toggle_theme(&mut self, cx: &mut Context<Self>) {
        if let Err(e) = self.theme_registry.toggle_mode() {
            println!("⚠️ Failed to toggle theme: {}", e);
        } else {
            println!("🎨 Theme toggled to: {:?}", self.theme_registry.mode());
            cx.notify();
        }
    }
    
    // Fast filtering method that works on cached data
    fn apply_time_filter(&mut self) {
        if let Some(ref full_data) = self.full_analytics_data {
            let start = std::time::Instant::now();
            
            // Filter entries based on time range
            let aggregator = UsageAggregator::new();
            let filtered_entries = aggregator.filter_by_time_range(&full_data.entries, self.current_time_range);
            
            // For now, recalculate stats from filtered entries
            // TODO: In future, we could pre-calculate stats for each time range
            let filtered_stats = aggregator.calculate_usage_stats(&filtered_entries);
            
            self.analytics_data = Some(Arc::new(filtered_stats));
            
            let elapsed = start.elapsed();
            println!("⚡ Time filter applied in {:?}", elapsed);
        }
    }

    fn render_time_range_filter(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme_registry.colors();
        
        div()
            .flex()
            .items_center()
            .gap_1()
            .p_1()
            .bg(theme.surface)
            .border_1()
            .border_color(theme.border)
            .rounded_md()
            .child(self.render_time_range_button("All Time", TimeRange::AllTime, cx))
            .child(self.render_time_range_button("30D", TimeRange::Last30Days, cx))
            .child(self.render_time_range_button("7D", TimeRange::Last7Days, cx))
    }
    
    fn render_time_range_button(&self, label: &str, range: TimeRange, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme_registry.colors();
        let is_active = self.current_time_range == range;
        let label_string = label.to_string();
        let elevated_surface = theme.elevated_surface;
        
        div()
            .px_3()
            .py_1()
            .text_xs()
            .font_weight(if is_active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
            .text_color(if is_active { theme.text } else { theme.text_muted })
            .bg(if is_active { theme.text_accent } else { theme.surface })
            .border_1()
            .border_color(if is_active { theme.text_accent } else { theme.border })
            .rounded_sm()
            .cursor_pointer()
            .hover(move |style| if !is_active { style.bg(elevated_surface) } else { style })
            .on_mouse_down(MouseButton::Left, cx.listener(move |view: &mut RootView, _event, _window, cx| {
                view.set_time_range(range, cx);
            }))
            .child(label_string)
    }
    
    pub fn set_time_range(&mut self, range: TimeRange, cx: &mut Context<Self>) {
        println!("🎯 set_time_range called: current={:?}, new={:?}", self.current_time_range, range);
        if self.current_time_range != range {
            println!("🔄 Switching to time range: {:?}", range);
            self.current_time_range = range;
            self.apply_time_filter(); // Use fast filtering instead of full reload
            cx.notify();
        } else {
            println!("⚠️ Time range is already set to {:?}, skipping", range);
        }
    }

    fn render_theme_toggle(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.theme_registry.colors();
        let is_dark = self.theme_registry.is_dark();
        let elevated_surface = colors.elevated_surface;
        let border_color = colors.border;
        
        div()
            .id("theme-toggle")
            .flex()
            .items_center()
            .justify_center()
            .w(px(40.0))
            .h(px(32.0))
            .bg(colors.surface)
            .border_1()
            .border_color(colors.border)
            .rounded(px(6.0))
            .cursor_pointer()
            .hover(move |style| style.bg(elevated_surface))
            .active(move |style| style.bg(border_color))
            .on_mouse_down(MouseButton::Left, cx.listener(|view: &mut RootView, _event, _window, cx| {
                view.toggle_theme(cx);
            }))
            .child(
                div()
                    .text_size(px(14.0))
                    .text_color(colors.text)
                    .child(if is_dark { "🌙" } else { "☀️" })
            )
    }
    
    
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut view = Self {
            focus_handle: cx.focus_handle(),
            active_tab: DashboardTab::Overview,
            loading_message: "Loading analytics data...".to_string(),
            analytics_data: None,
            full_analytics_data: None,
            loading_state: LoadingState::LoadingInitial,
            is_loading: true,
            theme_registry: ThemeRegistry::new(),
            current_time_range: TimeRange::Last30Days,
        };
        
        // Focus will be handled by the window system when the view is rendered
        
        // Load data synchronously on initialization
        view.load_data_synchronously();
        view
    }
    
    fn load_data_synchronously(&mut self) {
        println!("🔄 Starting synchronous analytics data loading...");
        
        // Load full data once
        match Self::load_analytics_data_sync() {
            Ok(stats) => {
                println!("✅ Real analytics data loaded successfully with {} entries", stats.entries.len());
                self.full_analytics_data = Some(Arc::new(stats));
                // Apply initial filter
                self.apply_time_filter();
                self.loading_state = LoadingState::LoadedFull;
                self.loading_message = "Dashboard ready - real data loaded".to_string();
                self.is_loading = false;
            }
            Err(e) => {
                println!("⚠️ Failed to load real data: {}, using sample data", e);
                self.loading_state = LoadingState::LoadedFull;
                self.loading_message = "Dashboard ready - using sample data".to_string();
                self.is_loading = false;
                // analytics_data remains None, will use sample data
            }
        }
    }
    
    fn load_analytics_data_sync() -> anyhow::Result<UsageStats> {
        // Use the existing analytics processor
        let processor = UsageProcessor::new()?;
        let entries = processor.process_all_files()?;
        
        println!("📊 Processing {} usage entries...", entries.len());
        
        let aggregator = UsageAggregator::new();
        let stats = aggregator.aggregate_entries(entries);
        
        println!("✅ Analytics computation complete");
        Ok(stats)
    }
    
    fn reload_data_with_time_range(&mut self, cx: &mut Context<Self>) {
        println!("🔄 reload_data_with_time_range called with: {:?}", self.current_time_range);
        
        // Set loading state
        self.is_loading = true;
        self.loading_message = format!("Filtering data for {:?}...", self.current_time_range);
        cx.notify();
        
        // Attempt to load and filter real data
        match Self::load_analytics_data_sync_with_filter(self.current_time_range) {
            Ok(stats) => {
                println!("✅ Filtered analytics data loaded successfully");
                println!("📊 Stats - Total cost: ${:.2}, Total tokens: {}, Model count: {}, Project count: {}", 
                    stats.total_cost, stats.total_tokens, stats.model_stats.len(), stats.project_stats.len());
                self.analytics_data = Some(Arc::new(stats));
                self.loading_state = LoadingState::LoadedFull;
                self.loading_message = "Dashboard ready - real data loaded".to_string();
                self.is_loading = false;
            }
            Err(e) => {
                println!("⚠️ Failed to load filtered data: {}, using sample data", e);
                self.loading_state = LoadingState::LoadedFull;
                self.loading_message = "Dashboard ready - using sample data".to_string();
                self.is_loading = false;
                // analytics_data remains None, will use sample data
            }
        }
        
        cx.notify();
    }
    
    fn load_analytics_data_sync_with_filter(time_range: TimeRange) -> anyhow::Result<UsageStats> {
        // Use the existing analytics processor
        let processor = UsageProcessor::new()?;
        let entries = processor.process_all_files()?;
        
        println!("📊 Processing {} usage entries with filter {:?}...", entries.len(), time_range);
        
        // Debug: Show date range of entries
        if !entries.is_empty() {
            let min_date = entries.iter().map(|e| e.timestamp).min().unwrap();
            let max_date = entries.iter().map(|e| e.timestamp).max().unwrap();
            println!("📅 Entry date range: {} to {}", min_date.format("%Y-%m-%d"), max_date.format("%Y-%m-%d"));
        }
        
        let aggregator = UsageAggregator::new();
        
        // Filter entries by time range
        let filtered_entries = aggregator.filter_by_time_range(&entries, time_range);
        println!("📊 Filtered to {} entries (from {} total)", filtered_entries.len(), entries.len());
        
        if !filtered_entries.is_empty() {
            let filtered_min = filtered_entries.iter().map(|e| e.timestamp).min().unwrap();
            let filtered_max = filtered_entries.iter().map(|e| e.timestamp).max().unwrap();
            println!("📅 Filtered date range: {} to {}", filtered_min.format("%Y-%m-%d"), filtered_max.format("%Y-%m-%d"));
        }
        
        let stats = aggregator.aggregate_entries(filtered_entries);
        
        println!("✅ Filtered analytics computation complete");
        Ok(stats)
    }
    
    /// Get analytics data - real data if loaded, sample data as fallback
    fn get_analytics_data(&self) -> UsageStats {
        if let Some(ref real_data) = self.analytics_data {
            println!("🔍 Using real analytics data: ${:.2} total, {} tokens, {} models, {} projects", 
                real_data.total_cost, real_data.total_tokens, real_data.model_stats.len(), real_data.project_stats.len());
            (**real_data).clone()
        } else {
            println!("⚠️ Falling back to sample data - real data not loaded");
            self.get_sample_analytics()
        }
    }
    
    /// Generate sample analytics data for demonstration (fallback when real data not loaded)
    fn get_sample_analytics(&self) -> UsageStats {
        let mut model_stats = HashMap::new();
        
        // Claude 3.5 Sonnet
        model_stats.insert("claude-3-5-sonnet-20241022".to_string(), ModelStats {
            model: "claude-3-5-sonnet-20241022".to_string(),
            display_name: "Claude 3.5 Sonnet".to_string(),
            total_cost: 12.45,
            total_tokens: 125000,
            input_tokens: 75000,
            output_tokens: 35000,
            cache_read_tokens: 10000,
            cache_creation_tokens: 5000,
            request_count: 156,
        });
        
        // Claude 3 Opus
        model_stats.insert("claude-3-opus-20240229".to_string(), ModelStats {
            model: "claude-3-opus-20240229".to_string(),
            display_name: "Claude 3 Opus".to_string(),
            total_cost: 8.72,
            total_tokens: 42000,
            input_tokens: 28000,
            output_tokens: 12000,
            cache_read_tokens: 1500,
            cache_creation_tokens: 500,
            request_count: 47,
        });
        
        // Claude 3 Haiku
        model_stats.insert("claude-3-haiku-20240307".to_string(), ModelStats {
            model: "claude-3-haiku-20240307".to_string(),
            display_name: "Claude 3 Haiku".to_string(),
            total_cost: 2.31,
            total_tokens: 89000,
            input_tokens: 65000,
            output_tokens: 20000,
            cache_read_tokens: 3000,
            cache_creation_tokens: 1000,
            request_count: 203,
        });
        
        let mut project_stats = HashMap::new();
        project_stats.insert("/Users/dev/rust-project".to_string(), ProjectStats {
            project_name: "rust-project".to_string(),
            project_path: "/Users/dev/rust-project".to_string(),
            total_cost: 15.23,
            total_tokens: 145000,
            input_tokens: 95000,
            output_tokens: 35000,
            cache_read_tokens: 10000,
            cache_creation_tokens: 5000,
            request_count: 198,
            session_count: 12,
            last_used: chrono::Utc::now(),
        });
        
        UsageStats {
            total_cost: 23.48,
            total_input_tokens: 168000,
            total_output_tokens: 67000,
            total_cache_read_tokens: 14500,
            total_cache_creation_tokens: 6500,
            total_tokens: 256000,
            session_count: 15,
            entries: vec![], // Empty for demo
            model_stats,
            project_stats,
            session_stats: HashMap::new(),
            daily_usage: HashMap::new(),
        }
    }
    
    /// Get sessions data - real data if loaded, sample data as fallback
    fn get_sessions_data(&self) -> Vec<SessionStats> {
        if let Some(ref real_data) = self.analytics_data {
            // Extract sessions from real analytics data
            real_data.session_stats.values()
                .cloned()
                .collect::<Vec<_>>()
        } else {
            self.get_sample_sessions_analytics()
        }
    }
    
    /// Generate sample session analytics data for demonstration
    fn get_sample_sessions_analytics(&self) -> Vec<SessionStats> {
        use chrono::{Utc, Duration};
        
        vec![
            SessionStats {
                session_id: "session_2024071201".to_string(),
                project_path: "/Users/dev/rust-project".to_string(),
                total_cost: 8.45,
                total_tokens: 89000,
                input_tokens: 65000,
                output_tokens: 18000,
                cache_read_tokens: 4000,
                cache_creation_tokens: 2000,
                request_count: 87,
                timestamp: Utc::now() - Duration::hours(2),
            },
            SessionStats {
                session_id: "session_2024071901".to_string(),
                project_path: "/Users/dev/web-app".to_string(),
                total_cost: 6.72,
                total_tokens: 76000,
                input_tokens: 52000,
                output_tokens: 16000,
                cache_read_tokens: 5000,
                cache_creation_tokens: 3000,
                request_count: 65,
                timestamp: Utc::now() - Duration::days(1) - Duration::hours(4),
            },
            SessionStats {
                session_id: "session_2024071902".to_string(),
                project_path: "/Users/dev/rust-project".to_string(),
                total_cost: 4.23,
                total_tokens: 45000,
                input_tokens: 32000,
                output_tokens: 9000,
                cache_read_tokens: 3000,
                cache_creation_tokens: 1000,
                request_count: 34,
                timestamp: Utc::now() - Duration::days(1) - Duration::hours(8),
            },
            SessionStats {
                session_id: "session_2024071801".to_string(),
                project_path: "/Users/dev/python-tool".to_string(),
                total_cost: 3.08,
                total_tokens: 38000,
                input_tokens: 28000,
                output_tokens: 7000,
                cache_read_tokens: 2000,
                cache_creation_tokens: 1000,
                request_count: 23,
                timestamp: Utc::now() - Duration::days(2) - Duration::hours(6),
            },
            SessionStats {
                session_id: "session_2024071701".to_string(),
                project_path: "/Users/dev/web-app".to_string(),
                total_cost: 1.25,
                total_tokens: 15000,
                input_tokens: 11000,
                output_tokens: 3000,
                cache_read_tokens: 800,
                cache_creation_tokens: 200,
                request_count: 12,
                timestamp: Utc::now() - Duration::days(3) - Duration::hours(10),
            },
        ]
    }
    
    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme_registry.colors();
        
        div()
            .flex()
            .justify_between()
            .items_center()
            .bg(theme.elevated_surface)
            .border_b_1()
            .border_color(theme.border)
            .px_6()
            .py_4()
            .child(
                // Title
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text_accent)
                    .child("Claude Usage Dashboard")
            )
            .child(
                // Right side: Status indicator and theme toggle
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(
                        // Status indicator
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.text_muted)
                                    .child(self.loading_message.clone())
                            )
                            .child(
                                // Show current time range and entry count
                                if let Some(ref data) = self.analytics_data {
                                    div()
                                        .text_xs()
                                        .text_color(theme.text_muted)
                                        .child(format!(" | {:?}: {} entries", self.current_time_range, data.entries.len()))
                                } else {
                                    div()
                                }
                            )
                            .child(
                                // Status dot
                                div()
                                    .w_3()  // Make it slightly bigger
                                    .h_3()  // Make it slightly bigger
                                    .rounded_full()
                                    .bg({
                                        let color = if !self.is_loading && self.analytics_data.is_some() {
                                            println!("🟢 Status dot: GREEN (data loaded) - color: {:?}", theme.success);
                                            theme.success // Green when data is loaded
                                        } else if self.is_loading {
                                            println!("🔵 Status dot: BLUE (loading) - color: {:?}", theme.text_accent);
                                            theme.text_accent // Blue during loading
                                        } else {
                                            println!("⚪ Status dot: GRAY (no data) - color: {:?}", theme.text_muted);
                                            theme.text_muted // Gray when no data
                                        };
                                        println!("   is_loading: {}, analytics_data: {}", 
                                            self.is_loading, 
                                            self.analytics_data.is_some());
                                        color
                                    })
                            )
                    )
                    .child(
                        // Time range filter buttons
                        self.render_time_range_filter(cx)
                    )
                    .child(
                        // Theme toggle button
                        self.render_theme_toggle(cx)
                    )
            )
    }
    
    fn render_tab_navigation(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme_registry.colors();
        
        div()
            .flex()
            .justify_between()
            .bg(theme.surface)
            .border_b_1()
            .border_color(theme.border)
            .px_6()
            .child(
                div()
                    .flex()
                    .children(
                        DashboardTab::all().into_iter().enumerate().map(|(index, tab)| {
                            let is_active = self.active_tab == tab;
                            let key_number = index + 1;
                            let tab_clone = tab.clone();
                            let text_accent = theme.text_accent;
                            
                            div()
                                .px_4()
                                .py_3()
                                .cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |view, _event, _window, cx| {
                                    view.set_active_tab(tab_clone.clone(), cx);
                                }))
                                .border_b_2()
                                .border_color(if is_active {
                                    theme.text_accent
                                } else {
                                    hsla(0.0, 0.0, 0.0, 0.0) // Transparent
                                })
                                .text_color(if is_active {
                                    theme.text_accent
                                } else {
                                    theme.text_muted
                                })
                                .font_weight(if is_active {
                                    FontWeight::SEMIBOLD
                                } else {
                                    FontWeight::NORMAL
                                })
                                .hover(|style| {
                                    style.text_color(text_accent)
                                })
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(format!("{}", key_number))
                                        .child(tab.title())
                                )
                        })
                    )
            )
            .child(
                div()
                    .py_3()
                    .text_xs()
                    .text_color(theme.text_muted)
                    .child("Press 1-5 to switch tabs • Alt+1/2/3 for time ranges")
            )
    }
    
    fn render_main_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("main-content")
            .flex_1()
            .h_full()
            .overflow_scroll()
            .p_6()
            .child(
                if self.is_loading {
                    self.render_loading_content()
                } else {
                    self.render_active_tab_content(cx)
                }
            )
    }
    
    fn render_loading_content(&self) -> Div {
        let theme = self.theme_registry.colors();
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .h_96()
            .gap_6()
            .child(
                // Loading spinner placeholder (would be an actual spinner in real UI)
                div()
                    .w_8()
                    .h_8()
                    .border_2()
                    .border_color(theme.text_accent)
                    .rounded_full()
                    // Note: GPUI doesn't have built-in animations, but this represents a spinner
            )
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text)
                    .child(self.loading_message.clone())
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.text_muted)
                    .child(match &self.loading_state {
                        LoadingState::LoadingInitial => "Scanning Claude usage files...".to_string(),
                        LoadingState::LoadedFull => "Processing usage data...".to_string(),
                        LoadingState::Error(e) => format!("Error: {}", e),
                    })
            )
    }
    
    fn render_active_tab_content(&self, _cx: &mut Context<Self>) -> Div {
        match &self.active_tab {
            DashboardTab::Overview => self.render_overview_content(),
            DashboardTab::Models => self.render_models_content(),
            DashboardTab::Projects => self.render_projects_content(),
            DashboardTab::Sessions => self.render_sessions_content(),
            DashboardTab::Timeline => self.render_timeline_content(),
        }
    }
    
    fn render_overview_content(&self) -> Div {
        let theme = self.theme_registry.colors();
        let analytics = self.get_analytics_data();
        
        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .text_3xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text)
                    .child("Usage Overview")
            )
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(
                        self.render_metric_card(
                            "Total Cost", 
                            format!("${:.2}", analytics.total_cost), 
                            MetricType::Primary
                        )
                    )
                    .child(
                        self.render_metric_card(
                            "Total Tokens", 
                            self.format_number(analytics.total_tokens), 
                            MetricType::Secondary
                        )
                    )
                    .child(
                        self.render_metric_card(
                            "Sessions", 
                            analytics.session_count.to_string(), 
                            MetricType::Tertiary
                        )
                    )
                    .child(
                        self.render_metric_card(
                            "Models Used", 
                            analytics.model_stats.len().to_string(), 
                            MetricType::Quaternary
                        )
                    )
            )
            .child(self.render_breakdown_section(&analytics))
    }
    
    fn render_breakdown_section(&self, analytics: &UsageStats) -> Div {
        div()
            .mt_8()
            .flex()
            .gap_6()
            .child(self.render_model_breakdown(analytics))
            .child(self.render_cost_breakdown(analytics))
    }
    
    fn render_model_breakdown(&self, analytics: &UsageStats) -> Div {
        let theme = self.theme_registry.colors();
        div()
            .flex_1()
            .p_6()
            .bg(theme.surface)
            .rounded_lg()
            .border_1()
            .border_color(theme.border)
            .shadow_sm()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text)
                    .mb_4()
                    .child("Usage by Model")
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .children(
                        analytics.model_stats.values()
                            .map(|model| self.render_model_item(model, analytics.total_cost))
                            .collect::<Vec<_>>()
                    )
            )
    }
    
    fn render_model_item(&self, model: &ModelStats, total_cost: f64) -> Div {
        let theme = self.theme_registry.colors();
        let percentage = if total_cost > 0.0 {
            (model.total_cost / total_cost * 100.0) as u32
        } else {
            0
        };
        
        div()
            .flex()
            .justify_between()
            .items_center()
            .p_3()
            .bg(theme.elevated_surface)
            .rounded_md()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text)
                            .child(model.display_name.clone())
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child(format!("{} requests", model.request_count))
                    )
            )
            .child(
                div()
                    .text_right()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.success)
                            .child(format!("${:.2}", model.total_cost))
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child(format!("{}%", percentage))
                    )
            )
    }
    
    fn render_cost_breakdown(&self, analytics: &UsageStats) -> Div {
        let theme = self.theme_registry.colors();
        div()
            .flex_1()
            .p_6()
            .bg(theme.surface)
            .rounded_lg()
            .border_1()
            .border_color(theme.border)
            .shadow_sm()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text)
                    .mb_4()
                    .child("Token Usage")
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(self.render_token_breakdown_item("Input Tokens", analytics.total_input_tokens, theme.metric_primary, analytics.total_tokens))
                    .child(self.render_token_breakdown_item("Output Tokens", analytics.total_output_tokens, theme.metric_secondary, analytics.total_tokens))
                    .child(self.render_token_breakdown_item("Cache Read", analytics.total_cache_read_tokens, theme.metric_tertiary, analytics.total_tokens))
                    .child(self.render_token_breakdown_item("Cache Creation", analytics.total_cache_creation_tokens, theme.metric_quaternary, analytics.total_tokens))
            )
    }
    
    fn render_token_breakdown_item(&self, label: &str, count: u64, color: Hsla, total_tokens: u64) -> Div {
        let theme = self.theme_registry.colors();
        let label_string = label.to_string();
        let percentage = if total_tokens > 0 {
            (count as f64 / total_tokens as f64 * 100.0) as u32
        } else {
            0
        };
        
        div()
            .flex()
            .justify_between()
            .items_center()
            .p_3()
            .bg(theme.elevated_surface)
            .rounded_md()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .w_3()
                            .h_3()
                            .bg(color)
                            .rounded_full()
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text)
                            .child(label_string)
                    )
            )
            .child(
                div()
                    .text_right()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(self.format_number(count))
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child(format!("{}%", percentage))
                    )
            )
    }
    
    fn format_number(&self, num: u64) -> String {
        if num >= 1_000_000 {
            format!("{:.1}M", num as f64 / 1_000_000.0)
        } else if num >= 1_000 {
            format!("{:.1}K", num as f64 / 1_000.0)
        } else {
            num.to_string()
        }
    }
    
    fn render_models_content(&self) -> Div {
        let theme = self.theme_registry.colors();
        let analytics = self.get_analytics_data();
        
        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .text_3xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text)
                    .child("Model Analytics")
            )
            .child(self.render_models_summary(&analytics))
            .child(self.render_models_detailed_list(&analytics))
    }
    
    fn render_models_summary(&self, analytics: &UsageStats) -> Div {
        div()
            .flex()
            .gap_4()
            .child(
                self.render_metric_card(
                    "Total Models", 
                    analytics.model_stats.len().to_string(), 
                    MetricType::Primary
                )
            )
            .child(
                self.render_metric_card(
                    "Most Used", 
                    analytics.model_stats.values()
                        .max_by_key(|m| m.request_count)
                        .map(|m| m.display_name.clone())
                        .unwrap_or("No data".to_string()), 
                    MetricType::Secondary
                )
            )
            .child(
                self.render_metric_card(
                    "Total Requests", 
                    analytics.model_stats.values()
                        .map(|m| m.request_count)
                        .sum::<usize>()
                        .to_string(), 
                    MetricType::Tertiary
                )
            )
            .child(
                self.render_metric_card(
                    "Avg Cost/Request", 
                    format!("${:.3}", analytics.total_cost / analytics.model_stats.values().map(|m| m.request_count).sum::<usize>() as f64), 
                    MetricType::Quaternary
                )
            )
    }
    
    fn render_models_detailed_list(&self, analytics: &UsageStats) -> Div {
        let theme = self.theme_registry.colors();
        div()
            .p_6()
            .bg(theme.surface)
            .rounded_lg()
            .border_1()
            .border_color(theme.border)
            .shadow_sm()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text)
                    .mb_6()
                    .child("Detailed Model Breakdown")
            )
            .child(
                div()
                    .id("models-list")
                    .flex()
                    .flex_col()
                    .gap_4()
                    .h(px(400.0))
                    .overflow_scroll()
                    .children(
                        analytics.model_stats.values()
                            .map(|model| self.render_detailed_model_card(model))
                            .collect::<Vec<_>>()
                    )
            )
    }
    
    fn render_detailed_model_card(&self, model: &ModelStats) -> Div {
        let theme = self.theme_registry.colors();
        div()
            .p_6()
            .bg(theme.elevated_surface)
            .border_1()
            .border_color(theme.border)
            .rounded_lg()
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_start()
                    .mb_4()
                    .child(
                        div()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.text)
                                    .child(model.display_name.clone())
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.text_muted)
                                    .child(model.model.clone())
                            )
                    )
                    .child(
                        div()
                            .text_right()
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.success)
                                    .child(format!("${:.2}", model.total_cost))
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.text_muted)
                                    .child(format!("{} requests", model.request_count))
                            )
                    )
            )
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(self.render_token_stat("Input", model.input_tokens, theme.metric_primary))
                    .child(self.render_token_stat("Output", model.output_tokens, theme.metric_secondary))
                    .child(self.render_token_stat("Cache Read", model.cache_read_tokens, theme.metric_tertiary))
                    .child(self.render_token_stat("Cache Creation", model.cache_creation_tokens, theme.metric_quaternary))
            )
    }
    
    fn render_token_stat(&self, label: &str, count: u64, color: Hsla) -> Div {
        let theme = self.theme_registry.colors();
        let label_string = label.to_string();
        div()
            .flex()
            .items_center()
            .gap_3()
            .p_3()
            .bg(theme.surface)
            .rounded_md()
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .w_4()
                    .h_4()
                    .bg(color)
                    .rounded_full()
            )
            .child(
                div()
                    .flex_1()
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.text_muted)
                            .child(label_string)
                    )
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(self.format_number(count))
                    )
            )
    }
    
    fn render_projects_content(&self) -> Div {
        let theme = self.theme_registry.colors();
        let analytics = self.get_analytics_data();
        
        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .text_3xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text)
                    .child("Project Analytics")
            )
            .child(self.render_projects_summary(&analytics))
            .child(self.render_projects_list(&analytics))
    }
    
    fn render_projects_summary(&self, analytics: &UsageStats) -> Div {
        div()
            .flex()
            .gap_4()
            .child(
                self.render_metric_card(
                    "Active Projects", 
                    analytics.project_stats.len().to_string(), 
                    MetricType::Primary
                )
            )
            .child(
                self.render_metric_card(
                    "Most Active", 
                    analytics.project_stats.values()
                        .max_by_key(|p| p.request_count)
                        .map(|p| p.project_name.clone())
                        .unwrap_or("No data".to_string()), 
                    MetricType::Secondary
                )
            )
            .child(
                self.render_metric_card(
                    "Total Sessions", 
                    analytics.project_stats.values()
                        .map(|p| p.session_count)
                        .sum::<usize>()
                        .to_string(), 
                    MetricType::Tertiary
                )
            )
            .child(
                self.render_metric_card(
                    "Avg Cost/Project", 
                    format!("${:.2}", analytics.total_cost / analytics.project_stats.len() as f64), 
                    MetricType::Quaternary
                )
            )
    }
    
    fn render_projects_list(&self, analytics: &UsageStats) -> Div {
        let theme = self.theme_registry.colors();
        div()
            .p_6()
            .bg(theme.surface)
            .rounded_lg()
            .border_1()
            .border_color(theme.border)
            .shadow_sm()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text)
                    .mb_6()
                    .child("Project Breakdown")
            )
            .child(
                div()
                    .id("projects-list")
                    .flex()
                    .flex_col()
                    .gap_4()
                    .max_h(px(500.0))
                    .overflow_scroll()
                    .children(
                        analytics.project_stats.values()
                            .map(|project| self.render_project_card(project))
                            .collect::<Vec<_>>()
                    )
            )
    }
    
    fn render_project_card(&self, project: &ProjectStats) -> Div {
        let theme = self.theme_registry.colors();
        div()
            .p_6()
            .bg(theme.elevated_surface)
            .border_1()
            .border_color(theme.border)
            .rounded_lg()
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_start()
                    .mb_4()
                    .child(
                        div()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.text)
                                    .child(project.project_name.clone())
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.text_muted)
                                    .child(project.project_path.clone())
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .child(format!("Last used: {}", project.last_used.format("%Y-%m-%d %H:%M")))
                            )
                    )
                    .child(
                        div()
                            .text_right()
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.success)
                                    .child(format!("${:.2}", project.total_cost))
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.text_muted)
                                    .child(format!("{} sessions", project.session_count))
                            )
                    )
            )
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(self.render_project_stat("Requests", project.request_count.to_string(), theme.metric_primary))
                    .child(self.render_project_stat("Total Tokens", self.format_number(project.total_tokens), theme.metric_secondary))
                    .child(self.render_project_stat("Input", self.format_number(project.input_tokens), theme.metric_tertiary))
                    .child(self.render_project_stat("Output", self.format_number(project.output_tokens), theme.metric_quaternary))
            )
    }
    
    fn render_project_stat(&self, label: &str, value: String, color: Hsla) -> Div {
        let theme = self.theme_registry.colors();
        let label_string = label.to_string();
        div()
            .flex()
            .items_center()
            .gap_3()
            .p_3()
            .bg(theme.surface)
            .rounded_md()
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .w_3()
                    .h_3()
                    .bg(color)
                    .rounded_full()
            )
            .child(
                div()
                    .flex_1()
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child(label_string)
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(value)
                    )
            )
    }
    
    fn render_sessions_content(&self) -> Div {
        let theme = self.theme_registry.colors();
        let sessions = self.get_sessions_data();
        
        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .text_3xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text)
                    .child("Session History")
            )
            .child(self.render_sessions_summary(&sessions))
            .child(self.render_sessions_timeline(&sessions))
    }
    
    fn render_sessions_summary(&self, sessions: &[SessionStats]) -> Div {
        let total_sessions = sessions.len();
        let total_cost: f64 = sessions.iter().map(|s| s.total_cost).sum();
        let total_requests: usize = sessions.iter().map(|s| s.request_count).sum();
        let avg_cost_per_session = if total_sessions > 0 { total_cost / total_sessions as f64 } else { 0.0 };
        
        div()
            .flex()
            .gap_4()
            .child(
                self.render_metric_card(
                    "Total Sessions", 
                    total_sessions.to_string(), 
                    MetricType::Primary
                )
            )
            .child(
                self.render_metric_card(
                    "Total Cost", 
                    format!("${:.2}", total_cost), 
                    MetricType::Secondary
                )
            )
            .child(
                self.render_metric_card(
                    "Total Requests", 
                    total_requests.to_string(), 
                    MetricType::Tertiary
                )
            )
            .child(
                self.render_metric_card(
                    "Avg Cost/Session", 
                    format!("${:.2}", avg_cost_per_session), 
                    MetricType::Quaternary
                )
            )
    }
    
    fn render_sessions_timeline(&self, sessions: &[SessionStats]) -> Div {
        let theme = self.theme_registry.colors();
        div()
            .p_6()
            .bg(theme.surface)
            .rounded_lg()
            .border_1()
            .border_color(theme.border)
            .shadow_sm()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text)
                    .mb_6()
                    .child("Recent Sessions Timeline")
            )
            .child(
                div()
                    .id("sessions-list")
                    .flex()
                    .flex_col()
                    .gap_4()
                    .max_h(px(500.0))
                    .overflow_scroll()
                    .children(
                        sessions.iter()
                            .map(|session| self.render_session_timeline_item(session))
                            .collect::<Vec<_>>()
                    )
            )
    }
    
    fn render_session_timeline_item(&self, session: &SessionStats) -> Div {
        let theme = self.theme_registry.colors();
        let project_name = session.project_path
            .split('/')
            .last()
            .unwrap_or("Unknown Project")
            .to_string();
        
        div()
            .flex()
            .items_start()
            .gap_4()
            .p_4()
            .bg(theme.surface)
            .border_1()
            .border_color(theme.border)
            .rounded_lg()
            .child(
                // Timeline dot and line
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .child(
                        div()
                            .w_3()
                            .h_3()
                            .bg(theme.text_accent)
                            .rounded_full()
                            .mt_1()
                    )
            )
            .child(
                // Session content
                div()
                    .flex_1()
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_start()
                            .mb_3()
                            .child(
                                div()
                                    .child(
                                        div()
                                            .text_lg()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(theme.text)
                                            .child(project_name)
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme.text_muted)
                                            .child(format!("Session: {}", &session.session_id[..12]))
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.text_muted)
                                            .child(session.timestamp.format("%Y-%m-%d %H:%M").to_string())
                                    )
                            )
                            .child(
                                div()
                                    .text_right()
                                    .child(
                                        div()
                                            .text_xl()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(theme.success)
                                            .child(format!("${:.2}", session.total_cost))
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme.text_muted)
                                            .child(format!("{} requests", session.request_count))
                                    )
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .gap_3()
                            .child(self.render_session_stat("Total", self.format_number(session.total_tokens), theme.metric_primary))
                            .child(self.render_session_stat("Input", self.format_number(session.input_tokens), theme.metric_secondary))
                            .child(self.render_session_stat("Output", self.format_number(session.output_tokens), theme.metric_tertiary))
                            .child(self.render_session_stat("Cache", self.format_number(session.cache_read_tokens + session.cache_creation_tokens), theme.metric_quaternary))
                    )
            )
    }
    
    fn render_session_stat(&self, label: &str, value: String, color: Hsla) -> Div {
        let theme = self.theme_registry.colors();
        let label_string = label.to_string();
        div()
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .py_2()
            .bg(theme.surface)
            .rounded_md()
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .w_2()
                    .h_2()
                    .bg(color)
                    .rounded_full()
            )
            .child(
                div()
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child(label_string)
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(value)
                    )
            )
    }
    
    fn render_timeline_content(&self) -> Div {
        let theme = self.theme_registry.colors();
        let daily_usage = self.get_daily_usage_data();
        
        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .text_3xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text)
                    .child("Usage Timeline")
            )
            .child(self.render_timeline_summary(&daily_usage))
            .child(self.render_daily_usage_timeline(&daily_usage))
    }
    
    /// Get daily usage data - real data if loaded, sample data as fallback
    fn get_daily_usage_data(&self) -> Vec<DailyUsage> {
        if let Some(ref real_data) = self.analytics_data {
            // Extract daily usage from real analytics data
            real_data.daily_usage.values()
                .cloned()
                .collect::<Vec<_>>()
        } else {
            self.get_sample_daily_usage()
        }
    }
    
    /// Generate sample daily usage data for demonstration
    fn get_sample_daily_usage(&self) -> Vec<DailyUsage> {
        vec![
            DailyUsage {
                date: "2024-07-20".to_string(),
                total_cost: 8.45,
                total_tokens: 89000,
                input_tokens: 65000,
                output_tokens: 18000,
                cache_read_tokens: 4000,
                cache_creation_tokens: 2000,
                request_count: 87,
                models_used: vec![
                    "claude-3-5-sonnet-20241022".to_string(),
                    "claude-3-haiku-20240307".to_string(),
                ],
            },
            DailyUsage {
                date: "2024-07-19".to_string(),
                total_cost: 10.95,
                total_tokens: 121000,
                input_tokens: 84000,
                output_tokens: 25000,
                cache_read_tokens: 8000,
                cache_creation_tokens: 4000,
                request_count: 99,
                models_used: vec![
                    "claude-3-5-sonnet-20241022".to_string(),
                    "claude-3-opus-20240229".to_string(),
                    "claude-3-haiku-20240307".to_string(),
                ],
            },
            DailyUsage {
                date: "2024-07-18".to_string(),
                total_cost: 3.08,
                total_tokens: 38000,
                input_tokens: 28000,
                output_tokens: 7000,
                cache_read_tokens: 2000,
                cache_creation_tokens: 1000,
                request_count: 23,
                models_used: vec![
                    "claude-3-haiku-20240307".to_string(),
                ],
            },
            DailyUsage {
                date: "2024-07-17".to_string(),
                total_cost: 1.25,
                total_tokens: 15000,
                input_tokens: 11000,
                output_tokens: 3000,
                cache_read_tokens: 800,
                cache_creation_tokens: 200,
                request_count: 12,
                models_used: vec![
                    "claude-3-haiku-20240307".to_string(),
                ],
            },
            DailyUsage {
                date: "2024-07-16".to_string(),
                total_cost: 0.0,
                total_tokens: 0,
                input_tokens: 0,
                output_tokens: 0,
                cache_read_tokens: 0,
                cache_creation_tokens: 0,
                request_count: 0,
                models_used: vec![],
            },
            DailyUsage {
                date: "2024-07-15".to_string(),
                total_cost: 5.67,
                total_tokens: 67000,
                input_tokens: 45000,
                output_tokens: 15000,
                cache_read_tokens: 5000,
                cache_creation_tokens: 2000,
                request_count: 45,
                models_used: vec![
                    "claude-3-5-sonnet-20241022".to_string(),
                    "claude-3-opus-20240229".to_string(),
                ],
            },
            DailyUsage {
                date: "2024-07-14".to_string(),
                total_cost: 2.34,
                total_tokens: 28000,
                input_tokens: 20000,
                output_tokens: 6000,
                cache_read_tokens: 1500,
                cache_creation_tokens: 500,
                request_count: 18,
                models_used: vec![
                    "claude-3-haiku-20240307".to_string(),
                ],
            },
        ]
    }
    
    fn render_timeline_summary(&self, daily_usage: &[DailyUsage]) -> Div {
        let total_days = daily_usage.len();
        let active_days = daily_usage.iter().filter(|d| d.request_count > 0).count();
        let total_cost: f64 = daily_usage.iter().map(|d| d.total_cost).sum();
        let avg_daily_cost = if active_days > 0 { total_cost / active_days as f64 } else { 0.0 };
        
        div()
            .flex()
            .gap_4()
            .child(
                self.render_metric_card(
                    "Total Days", 
                    total_days.to_string(), 
                    MetricType::Primary
                )
            )
            .child(
                self.render_metric_card(
                    "Active Days", 
                    active_days.to_string(), 
                    MetricType::Secondary
                )
            )
            .child(
                self.render_metric_card(
                    "Total Cost", 
                    format!("${:.2}", total_cost), 
                    MetricType::Tertiary
                )
            )
            .child(
                self.render_metric_card(
                    "Avg Daily Cost", 
                    format!("${:.2}", avg_daily_cost), 
                    MetricType::Quaternary
                )
            )
    }
    
    fn render_daily_usage_timeline(&self, daily_usage: &[DailyUsage]) -> Div {
        let theme = self.theme_registry.colors();
        
        // Group data by month
        let monthly_data = self.group_daily_usage_by_month(daily_usage);
        
        div()
            .p_6()
            .bg(theme.surface)
            .rounded_lg()
            .border_1()
            .border_color(theme.border)
            .shadow_sm()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text)
                    .mb_6()
                    .child("Usage by Month")
            )
            .child(self.render_monthly_bar_chart(monthly_data))
    }
    
    fn group_daily_usage_by_month(&self, daily_usage: &[DailyUsage]) -> Vec<MonthlyUsage> {
        let mut monthly_map: HashMap<String, MonthlyUsage> = HashMap::new();
        
        for day in daily_usage {
            // Parse date and extract month (assuming format YYYY-MM-DD)
            let month = if day.date.len() >= 7 {
                format!("{}-{}", &day.date[0..4], &day.date[5..7])
            } else {
                day.date.clone()
            };
            
            let entry = monthly_map.entry(month.clone()).or_insert(MonthlyUsage {
                month: month.clone(),
                total_cost: 0.0,
                total_tokens: 0,
                request_count: 0,
                days_count: 0,
            });
            
            entry.total_cost += day.total_cost;
            entry.total_tokens += day.total_tokens;
            entry.request_count += day.request_count;
            entry.days_count += 1;
        }
        
        let mut monthly_data: Vec<_> = monthly_map.into_values().collect();
        monthly_data.sort_by(|a, b| a.month.cmp(&b.month));
        monthly_data
    }
    
    fn render_monthly_bar_chart(&self, monthly_data: Vec<MonthlyUsage>) -> Div {
        let _theme = self.theme_registry.colors();
        let max_cost = monthly_data.iter()
            .map(|m| m.total_cost)
            .fold(0.0f64, |a, b| a.max(b))
            .max(1.0);
        
        div()
            .flex()
            .flex_col()
            .gap_4()
            .children(
                monthly_data.iter()
                    .map(|month| self.render_monthly_bar(month, max_cost))
                    .collect::<Vec<_>>()
            )
    }
    
    fn render_monthly_bar(&self, month: &MonthlyUsage, max_cost: f64) -> Div {
        let theme = self.theme_registry.colors();
        let _percentage = ((month.total_cost / max_cost) * 100.0) as u32;
        let bar_width = (month.total_cost / max_cost * 300.0).max(10.0) as f32;
        
        div()
            .flex()
            .items_center()
            .gap_4()
            .p_4()
            .bg(theme.elevated_surface)
            .border_1()
            .border_color(theme.border)
            .rounded_lg()
            .child(
                // Month label
                div()
                    .w_20()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text)
                    .child(month.month.clone())
            )
            .child(
                // Bar chart area
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .w_full()
                            .h_6()
                            .bg(theme.border)
                            .rounded(px(3.0))
                            .overflow_hidden()
                            .child(
                                div()
                                    .w(px(bar_width))
                                    .h_full()
                                    .bg(theme.metric_primary)
                                    .rounded(px(3.0))
                            )
                    )
            )
            .child(
                // Statistics
                div()
                    .flex()
                    .items_center()
                    .gap_6()
                    .child(
                        div()
                            .text_right()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.success)
                                    .child(format!("${:.2}", month.total_cost))
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .child(format!("{} days", month.days_count))
                            )
                    )
                    .child(
                        div()
                            .text_right()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.text)
                                    .child(self.format_number(month.total_tokens))
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .child(format!("{} requests", month.request_count))
                            )
                    )
            )
    }
    // Removed unused render_daily_usage_bar method during cleanup (replaced by monthly chart)
    
    fn render_metric_card(&self, title: &'static str, value: String, metric_type: MetricType) -> impl IntoElement {
        let theme = self.theme_registry.colors();
        let value_color = match metric_type {
            MetricType::Primary => theme.metric_primary,
            MetricType::Secondary => theme.metric_secondary,
            MetricType::Tertiary => theme.metric_tertiary,
            MetricType::Quaternary => theme.metric_quaternary,
        };
        
        div()
            .bg(theme.surface)
            .rounded_lg()
            .p_6()
            .border_1()
            .border_color(theme.border)
            .shadow_sm()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text_muted)
                            .child(title)
                    )
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(value_color)
                            .child(value)
                    )
            )
    }
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme_registry.colors();
        
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.background)
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|view: &mut RootView, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<RootView>| {
                // Tab navigation using number keys 1-5
                // Time range filtering using alt+1, alt+2, alt+3
                if event.keystroke.modifiers.alt {
                    match event.keystroke.key.as_str() {
                        "1" => {
                            view.set_time_range(TimeRange::AllTime, cx);
                        }
                        "2" => {
                            view.set_time_range(TimeRange::Last30Days, cx);
                        }
                        "3" => {
                            view.set_time_range(TimeRange::Last7Days, cx);
                        }
                        _ => {}
                    }
                } else {
                    match event.keystroke.key.as_str() {
                        "1" => {
                            view.active_tab = DashboardTab::Overview;
                            cx.notify();
                        }
                        "2" => {
                            view.active_tab = DashboardTab::Models;
                            cx.notify();
                        }
                        "3" => {
                            view.active_tab = DashboardTab::Projects;
                            cx.notify();
                        }
                        "4" => {
                            view.active_tab = DashboardTab::Sessions;
                            cx.notify();
                        }
                        "5" => {
                            view.active_tab = DashboardTab::Timeline;
                            cx.notify();
                        }
                        _ => {}
                    }
                }
            }))
            .child(self.render_header(cx))
            .child(self.render_tab_navigation(cx))
            .child(self.render_main_content(cx))
    }
}

impl Focusable for RootView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}