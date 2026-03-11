//! Debug domain resources.

use bevy::prelude::*;

/// Resource controlling which debug overlays are visible.
#[allow(clippy::struct_excessive_bools)]
#[derive(Resource, Default)]
pub struct DebugOverlays {
    /// Show frames-per-second counter.
    pub show_fps: bool,
    /// Show entity hitbox outlines.
    pub show_hitboxes: bool,
    /// Show velocity vector arrows on moving entities.
    pub show_velocity_vectors: bool,
    /// Show current game state label.
    pub show_state: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_overlays_default_all_off() {
        let overlays = DebugOverlays::default();
        assert!(!overlays.show_fps);
        assert!(!overlays.show_hitboxes);
        assert!(!overlays.show_velocity_vectors);
        assert!(!overlays.show_state);
    }
}
