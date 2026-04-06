//! Slide transition effects.
//!
//! The primary API is [`Slide`] with [`SlideDirection`], a unified
//! `OneShotTransition` that slides the camera in any of four directions.

use bevy::prelude::*;

use super::super::shared::{ScreenSize, SlideStartEnd, TransitionProgress};
use crate::transition::{
    messages::{TransitionOver, TransitionReady, TransitionRunComplete},
    resources::StartingTransition,
    traits::{OneShotTransition, Transition},
};

// ---------------------------------------------------------------------------
// SlideDirection
// ---------------------------------------------------------------------------

/// Direction for slide transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SlideDirection {
    /// Slide content to the left (camera moves left, negative X).
    #[default]
    Left,
    /// Slide content to the right (camera moves right, positive X).
    Right,
    /// Slide content upward (camera moves up, positive Y).
    Up,
    /// Slide content downward (camera moves down, negative Y).
    Down,
}

// ---------------------------------------------------------------------------
// Slide (unified effect)
// ---------------------------------------------------------------------------

/// Slide content in the specified direction.
///
/// An `OneShotTransition` that animates the camera position by one screen
/// dimension in the given [`SlideDirection`].
pub struct Slide {
    /// Duration in seconds.
    pub duration: f32,
    /// Direction of the slide.
    pub direction: SlideDirection,
}

impl Default for Slide {
    fn default() -> Self {
        Self {
            duration: 0.3,
            direction: SlideDirection::Left,
        }
    }
}

impl Transition for Slide {
    fn insert_starting(&self, world: &mut World) {
        world.insert_resource(StartingTransition::<Self>::new());
        world.insert_resource(SlideConfig {
            duration: self.duration,
            direction: self.direction,
        });
    }
}
impl OneShotTransition for Slide {}

// ---------------------------------------------------------------------------
// SlideConfig (unified config)
// ---------------------------------------------------------------------------

/// Configuration resource inserted by [`Slide::insert_starting`].
#[derive(Resource)]
pub struct SlideConfig {
    /// Duration in seconds.
    pub duration: f32,
    /// Direction of the slide.
    pub direction: SlideDirection,
}

// ---------------------------------------------------------------------------
// Unified Slide systems
// ---------------------------------------------------------------------------

/// Start system for [`Slide`] -- records camera position and sends
/// `TransitionReady`.
pub(crate) fn slide_start(
    config: Res<SlideConfig>,
    screen: Res<ScreenSize>,
    cameras: Query<&Transform, With<Camera2d>>,
    mut writer: MessageWriter<TransitionReady>,
    mut commands: Commands,
) {
    let camera_pos = cameras
        .iter()
        .next()
        .map_or(Vec2::ZERO, |t| Vec2::new(t.translation.x, t.translation.y));

    let target = match config.direction {
        SlideDirection::Left => Vec2::new(camera_pos.x - screen.0.x, camera_pos.y),
        SlideDirection::Right => Vec2::new(camera_pos.x + screen.0.x, camera_pos.y),
        SlideDirection::Up => Vec2::new(camera_pos.x, camera_pos.y + screen.0.y),
        SlideDirection::Down => Vec2::new(camera_pos.x, camera_pos.y - screen.0.y),
    };

    commands.insert_resource(SlideStartEnd {
        start: camera_pos,
        target,
    });
    commands.insert_resource(TransitionProgress {
        elapsed: 0.0,
        duration: config.duration,
        completed: false,
    });
    commands.remove_resource::<SlideConfig>();
    writer.write(TransitionReady);
}

/// Run system for [`Slide`] -- lerps camera toward target.
pub(crate) fn slide_run(
    mut cameras: Query<&mut Transform, With<Camera2d>>,
    slide: Res<SlideStartEnd>,
    mut progress: ResMut<TransitionProgress>,
    time: Res<Time<Real>>,
    mut writer: MessageWriter<TransitionRunComplete>,
) {
    if progress.completed {
        return;
    }

    progress.elapsed += time.delta_secs();

    let t = if progress.duration > 0.0 {
        (progress.elapsed / progress.duration).clamp(0.0, 1.0)
    } else {
        1.0
    };

    let pos = slide.start.lerp(slide.target, t);
    for mut transform in &mut cameras {
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }

    if t >= 1.0 {
        progress.completed = true;
        writer.write(TransitionRunComplete);
    }
}

/// End system for [`Slide`] -- removes resources and sends `TransitionOver`.
pub(crate) fn slide_end(mut writer: MessageWriter<TransitionOver>, mut commands: Commands) {
    commands.remove_resource::<SlideStartEnd>();
    commands.remove_resource::<TransitionProgress>();
    writer.write(TransitionOver);
}
