use gpui::*;
use gpui::prelude::FluentBuilder;

#[derive(Clone, Debug, PartialEq)]
pub enum Tab {
    Overview,
    Models,
    Projects,
    Sessions,
    Timeline,
}

impl Tab {
    pub fn all() -> Vec<Tab> {
        vec![
            Tab::Overview,
            Tab::Models,
            Tab::Projects,
            Tab::Sessions,
            Tab::Timeline,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            Tab::Overview => "Overview",
            Tab::Models => "By Model",
            Tab::Projects => "By Project",
            Tab::Sessions => "By Session",
            Tab::Timeline => "Timeline",
        }
    }
}

pub struct TabBar {
    active_tab: Tab,
}

impl TabBar {
    pub fn new() -> Self {
        Self {
            active_tab: Tab::Overview,
        }
    }

    pub fn active_tab(&self) -> &Tab {
        &self.active_tab
    }

    pub fn set_active_tab(&mut self, tab: Tab) {
        self.active_tab = tab;
    }

    pub fn render(&self, on_tab_click: impl Fn(Tab) + 'static + Clone) -> impl IntoElement {
        div()
            .flex()
            .bg(white())
            .border_b_1()
            .border_color(rgb(0xe5e7eb))
            .children(Tab::all().into_iter().map(|tab| {
                let is_active = tab == self.active_tab;
                let tab_clone = tab.clone();
                let on_click = on_tab_click.clone();
                
                div()
                    .px_6()
                    .py_3()
                    .cursor_pointer()
                    .when(is_active, |div| {
                        div.border_b_2()
                            .border_color(rgb(0xd97757)) // Orange accent
                            .text_color(rgb(0xd97757))
                            .font_weight(FontWeight::SEMIBOLD)
                    })
                    .when(!is_active, |div| {
                        div.text_color(rgb(0x6b7280))
                            .hover(|div| div.text_color(rgb(0x374151)))
                    })
                    // TODO: Add click handler when we have proper state management
                    .child(tab.title())
            }))
    }
}