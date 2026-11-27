// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The connector catalogue: the tool servers onboarding and Settings
//! offer, each with its logo, its server, how it signs in, and whether
//! it can be connected yet. The rows are data, in `table`.
//!
//! A connector the engine signs in to through the browser needs nothing
//! from the person here; one that takes a pasted token asks for it in its
//! row first. "Custom server" asks for a URL or a command.

mod table;

use isonomic_proto::connections::{AddConnection, ConnectionSecret, Server};

pub use table::CONNECTORS;

pub const DEVELOPER: &str = "Developer tools";
pub const PRODUCTIVITY: &str = "Productivity";
pub const TALK: &str = "Communication";
pub const MONEY: &str = "Business";
pub const DESIGN: &str = "Design";
/// The first tab, which shows every connector.
pub const ALL: &str = "All integrations";

/// Where a connector's server is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endpoint {
    /// Reached at this URL.
    Remote(&'static str),
    /// Started on this computer.
    Local(&'static str, &'static [&'static str]),
    /// Typed by the person: a URL, or a command and its arguments.
    Typed,
    /// Not known yet: the connector is coming soon.
    Unknown,
}

/// How a connector signs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    /// In the browser, through the engine.
    Browser,
    /// With a token the person pastes into the row.
    Token,
    /// Not at all: a server on this computer.
    Open,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Connector {
    pub key: &'static str,
    pub name: &'static str,
    pub about: &'static str,
    pub category: &'static str,
    /// The logo's file, under `assets/onboarding/logos`; a monogram when
    /// there is none.
    pub logo: Option<&'static str>,
    pub endpoint: Endpoint,
    pub access: Access,
    /// Can it be connected now? The rest are listed as coming soon.
    pub available: bool,
}

impl Connector {
    /// Does its row ask for something typed before it can be switched on?
    pub fn asks(&self) -> Option<&'static str> {
        match (self.endpoint, self.access) {
            (Endpoint::Typed, _) => Some("Server URL, or the command that starts it"),
            (_, Access::Token) => Some("Paste an access token"),
            _ => None,
        }
    }

    /// The call that connects it, with what was typed in its row, if
    /// that is enough.
    pub fn add(&self, typed: &str) -> Option<AddConnection> {
        let typed = typed.trim();
        if !self.available || (self.asks().is_some() && typed.is_empty()) {
            return None;
        }
        let server = match self.endpoint {
            Endpoint::Remote(url) => Server::Remote { url: url.into() },
            Endpoint::Local(command, args) => Server::Local {
                command: command.into(),
                args: args.iter().map(|arg| (*arg).to_owned()).collect(),
            },
            Endpoint::Typed => typed_server(typed)?,
            Endpoint::Unknown => return None,
        };
        let secret = (self.access == Access::Token && self.endpoint != Endpoint::Typed)
            .then(|| ConnectionSecret::new(typed));
        let name = match self.endpoint {
            Endpoint::Typed => typed_name(&server),
            _ => self.name.to_owned(),
        };
        Some(AddConnection {
            name,
            key: Some(self.key.to_owned()),
            server,
            secret,
        })
    }
}

/// A URL is a remote server; anything else, a command and its arguments.
fn typed_server(typed: &str) -> Option<Server> {
    if typed.starts_with("https://") || typed.starts_with("http://") {
        return Some(Server::Remote { url: typed.into() });
    }
    let mut words = typed.split_whitespace().map(str::to_owned);
    let command = words.next()?;
    Some(Server::Local {
        command,
        args: words.collect(),
    })
}

/// What a custom server is called: its host, or its command.
fn typed_name(server: &Server) -> String {
    match server {
        Server::Remote { url } => url
            .split("://")
            .nth(1)
            .and_then(|rest| rest.split('/').next())
            .unwrap_or(url)
            .to_owned(),
        Server::Local { command, .. } => command.rsplit('/').next().unwrap_or(command).to_owned(),
        _ => "Custom server".to_owned(),
    }
}

/// The connector with `key`.
pub fn find(key: &str) -> Option<&'static Connector> {
    CONNECTORS.iter().find(|connector| connector.key == key)
}

/// The tabs: every connector, then each category in the order the
/// catalogue first has it.
pub fn tabs() -> Vec<&'static str> {
    let mut tabs = vec![ALL];
    for connector in &CONNECTORS {
        if !tabs.contains(&connector.category) {
            tabs.push(connector.category);
        }
    }
    tabs
}

/// Is `connector` under tab `tab`, with `sought` in the search box?
pub fn listed(connector: &Connector, tab: usize, sought: &str) -> bool {
    let sought = sought.trim().to_lowercase();
    let under = tab == 0 || tabs().get(tab) == Some(&connector.category);
    under
        && (connector.name.to_lowercase().contains(&sought)
            || connector.about.to_lowercase().contains(&sought))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_is_once_and_every_available_one_has_a_server() {
        for (index, connector) in CONNECTORS.iter().enumerate() {
            assert!(CONNECTORS[..index].iter().all(|c| c.key != connector.key));
            assert!(!connector.available || connector.endpoint != Endpoint::Unknown);
        }
    }

    #[test]
    fn a_token_row_needs_its_token_and_carries_it_as_a_secret() {
        let token = find("github").unwrap();
        assert_eq!(token.add("  "), None);
        let add = token.add("ghp-1").unwrap();
        assert_eq!(
            add.secret.map(|s| s.reveal().to_owned()).as_deref(),
            Some("ghp-1")
        );
        let browser = find("linear").unwrap().add("").unwrap();
        assert!(browser.secret.is_none() && browser.key.as_deref() == Some("linear"));
        assert_eq!(find("gmail").unwrap().add(""), None);
    }

    #[test]
    fn a_custom_server_is_a_url_or_a_command() {
        let custom = find("custom").unwrap();
        let remote = custom.add("https://tools.example.com/mcp").unwrap();
        assert_eq!(remote.name, "tools.example.com");
        let local = custom.add("/usr/local/bin/server --stdio").unwrap();
        assert_eq!(
            local.server,
            Server::Local {
                command: "/usr/local/bin/server".into(),
                args: vec!["--stdio".into()]
            }
        );
        assert_eq!(local.name, "server");
    }
}
