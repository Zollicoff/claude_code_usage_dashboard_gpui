use gpui::*;
use gpui::prelude::FluentBuilder;

use crate::usage::types::{UsageEntry, SessionStats};
use crate::usage::stats::StatsCalculator;

pub struct TimelineChart {
    data: Vec<SessionStats>,
    max_cost: f64,
}

impl TimelineChart {
    pub fn new(entries: &[UsageEntry]) -> Self {
        let calculator = StatsCalculator::new();
        let session_stats = calculator.calculate_session_stats(entries);
        let max_cost = session_stats.iter()
            .map(|s| s.cost)
            .fold(0.0, f64::max);

        Self {
            data: session_stats,
            max_cost,
        }
    }

    pub fn render(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(0x111827))
                    .child("Daily Usage Timeline")
            )
            .child(
                div()
                    .bg(white())
                    .rounded(px(8.0))
                    .shadow_sm()
                    .p_6()
                    .child(
                        if self.data.is_empty() {
                            div()
                                .flex()
                                .justify_center()
                                .items_center()
                                .h(px(200.0))
                                .text_color(rgb(0x6b7280))
                                .child("No usage data available")
                                .into_any_element()
                        } else {
                            self.render_chart().into_any_element()
                        }
                    )
            )
    }

    fn render_chart(&self) -> Div {
        let chart_height = 200.0_f32;
        let bar_width = 20.0_f32;
        let bar_gap = 4.0_f32;
        let total_width = (self.data.len() as f32) * (bar_width + bar_gap);

        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(
                // Chart area
                div()
                    .h(px(chart_height))
                    .w(px(total_width.min(800.0)))
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .items_end()
                            .h_full()
                            .gap(px(bar_gap))
                            .children(self.data.iter().take(30).map(|session| {
                                self.render_bar(session, chart_height as f64, bar_width as f64)
                            }))
                    )
            )
            .child(
                // Legend
                div()
                    .flex()
                    .justify_between()
                    .text_sm()
                    .text_color(rgb(0x6b7280))
                    .child(format!("Showing last {} days", self.data.len().min(30)))
                    .child(format!("Peak: ${:.2}", self.max_cost))
            )
    }

    fn render_bar(&self, session: &SessionStats, chart_height: f64, bar_width: f64) -> impl IntoElement {
        let height_ratio = if self.max_cost > 0.0 {
            (session.cost / self.max_cost).min(1.0)
        } else {
            0.0
        };
        let bar_height = (chart_height * height_ratio * 0.8).max(2.0); // Leave some padding at top

        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(4.0))
            .child(
                // Bar
                div()
                    .w(px(bar_width as f32))
                    .h(px(bar_height as f32))
                    .bg(self.get_bar_color(session.cost))
                    .rounded(px(2.0))
                    .when(session.cost > 0.0, |div| {
                        div.hover(|div| {
                            div.bg(rgb(0xd97757))
                                .shadow_lg()
                        })
                    })
                    .relative()
                    .child(
                        // Tooltip on hover (simplified)
                        div()
                            .absolute()
                            .bottom(px((bar_height + 8.0) as f32))
                            .left(px(-30.0))
                            .w(px((bar_width + 60.0) as f32))
                            .p_2()
                            .bg(rgb(0x1f2937))
                            .text_color(white())
                            .text_xs()
                            .rounded(px(4.0))
                            .invisible() // TODO: Show on hover
                            .child(format!("${:.2}", session.cost))
                    )
            )
            .child(
                // Date label
                div()
                    .text_xs()
                    .text_color(rgb(0x9ca3af))
                    .child(
                        session.date
                            .split('-')
                            .last()
                            .unwrap_or("")
                            .to_string()
                    )
            )
    }

    fn get_bar_color(&self, cost: f64) -> Rgba {
        if cost == 0.0 {
            rgb(0xe5e7eb) // Gray for no usage
        } else if cost < self.max_cost * 0.3 {
            rgb(0x10b981) // Green for low usage
        } else if cost < self.max_cost * 0.7 {
            rgb(0xf59e0b) // Yellow for medium usage  
        } else {
            rgb(0xef4444) // Red for high usage
        }
    }
}