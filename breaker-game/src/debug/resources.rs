//! Debug domain resources.

#[cfg(feature = "dev")]
use bevy::prelude::*;

/// Identifies a specific debug overlay.
#[cfg(feature = "dev")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Overlay {
    /// Frames-per-second counter.
    Fps,
    /// Entity hitbox outlines.
    Hitboxes,
    /// Velocity vector arrows on moving entities.
    VelocityVectors,
    /// Current game state label.
    State,
    /// Bolt telemetry window.
    BoltInfo,
    /// Breaker state telemetry window.
    DashState,
    /// Input actions debug window.
    InputActions,
}

#[cfg(feature = "dev")]
impl Overlay {
    /// Total number of overlay variants.
    const COUNT: usize = 7;
}

/// Resource controlling which debug overlays are visible.
///
/// Uses an enum-indexed array instead of individual bool fields to avoid
/// `clippy::struct_excessive_bools`.
#[cfg(feature = "dev")]
#[derive(Resource)]
pub(crate) struct DebugOverlays {
    /// Visibility flags indexed by [`Overlay`] variant.
    flags: [bool; Overlay::COUNT],
}

#[cfg(feature = "dev")]
impl Default for DebugOverlays {
    fn default() -> Self {
        Self {
            flags: [false; Overlay::COUNT],
        }
    }
}

#[cfg(feature = "dev")]
impl DebugOverlays {
    /// Returns a mutable reference to the flag for the given overlay.
    ///
    /// Compatible with `egui::Ui::checkbox(&mut bool, label)`.
    pub(crate) const fn flag_mut(&mut self, overlay: Overlay) -> &mut bool {
        &mut self.flags[overlay as usize]
    }

    /// Returns whether the given overlay is active.
    #[must_use]
    pub(crate) const fn is_active(&self, overlay: Overlay) -> bool {
        self.flags[overlay as usize]
    }
}

/// Tracks the last bump outcome for debug display.
#[cfg(feature = "dev")]
#[derive(Resource, Default)]
pub(crate) struct LastBumpResult(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "dev")]
    #[test]
    fn debug_overlays_default_all_off() {
        let overlays = DebugOverlays::default();
        assert!(!overlays.is_active(Overlay::Fps));
        assert!(!overlays.is_active(Overlay::Hitboxes));
        assert!(!overlays.is_active(Overlay::VelocityVectors));
        assert!(!overlays.is_active(Overlay::State));
        assert!(!overlays.is_active(Overlay::BoltInfo));
        assert!(!overlays.is_active(Overlay::DashState));
        assert!(!overlays.is_active(Overlay::InputActions));
    }

    #[cfg(feature = "dev")]
    #[test]
    fn last_bump_result_default_is_empty() {
        let result = LastBumpResult::default();
        assert!(result.0.is_empty());
    }
}
