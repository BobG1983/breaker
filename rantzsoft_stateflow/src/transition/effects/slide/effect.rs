//! Slide transition effects.
//!
//! The primary API is [`Slide`] with [`SlideDirection`], a unified
//! `OneShotTransition` that uses a post-process shader with UV offset.

use bevy::prelude::*;

use super::super::{
    post_process::{EffectType, TransitionEffect, slide_direction_to_vec4},
    shared::TransitionProgress,
};
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
/// An `OneShotTransition` that animates a UV offset shader effect in the given
/// [`SlideDirection`].
pub struct Slide {
    /// Duration in seconds.
    pub duration:  f32,
    /// Direction of the slide.
    pub direction: SlideDirection,
}

impl Default for Slide {
    fn default() -> Self {
        Self {
            duration:  0.3,
            direction: SlideDirection::Left,
        }
    }
}

impl Transition for Slide {
    fn insert_starting(&self, world: &mut World) {
        world.insert_resource(StartingTransition::<Self>::new());
        world.insert_resource(SlideConfig {
            duration:  self.duration,
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
    pub duration:  f32,
    /// Direction of the slide.
    pub direction: SlideDirection,
}

// ---------------------------------------------------------------------------
// Unified Slide systems
// ---------------------------------------------------------------------------

/// Start system for [`Slide`] — inserts `TransitionEffect` on camera and sends
/// `TransitionReady`.
pub(crate) fn slide_start(
    config: Res<SlideConfig>,
    cameras: Query<Entity, With<Camera2d>>,
    mut writer: MessageWriter<TransitionReady>,
    mut commands: Commands,
) {
    if let Some(camera) = cameras.iter().next() {
        commands.entity(camera).insert(TransitionEffect {
            color:       Vec4::ZERO,
            direction:   slide_direction_to_vec4(&config.direction),
            effect_type: EffectType::SLIDE,
            progress:    0.0,
        });
    }
    commands.insert_resource(TransitionProgress {
        elapsed:   0.0,
        duration:  config.duration,
        completed: false,
    });
    commands.remove_resource::<SlideConfig>();
    writer.write(TransitionReady);
}

/// Run system for [`Slide`] — updates `TransitionEffect.progress`.
pub(crate) fn slide_run(
    mut effects: Query<&mut TransitionEffect>,
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

    for mut effect in &mut effects {
        effect.progress = t;
    }

    if t >= 1.0 {
        progress.completed = true;
        writer.write(TransitionRunComplete);
    }
}

/// End system for [`Slide`] — removes `TransitionEffect` from camera and sends
/// `TransitionOver`.
pub(crate) fn slide_end(
    mut writer: MessageWriter<TransitionOver>,
    mut commands: Commands,
    cameras: Query<Entity, With<Camera2d>>,
) {
    if let Some(camera) = cameras.iter().next() {
        commands.entity(camera).remove::<TransitionEffect>();
    }
    commands.remove_resource::<TransitionProgress>();
    writer.write(TransitionOver);
}
