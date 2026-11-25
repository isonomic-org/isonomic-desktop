// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The mark's sound: the brand's own recording, played once from the
//! window's first frame, at its own speed and to its own end. The mark's
//! intro is timed to it (`ui::onboarding::at`). It is the system's player
//! that plays it, at the volume the person has set; where there is no
//! such player the intro is silent.

/// The recording, as the brand folder holds it: two channels, 48 kHz.
const RECORDING: &[u8] = include_bytes!("../../../../assets/onboarding/isonomic-logo-sound.wav");

pub struct Sound {
    #[cfg(target_os = "macos")]
    player: objc2::rc::Retained<objc2_app_kit::NSSound>,
}

impl Sound {
    /// Ready the recording, so playing it later starts at once.
    #[cfg(target_os = "macos")]
    pub fn load() -> Option<Self> {
        use objc2::AnyThread as _;
        let data = objc2_foundation::NSData::with_bytes(RECORDING);
        let player = objc2_app_kit::NSSound::initWithData(objc2_app_kit::NSSound::alloc(), &data)?;
        Some(Self { player })
    }

    #[cfg(not(target_os = "macos"))]
    pub fn load() -> Option<Self> {
        let _ = RECORDING;
        None
    }

    /// Play it from its start. It plays on without being waited for.
    pub fn play(&self) {
        #[cfg(target_os = "macos")]
        if !self.player.play() {
            reports::log::line("onboarding: the mark's sound did not play");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RECORDING;

    #[test]
    fn the_recording_is_a_wave_file() {
        assert_eq!(&RECORDING[..4], b"RIFF");
        assert_eq!(&RECORDING[8..12], b"WAVE");
    }
}
