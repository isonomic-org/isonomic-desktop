// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The self-check's icon checks: the app's asset source serves Isonomic
//! UI's icons (every run, at connect), and in the check build a known
//! icon puts pixels on the screen (once first run is over).

use std::time::Instant;

use ui::{AnyWindowHandle, AsyncApp};

use super::say;
#[cfg(feature = "check-build")]
use super::screens;

/// The icons load: the app's asset source serves Isonomic UI's icons. Run
/// at connect, before first run.
pub(super) fn load(started: Instant, cx: &mut AsyncApp) -> Result<(), String> {
    use ui::assets::IconName;
    let path = IconName::Plus.path();
    let loaded = cx.update(|cx| cx.asset_source().load(&path));
    match loaded {
        Ok(Some(bytes)) if !bytes.is_empty() => {
            say(started, &format!("icons: loaded ({path})"));
            Ok(())
        }
        _ => Err(format!(
            "icons: {path} is not in the app's assets, so no icon draws"
        )),
    }
}

/// The icons draw, in the check build: the rail's Home icon puts pixels
/// on the screen. The rail is there only once first run is over, so this
/// runs at the end of `ready`.
pub(super) fn drawn(
    window: AnyWindowHandle,
    started: Instant,
    cx: &mut AsyncApp,
) -> Result<(), String> {
    #[cfg(feature = "check-build")]
    {
        let ink = cx.update(|cx| screens::ink_in(window, "rail-home", cx))?;
        if ink == 0 {
            return Err("icons: the rail's Home icon drew no pixels".into());
        }
        say(
            started,
            &format!("icons: the rail's Home icon drew {ink} pixels"),
        );
    }
    #[cfg(not(feature = "check-build"))]
    let _ = (window, started, cx);
    Ok(())
}
