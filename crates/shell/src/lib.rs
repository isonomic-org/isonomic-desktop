//! The window's shell (architecture spec, sections 2 and 6): a rail of tabs
//! down the left and a host that shows the selected one.
//!
//! Each tab is an entity that is built once and stays alive. The shell knows
//! nothing about the engine beyond the live status watch it hands to the
//! Home tab; it never touches a process, a socket or a path.

pub mod home;

use isonomic_client::Live;
use tokio::sync::watch;
use ui::base::{h_flex, v_flex};
use ui::component::theme::ActiveTheme as _;
use ui::{
    div, tokens, AnyView, AppContext as _, Context, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _,
    TestSupportExt as _, Window,
};

use home::HomeTab;

/// One entry in the rail and the view it opens.
struct Tab {
    /// Stable identifier, used for the rail item's element id.
    id: &'static str,
    title: SharedString,
    view: AnyView,
}

pub struct Shell {
    tabs: Vec<Tab>,
    selected: usize,
}

impl Shell {
    /// Build the shell with its system tabs. `live` is the engine link's
    /// watch on connection and service status.
    pub fn new(live: watch::Receiver<Live>, cx: &mut Context<Self>) -> Self {
        let home = cx.new(|cx| HomeTab::new(live, cx));
        Self {
            tabs: vec![Tab {
                id: "home",
                title: "Home".into(),
                view: home.into(),
            }],
            selected: 0,
        }
    }

    /// Identifier of the tab on screen.
    pub fn selected_tab(&self) -> &'static str {
        self.tabs[self.selected].id
    }

    fn select(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.tabs.len() && index != self.selected {
            self.selected = index;
            cx.notify();
        }
    }
}

impl Render for Shell {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let (accent, accent_text) = (theme.sidebar_accent, theme.sidebar_accent_foreground);
        let rail_items: Vec<_> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| {
                let selected = index == self.selected;
                // A monogram stands in for the icon until the icon set lands.
                let monogram: SharedString = tab.title.chars().take(1).collect::<String>().into();
                let mut item = div()
                    .id(SharedString::from(format!("rail-{}", tab.id)))
                    .test_support()
                    .size_9()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_md()
                    .cursor_pointer()
                    .child(monogram)
                    .on_click(cx.listener(move |shell, _, _, cx| shell.select(index, cx)));
                if selected {
                    item = item.bg(accent).text_color(accent_text);
                }
                item
            })
            .collect();

        h_flex()
            .id("shell")
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(
                v_flex()
                    .id("rail")
                    .test_support()
                    .w(tokens::rail_width())
                    .h_full()
                    .flex_shrink_0()
                    .items_center()
                    .gap_1()
                    .py_2()
                    .bg(theme.sidebar)
                    .text_color(theme.sidebar_foreground)
                    .border_r_1()
                    .border_color(theme.sidebar_border)
                    .children(rail_items),
            )
            .child(
                div()
                    .id("tab-host")
                    .flex_1()
                    .h_full()
                    .overflow_hidden()
                    .child(self.tabs[self.selected].view.clone()),
            )
    }
}
