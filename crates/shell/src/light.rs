// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The menu bar status light: which color for which state, the icon drawn
//! for it, and the line of text beside it. Pure functions, tested on every
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

/// A filled circle in `rgba`, centered in a transparent square of
/// `light::ICON_SIZE` pixels, with a one-pixel soft edge.
pub fn icon(rgba: [u8; 4]) -> Vec<u8> {
    let size = light::ICON_SIZE;
    let center = size as f32 / 2.0;
    let radius = size as f32 * 0.28;
    let mut pixels = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let distance = (dx * dx + dy * dy).sqrt();
            let coverage = (radius + 0.5 - distance).clamp(0.0, 1.0);
            let alpha = (f32::from(rgba[3]) * coverage).round() as u8;
            pixels.extend_from_slice(&[rgba[0], rgba[1], rgba[2], alpha]);
        }
    }
    pixels
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

    #[test]
    fn the_icon_is_a_solid_dot_on_a_clear_square() {
        let size = light::ICON_SIZE as usize;
        let pixels = icon(light::HEALTHY);
        assert_eq!(pixels.len(), size * size * 4);
        let alpha = |x: usize, y: usize| pixels[(y * size + x) * 4 + 3];
        assert_eq!(alpha(size / 2, size / 2), 255, "center is solid");
        assert_eq!(alpha(0, 0), 0, "corner is clear");
        assert_eq!(
            &pixels[(size / 2 * size + size / 2) * 4..][..3],
            &light::HEALTHY[..3]
        );
    }
}
