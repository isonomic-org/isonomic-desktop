// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The connector catalogue's rows, as data.

use super::Access::{Browser, Open, Token};
use super::Endpoint::{Remote, Typed, Unknown};
use super::{Connector, DESIGN, DEVELOPER, MONEY, PRODUCTIVITY, TALK};

const fn row(
    (key, name, about): (&'static str, &'static str, &'static str),
    category: &'static str,
    logo: Option<&'static str>,
    endpoint: super::Endpoint,
    access: super::Access,
    available: bool,
) -> Connector {
    Connector {
        key,
        name,
        about,
        category,
        logo,
        endpoint,
        access,
        available,
    }
}

#[rustfmt::skip]
pub const CONNECTORS: [Connector; 36] = [
    row(("github", "GitHub", "Pull requests, issues and repositories."), DEVELOPER, Some("github"), Remote("https://api.githubcopilot.com/mcp/"), Token, true),
    row(("linear", "Linear", "Issues, projects and cycles."), DEVELOPER, Some("linear"), Remote("https://mcp.linear.app/mcp"), Browser, true),
    row(("sentry", "Sentry", "Errors, issues and releases."), DEVELOPER, Some("sentry"), Remote("https://mcp.sentry.dev/mcp"), Browser, true),
    row(("vercel", "Vercel", "Deployments, projects and their logs."), DEVELOPER, Some("vercel"), Remote("https://mcp.vercel.com"), Browser, true),
    row(("cloudflare", "Cloudflare", "Workers, storage and DNS."), DEVELOPER, Some("cloudflare"), Remote("https://mcp.cloudflare.com/mcp"), Browser, true),
    row(("supabase", "Supabase", "Databases, tables and edge functions."), DEVELOPER, Some("supabase"), Remote("https://mcp.supabase.com/mcp"), Token, true),
    row(("hugging-face", "Hugging Face", "Models, datasets and Spaces."), DEVELOPER, Some("hugging-face"), Remote("https://huggingface.co/mcp"), Token, true),
    row(("notion", "Notion", "Pages, databases and notes."), PRODUCTIVITY, Some("notion"), Remote("https://mcp.notion.com/mcp"), Browser, true),
    row(("jira", "Jira", "Issues, boards and sprints."), PRODUCTIVITY, Some("jira"), Remote("https://mcp.atlassian.com/v2/mcp"), Browser, true),
    row(("confluence", "Confluence", "Spaces and pages."), PRODUCTIVITY, Some("confluence"), Remote("https://mcp.atlassian.com/v2/mcp"), Browser, true),
    row(("airtable", "Airtable", "Bases, tables and records."), PRODUCTIVITY, Some("airtable"), Remote("https://mcp.airtable.com/mcp"), Browser, true),
    row(("monday", "monday.com", "Boards, items and updates."), PRODUCTIVITY, None, Remote("https://mcp.monday.com/mcp"), Browser, true),
    row(("zapier", "Zapier", "Actions across thousands of apps."), PRODUCTIVITY, Some("zapier"), Remote("https://mcp.zapier.com/api/v1/connect"), Token, true),
    row(("intercom", "Intercom", "Conversations, tickets and contacts."), TALK, Some("intercom"), Remote("https://mcp.intercom.com/mcp"), Browser, true),
    row(("stripe", "Stripe", "Payments, customers and invoices."), MONEY, Some("stripe"), Remote("https://mcp.stripe.com"), Browser, true),
    row(("paypal", "PayPal", "Invoices, orders and transactions."), MONEY, Some("paypal"), Remote("https://mcp.paypal.com/http"), Browser, true),
    row(("webflow", "Webflow", "Sites, pages and collections."), DESIGN, Some("webflow"), Remote("https://mcp.webflow.com/mcp"), Browser, true),
    row(("figma-desktop", "Figma desktop app", "The design app's own server on this Mac."), DESIGN, Some("figma"), Remote("http://127.0.0.1:3845/mcp"), Open, true),
    row(("custom", "Custom server", "Any tool server, by its URL or the command that starts it."), DEVELOPER, None, Typed, Browser, true),
    row(("gmail", "Gmail", "Mail, threads and drafts."), TALK, Some("gmail"), Remote("https://gmailmcp.googleapis.com/mcp/v1"), Browser, false),
    row(("google-drive", "Google Drive", "Files and folders."), PRODUCTIVITY, Some("google-drive"), Remote("https://drivemcp.googleapis.com/mcp/v1"), Browser, false),
    row(("google-docs", "Google Docs", "Documents."), PRODUCTIVITY, Some("google-docs"), Remote("https://docsmcp.googleapis.com/mcp/v1"), Browser, false),
    row(("google-calendar", "Google Calendar", "Events and calendars."), PRODUCTIVITY, Some("google-calendar"), Remote("https://calendarmcp.googleapis.com/mcp/v1"), Browser, false),
    row(("figma", "Figma", "Design files, frames and components."), DESIGN, Some("figma"), Remote("https://mcp.figma.com/mcp"), Browser, false),
    row(("slack", "Slack", "Channels, threads and messages."), TALK, None, Remote("https://mcp.slack.com/mcp"), Browser, false),
    row(("dropbox", "Dropbox", "Files, folders and shared links."), PRODUCTIVITY, Some("dropbox"), Remote("https://mcp.dropbox.com/mcp"), Browser, false),
    row(("box", "Box", "Files and folders."), PRODUCTIVITY, Some("box"), Remote("https://mcp.box.com"), Browser, false),
    row(("outlook", "Microsoft Outlook", "Mail and calendar."), TALK, Some("microsoft"), Unknown, Browser, false),
    row(("teams", "Microsoft Teams", "Chats, channels and meetings."), TALK, Some("microsoft"), Unknown, Browser, false),
    row(("onedrive", "OneDrive", "Files and folders."), PRODUCTIVITY, Some("microsoft"), Unknown, Browser, false),
    row(("salesforce", "Salesforce", "Accounts, contacts and opportunities."), MONEY, None, Unknown, Browser, false),
    row(("hubspot", "HubSpot", "Contacts, companies and deals."), MONEY, Some("hubspot"), Remote("https://mcp.hubspot.com/"), Browser, false),
    row(("zoom", "Zoom", "Meetings and recordings."), TALK, Some("zoom"), Unknown, Browser, false),
    row(("asana", "Asana", "Tasks, projects and goals."), PRODUCTIVITY, Some("asana"), Remote("https://mcp.asana.com/v2/mcp"), Browser, false),
    row(("canva", "Canva", "Designs and brand templates."), DESIGN, None, Remote("https://mcp.canva.com/mcp"), Browser, false),
    row(("square", "Square", "Payments, orders and catalogue."), MONEY, Some("square"), Remote("https://mcp.squareup.com/mcp"), Browser, false),
];
