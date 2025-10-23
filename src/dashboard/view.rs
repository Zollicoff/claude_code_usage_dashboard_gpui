use gpui::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, error};

use crate::config::AppConfig;
use crate::integration::{IntegrationEngine, SyncManager, ConnectionTestResult};
use crate::models::IntegrationStats;

pub struct DashboardView {
    config: AppConfig,
    stats: Arc<RwLock<IntegrationStats>>,
    connection_status: Arc<RwLock<Option<ConnectionTestResult>>>,
    last_run_status: Arc<RwLock<String>>,
    is_running: Arc<RwLock<bool>>,
}

impl DashboardView {
    pub fn new(cx: &mut ViewContext<Self>, config: AppConfig) -> Self {
        info!("Creating dashboard view");

        let stats = Arc::new(RwLock::new(IntegrationStats::default()));
        let connection_status = Arc::new(RwLock::new(None));
        let last_run_status = Arc::new(RwLock::new("Not started".to_string()));
        let is_running = Arc::new(RwLock::new(false));

        // Initialize engine and load stats asynchronously
        let config_clone = config.clone();
        let stats_clone = stats.clone();
        let connection_clone = connection_status.clone();

        cx.spawn(|_view, mut cx| async move {
            // Initialize engine
            match IntegrationEngine::new(config_clone.clone()).await {
                Ok(engine) => {
                    info!("Integration engine initialized");

                    // Load initial stats
                    if let Ok(current_stats) = engine.get_statistics().await {
                        let mut stats = stats_clone.write().await;
                        *stats = current_stats;
                    }

                    // Test connections
                    if let Ok(test_result) = engine.test_connections().await {
                        let mut conn = connection_clone.write().await;
                        *conn = Some(test_result);
                    }

                    // Start scheduler if enabled
                    if config_clone.scheduler.enabled {
                        info!("Starting scheduler");
                        if let Ok(sync_manager) = SyncManager::new(
                            engine,
                            config_clone.scheduler.cron_expression.clone()
                        ).await {
                            if let Err(e) = sync_manager.start().await {
                                error!("Failed to start scheduler: {}", e);
                            } else {
                                info!("Scheduler started successfully");
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to initialize integration engine: {}", e);
                }
            }

            cx.update(|_cx| {
                // Update UI if needed
            }).ok();
        }).detach();

        Self {
            config,
            stats,
            connection_status,
            last_run_status,
            is_running,
        }
    }

    fn render_header(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .p_4()
            .border_b_1()
            .border_color(rgb(0x333333))
            .bg(rgb(0x1E1E1E))
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xFFFFFF))
                    .child("PropStream → Pipedrive Integration")
            )
            .child(
                div()
                    .mt_2()
                    .text_sm()
                    .text_color(rgb(0x888888))
                    .child("Automated real estate lead synchronization")
            )
    }

    fn render_stats(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let stats = self.stats.clone();

        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xFFFFFF))
                    .mb_4()
                    .child("Statistics")
            )
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(self.stat_card("Properties Fetched", "0", rgb(0x4A90E2)))
                    .child(self.stat_card("Qualified Leads", "0", rgb(0x50E3C2)))
                    .child(self.stat_card("Synced to Pipedrive", "0", rgb(0x7ED321)))
                    .child(self.stat_card("Duplicates Skipped", "0", rgb(0xF5A623)))
            )
    }

    fn stat_card(&self, label: &str, value: &str, color: Hsla) -> impl IntoElement {
        div()
            .flex_1()
            .p_4()
            .bg(rgb(0x2A2A2A))
            .rounded(px(8.0))
            .border_1()
            .border_color(color)
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x888888))
                    .mb_2()
                    .child(label)
            )
            .child(
                div()
                    .text_3xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(color)
                    .child(value)
            )
    }

    fn render_controls(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .p_4()
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(
                        div()
                            .px_6()
                            .py_3()
                            .bg(rgb(0x4A90E2))
                            .rounded(px(6.0))
                            .text_color(rgb(0xFFFFFF))
                            .font_weight(FontWeight::BOLD)
                            .cursor_pointer()
                            .child("Run Integration")
                    )
                    .child(
                        div()
                            .px_6()
                            .py_3()
                            .bg(rgb(0x2A2A2A))
                            .border_1()
                            .border_color(rgb(0x4A90E2))
                            .rounded(px(6.0))
                            .text_color(rgb(0x4A90E2))
                            .font_weight(FontWeight::BOLD)
                            .cursor_pointer()
                            .child("Test Connections")
                    )
                    .child(
                        div()
                            .px_6()
                            .py_3()
                            .bg(rgb(0x2A2A2A))
                            .border_1()
                            .border_color(rgb(0x50E3C2))
                            .rounded(px(6.0))
                            .text_color(rgb(0x50E3C2))
                            .font_weight(FontWeight::BOLD)
                            .cursor_pointer()
                            .child("Refresh Stats")
                    )
            )
    }

    fn render_status(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xFFFFFF))
                    .mb_4()
                    .child("Connection Status")
            )
            .child(
                div()
                    .p_4()
                    .bg(rgb(0x2A2A2A))
                    .rounded(px(8.0))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(self.connection_item("PropStream API", true))
                            .child(self.connection_item("Pipedrive API", true))
                            .child(self.connection_item("Database", true))
                    )
            )
    }

    fn connection_item(&self, name: &str, connected: bool) -> impl IntoElement {
        let (color, status) = if connected {
            (rgb(0x7ED321), "Connected")
        } else {
            (rgb(0xD0021B), "Disconnected")
        };

        div()
            .flex()
            .justify_between()
            .items_center()
            .p_2()
            .child(
                div()
                    .text_color(rgb(0xFFFFFF))
                    .child(name)
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .w(px(8.0))
                            .h(px(8.0))
                            .bg(color)
                            .rounded(px(4.0))
                    )
                    .child(
                        div()
                            .text_color(color)
                            .text_sm()
                            .child(status)
                    )
            )
    }

    fn render_config(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .p_4()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xFFFFFF))
                    .mb_4()
                    .child("Configuration")
            )
            .child(
                div()
                    .p_4()
                    .bg(rgb(0x2A2A2A))
                    .rounded(px(8.0))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(self.config_item("Scheduler", if self.config.scheduler.enabled { "Enabled" } else { "Disabled" }))
                            .child(self.config_item("Schedule", &self.config.scheduler.cron_expression))
                            .child(self.config_item("Max Leads/Run", &self.config.scheduler.max_leads_per_run.to_string()))
                    )
            )
    }

    fn config_item(&self, label: &str, value: &str) -> impl IntoElement {
        div()
            .flex()
            .justify_between()
            .p_2()
            .child(
                div()
                    .text_color(rgb(0x888888))
                    .child(label)
            )
            .child(
                div()
                    .text_color(rgb(0xFFFFFF))
                    .font_weight(FontWeight::BOLD)
                    .child(value)
            )
    }
}

impl Render for DashboardView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1A1A1A))
            .text_color(rgb(0xFFFFFF))
            .child(self.render_header(cx))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .overflow_y_scroll()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w_full()
                            .p_6()
                            .gap_6()
                            .child(self.render_stats(cx))
                            .child(self.render_controls(cx))
                            .child(
                                div()
                                    .flex()
                                    .gap_6()
                                    .child(
                                        div()
                                            .flex_1()
                                            .child(self.render_status(cx))
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .child(self.render_config(cx))
                                    )
                            )
                    )
            )
    }
}
