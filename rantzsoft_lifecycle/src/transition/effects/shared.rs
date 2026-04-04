//! Shared types for built-in transition effects.

use bevy::prelude::*;

/// Marker component for overlay entities spawned by transition effects.
///
/// End systems query for this component to find and despawn overlay entities.
#[derive(Component)]
pub struct TransitionOverlay;

/// Tracks elapsed time and completion state for transition run systems.
///
/// Inserted by start systems, read by run systems, removed by end systems.
/// Tests set this directly rather than relying on `Time<Real>`.
#[derive(Resource)]
pub struct TransitionProgress {
    /// Time elapsed since the transition started.
    pub elapsed: f32,
    /// Total duration of the transition.
    pub duration: f32,
    /// Whether the transition has completed (guards against double-sending
    /// `TransitionRunComplete`).
    pub completed: bool,
}

/// Screen dimensions used by overlay effects to size their sprites.
///
/// Defaults to 1920x1080. Tests can override with specific values.
#[derive(Resource)]
pub struct ScreenSize(pub Vec2);

impl Default for ScreenSize {
    fn default() -> Self {
        Self(Vec2::new(1920.0, 1080.0))
    }
}

/// Start and target positions for slide transitions.
///
/// Inserted by slide start systems, read by run systems, removed by end systems.
#[derive(Resource)]
pub(crate) struct SlideStartEnd {
    /// Camera position when the slide began.
    pub start: Vec2,
    /// Target camera position at the end of the slide.
    pub target: Vec2,
}

/// Direction for wipe transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WipeDirection {
    /// Wipe from right to left.
    #[default]
    Left,
    /// Wipe from left to right.
    Right,
    /// Wipe from bottom to top.
    Up,
    /// Wipe from top to bottom.
    Down,
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- TransitionOverlay ---

    #[test]
    fn transition_overlay_can_be_spawned_and_queried() {
        let mut world = World::new();
        let entity = world.spawn(TransitionOverlay).id();
        let has_overlay = world.get::<TransitionOverlay>(entity).is_some();
        assert!(
            has_overlay,
            "entity should have TransitionOverlay component"
        );
    }

    // --- TransitionProgress ---

    #[test]
    fn transition_progress_can_be_inserted_as_resource() {
        let mut world = World::new();
        world.insert_resource(TransitionProgress {
            elapsed: 0.0,
            duration: 0.5,
            completed: false,
        });
        assert!(world.contains_resource::<TransitionProgress>());
    }

    #[test]
    fn transition_progress_fields_are_accessible() {
        let mut world = World::new();
        world.insert_resource(TransitionProgress {
            elapsed: 0.3,
            duration: 0.5,
            completed: false,
        });
        let progress = world.resource::<TransitionProgress>();
        assert!((progress.elapsed - 0.3).abs() < f32::EPSILON);
        assert!((progress.duration - 0.5).abs() < f32::EPSILON);
        assert!(!progress.completed);
    }

    // --- ScreenSize ---

    #[test]
    fn screen_size_default_is_1920x1080() {
        let size = ScreenSize::default();
        assert!(
            (size.0.x - 1920.0).abs() < f32::EPSILON,
            "default width should be 1920.0"
        );
        assert!(
            (size.0.y - 1080.0).abs() < f32::EPSILON,
            "default height should be 1080.0"
        );
    }

    #[test]
    fn screen_size_can_be_set_to_custom_values() {
        let size = ScreenSize(Vec2::new(1280.0, 720.0));
        assert!((size.0.x - 1280.0).abs() < f32::EPSILON);
        assert!((size.0.y - 720.0).abs() < f32::EPSILON);
    }

    // --- SlideStartEnd ---

    #[test]
    fn slide_start_end_stores_start_and_target() {
        let mut world = World::new();
        world.insert_resource(SlideStartEnd {
            start: Vec2::new(0.0, 0.0),
            target: Vec2::new(-1280.0, 0.0),
        });
        let sse = world.resource::<SlideStartEnd>();
        assert!((sse.start.x).abs() < f32::EPSILON);
        assert!((sse.target.x - (-1280.0)).abs() < f32::EPSILON);
    }

    // --- WipeDirection ---

    #[test]
    fn wipe_direction_default_is_left() {
        assert_eq!(WipeDirection::default(), WipeDirection::Left);
    }

    #[test]
    fn wipe_direction_has_all_four_variants() {
        let left = WipeDirection::Left;
        let _ = &left;
        let right = WipeDirection::Right;
        let _ = &right;
        let up = WipeDirection::Up;
        let _ = &up;
        let down = WipeDirection::Down;
        let _ = &down;
    }
}
