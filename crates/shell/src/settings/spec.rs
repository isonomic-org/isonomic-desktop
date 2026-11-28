// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the Settings pages hold. Each page, tab, sub-page and dialog of the
//! design is one JSON file under `assets/settings`, read once and walked as
//! it is drawn. A page is a list of blocks (`k`): a header, a section's
//! title, a group of rows, tabs, a table. A row ends in controls (`c`): a
//! switch, a choice, a button, a field.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde_json::Value;

use super::specs::{LINKS, NAV, SPECS};

/// The most results the search lists.
const MOST_HITS: usize = 24;

struct Book {
    specs: HashMap<&'static str, Value>,
    nav: Value,
    links: Value,
}

fn book() -> &'static Book {
    static BOOK: OnceLock<Book> = OnceLock::new();
    BOOK.get_or_init(|| Book {
        specs: SPECS.iter().map(|(id, json)| (*id, read(json))).collect(),
        nav: read(NAV),
        links: read(LINKS),
    })
}

fn read(json: &str) -> Value {
    serde_json::from_str(json).expect("a settings file is JSON")
}

/// The page, tab, sub-page or dialog called `id`.
pub fn spec(id: &str) -> Option<&'static Value> {
    book().specs.get(id)
}

/// Every name there is, pages and dialogs alike.
pub fn ids() -> impl Iterator<Item = &'static str> {
    SPECS.iter().map(|(id, _)| *id)
}

/// `id` as the name that lives as long as the program, when there is one.
pub fn named(id: &str) -> Option<&'static str> {
    ids().find(|known| *known == id)
}

/// The sidebar's sections, each with its pages.
pub fn sections() -> &'static [Value] {
    match book().nav.as_array() {
        Some(sections) => sections.as_slice(),
        None => &[],
    }
}

/// The text under `key`, or nothing.
pub fn text(of: &'static Value, key: &str) -> &'static str {
    of[key].as_str().unwrap_or("")
}

/// The list under `key`, or none.
pub fn list(of: &'static Value, key: &str) -> &'static [Value] {
    match of[key].as_array() {
        Some(list) => list.as_slice(),
        None => &[],
    }
}

/// What the button `label` of `row` on `spec` opens: `d:` and a dialog's
/// name, or `p:` and a page's. A row that leads somewhere has the label `>`.
pub fn link(spec: &str, row: &str, label: &str) -> Option<&'static str> {
    book().links[format!("{spec}|{row}|{label}").as_str()].as_str()
}

/// A title as it is written in a name: lower case, a hyphen for each run
/// of anything else.
pub fn slug(title: &str) -> String {
    let mut out = String::new();
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_string()
}

fn pages() -> impl Iterator<Item = &'static str> {
    sections()
        .iter()
        .flat_map(|section| list(section, "items"))
        .map(|item| text(item, "id"))
}

/// The sidebar's page that `id` is shown under: itself, the page its
/// "back" names, or the page whose tab it is.
pub fn owner(id: &str) -> &'static str {
    if let Some(page) = pages().find(|page| *page == id) {
        return page;
    }
    let back = spec(id).and_then(|spec| {
        list(spec, "blocks")
            .iter()
            .find(|block| text(block, "k") == "back")
    });
    if let Some(back) = back {
        let parent = slug(text(back, "label"));
        if let Some(page) = pages().find(|page| *page == parent) {
            return page;
        }
    }
    pages()
        .filter(|page| id.starts_with(&format!("{page}-")))
        .max_by_key(|page| page.len())
        .unwrap_or("general")
}

/// The page behind tab `index`, called `label`, of the page `owner`.
pub fn tab(owner: &str, label: &str, index: usize) -> Option<&'static str> {
    if index == 0 {
        return named(owner);
    }
    named(&format!("{owner}-{}", slug(label)))
}

/// One result of the sidebar's search.
pub struct Hit {
    pub title: &'static str,
    /// The page a setting is on, or the section a page is in.
    pub under: &'static str,
    pub page: &'static str,
    pub icon: &'static str,
}

/// The pages and the settings whose titles hold `query`.
pub fn search(query: &str) -> Vec<Hit> {
    let query = query.to_lowercase();
    let mut hits = Vec::new();
    for section in sections() {
        for item in list(section, "items") {
            let (page, title, icon) = (text(item, "id"), text(item, "title"), text(item, "icon"));
            if title.to_lowercase().contains(&query) {
                let under = text(section, "label");
                hits.push(Hit {
                    title,
                    under,
                    page,
                    icon,
                });
            }
            let blocks = spec(page).map_or(&[][..], |spec| list(spec, "blocks"));
            for row in blocks.iter().flat_map(|block| list(block, "rows")) {
                let name = text(row, "title");
                if !name.is_empty() && name.to_lowercase().contains(&query) {
                    hits.push(Hit {
                        title: name,
                        under: title,
                        page,
                        icon,
                    });
                }
            }
        }
    }
    hits.truncate(MOST_HITS);
    hits
}

/// What is wrong with the files, for the tests: a page the sidebar names
/// that is not there, a button that opens nothing.
pub fn faults() -> Vec<String> {
    let mut faults = Vec::new();
    for page in pages() {
        if spec(page).is_none() {
            faults.push(format!("the sidebar names {page}, which is not a page"));
        }
    }
    let links = book().links.as_object();
    for (from, to) in links.into_iter().flatten() {
        let to = to.as_str().unwrap_or("");
        let known = to.split_once(':').and_then(|(_, id)| spec(id)).is_some();
        if !known {
            faults.push(format!("{from} opens {to}, which is not there"));
        }
        if spec(from.split('|').next().unwrap_or("")).is_none() {
            faults.push(format!("{from} is on no page"));
        }
    }
    faults
}
