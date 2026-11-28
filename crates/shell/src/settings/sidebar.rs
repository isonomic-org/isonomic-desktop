// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The list of pages down the left, by section, with its search: typing
//! puts the pages and settings that match in the list's place.

use ui::base::v_flex;
use ui::component::input::Input;
use ui::component::sidebar::{Sidebar, SidebarGroup, SidebarMenu, SidebarMenuItem};
use ui::component::theme::ActiveTheme as _;
use ui::component::StyledExt as _;
use ui::settings as sizes;
use ui::{
    div, relative, AnyElement, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, Styled as _, TestSupportExt as _, Window,
};

use super::spec::{self, list, text};
use super::{marks, Settings};

impl Settings {
    /// What the sidebar's search holds.
    pub fn query(&self, cx: &Context<Self>) -> String {
        self.search.read(cx).value().trim().to_string()
    }

    /// Go to a search result's page and empty the search.
    fn pick(&mut self, page: &'static str, window: &mut Window, cx: &mut Context<Self>) {
        self.search
            .update(cx, |search, cx| search.set_value("", window, cx));
        self.go(page, cx);
    }

    fn pages(&self, cx: &Context<Self>) -> Vec<SidebarGroup<SidebarMenu>> {
        let current = spec::owner(&self.page);
        let mut groups = Vec::new();
        for section in spec::sections() {
            let items = list(section, "items").iter().map(|item| {
                let page = text(item, "id");
                SidebarMenuItem::new(text(item, "title"))
                    .id(SharedString::from(format!("settings-nav-{page}")))
                    .icon(marks::icon(text(item, "icon")))
                    .active(page == current)
                    .on_click(cx.listener(move |this, _, _, cx| this.go(page, cx)))
            });
            let menu = SidebarMenu::new().children(items);
            groups.push(SidebarGroup::new(text(section, "label")).child(menu));
        }
        groups
    }

    fn results(&self, query: &str, cx: &Context<Self>) -> Vec<SidebarGroup<SidebarMenu>> {
        let hits = spec::search(query);
        if hits.is_empty() {
            return vec![SidebarGroup::new(format!("No settings match “{query}”"))];
        }
        let items = hits.into_iter().enumerate().map(|(index, hit)| {
            let page = hit.page;
            SidebarMenuItem::new(hit.title)
                .id(SharedString::from(format!("settings-hit-{index}")))
                .icon(marks::icon(hit.icon))
                .suffix(move |_, cx| {
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(hit.under)
                })
                .on_click(cx.listener(move |this, _, window, cx| this.pick(page, window, cx)))
        });
        let menu = SidebarMenu::new().children(items);
        vec![SidebarGroup::new("Results").child(menu)]
    }

    pub(super) fn sidebar(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let query = self.query(cx);
        let groups = if query.is_empty() {
            self.pages(cx)
        } else {
            self.results(&query, cx)
        };
        let search = Input::new(&self.search)
            .prefix(marks::icon("search"))
            .cleanable(true);
        let header = v_flex()
            .w_full()
            .gap_3()
            .child(div().px_2().text_base().font_semibold().child("Settings"))
            .child(search);
        div()
            .id("settings-sidebar")
            .test_support()
            .w(sizes::sidebar_width())
            .h_full()
            .flex_shrink_0()
            .child(
                Sidebar::new("settings-nav")
                    .w(relative(1.))
                    .collapsible(false)
                    .content_gap(sizes::section_gap())
                    .header(header)
                    .children(groups),
            )
            .into_any_element()
    }
}
