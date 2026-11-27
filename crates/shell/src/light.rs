// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The status light: which color for which state, and the line of text
//! beside it. Pure functions, tested on every
//! platform; only `menubar` touches the native status item.

use isonomic_client::Live;
use isonomic_proto::ServiceState;
use ui::tokens::light;

use crate::home::headline;

/// Light color for what the engine reports. No snapshot yet counts as
/// stopped: the window is still connecting.
pub fn color(live: &Live) -> [u8; 4] {
    match live.system.as_ref().map(|system| system.aggregate) {
        Some(ServiceState::Healthy) => light::HEALTHY,
        Some(ServiceState::Starting) => light::STARTING,
        Some(ServiceState::Degraded) => light::DEGRADED,
        Some(ServiceState::Stopped) | None => light::STOPPED,
    }
}

/// The text of the menu's first, disabled item.
pub fn status_line(live: &Live) -> String {
    format!("Isonomic: {}", headline(live))
}

#[cfg(test)]
mod tests {
    use super::*;
    use isonomic_proto::SystemStatus;

    fn live(state: ServiceState) -> Live {
        Live {
            schema: None,
            connected: true,
            engine: None,
            system: Some(SystemStatus {
                aggregate: state,
                services: Vec::new(),
                cold_start_ms: None,
            }),
        }
    }

    #[test]
    fn each_state_has_its_own_light_and_connecting_is_grey() {
        assert_eq!(color(&live(ServiceState::Healthy)), light::HEALTHY);
        assert_eq!(color(&live(ServiceState::Starting)), light::STARTING);
        assert_eq!(color(&live(ServiceState::Degraded)), light::DEGRADED);
        assert_eq!(color(&Live::default()), light::STOPPED);
        assert_eq!(
            status_line(&live(ServiceState::Healthy)),
            "Isonomic: All services healthy"
        );
    }
}
