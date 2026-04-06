//! Dissolve transition effects — `DissolveOut` (`OutTransition`) and `DissolveIn`
//! (`InTransition`).
//!
//! Dissolve effects use a post-process shader with noise-threshold dissolve.

use bevy::prelude::*;

use super::super::{
    post_process::{EffectType, TransitionEffect, color_to_linear_vec4},
    shared::TransitionProgress,
};
use crate::transition::{
    messages::{TransitionOver, TransitionReady, TransitionRunComplete},
    resources::StartingTransition,
    traits::{InTransition, OutTransition, Transition},
};

// ---------------------------------------------------------------------------
// Effect structs
// ---------------------------------------------------------------------------

/// Dissolve overlay fades in with a non-linear curve (hides content).
pub struct DissolveOut {
    /// Duration in seconds.
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
}

impl Default for DissolveOut {
    fn default() -> Self {
        Self {
            duration: 0.3,
            color: Color::BLACK,
        }
    }
}

impl Transition for DissolveOut {
    fn insert_starting(&self, world: &mut World) {
        world.insert_resource(StartingTransition::<Self>::new());
        world.insert_resource(DissolveOutConfig {
            color: self.color,
            duration: self.duration,
        });
    }
}
impl OutTransition for DissolveOut {}

/// Dissolve overlay fades out with a non-linear curve (reveals content).
pub struct DissolveIn {
    /// Duration in seconds.
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
}

impl Default for DissolveIn {
    fn default() -> Self {
        Self {
            duration: 0.3,
            color: Color::BLACK,
        }
    }
}

impl Transition for DissolveIn {
    fn insert_starting(&self, world: &mut World) {
        world.insert_resource(StartingTransition::<Self>::new());
        world.insert_resource(DissolveInConfig {
            color: self.color,
            duration: self.duration,
        });
    }
}
impl InTransition for DissolveIn {}

// ---------------------------------------------------------------------------
// Config resources
// ---------------------------------------------------------------------------

/// Configuration resource inserted by `DissolveOut::insert_starting`.
#[derive(Resource)]
pub struct DissolveOutConfig {
    /// Duration in seconds.
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
}

/// Configuration resource inserted by `DissolveIn::insert_starting`.
#[derive(Resource)]
pub struct DissolveInConfig {
    /// Duration in seconds.
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Start system for `DissolveOut` — inserts `TransitionEffect` on camera and
/// sends `TransitionReady`.
pub(crate) fn dissolve_out_start(
    mut commands: Commands,
    config: Res<DissolveOutConfig>,
    cameras: Query<Entity, With<Camera2d>>,
    mut writer: MessageWriter<TransitionReady>,
) {
    if let Some(camera) = cameras.iter().next() {
        commands.entity(camera).insert(TransitionEffect {
            color: color_to_linear_vec4(config.color),
            direction: Vec4::ZERO,
            effect_type: EffectType::DISSOLVE,
            progress: 0.0,
        });
    }
    commands.insert_resource(TransitionProgress {
        elapsed: 0.0,
        duration: config.duration,
        completed: false,
    });
    commands.remove_resource::<DissolveOutConfig>();
    writer.write(TransitionReady);
}

/// Run system for `DissolveOut` — increases `TransitionEffect.progress`.
pub(crate) fn dissolve_out_run(
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

/// End system for `DissolveOut` — removes `TransitionEffect` from camera and
/// sends `TransitionOver`.
pub(crate) fn dissolve_out_end(
    mut commands: Commands,
    cameras: Query<Entity, With<Camera2d>>,
    mut writer: MessageWriter<TransitionOver>,
) {
    if let Some(camera) = cameras.iter().next() {
        commands.entity(camera).remove::<TransitionEffect>();
    }
    commands.remove_resource::<TransitionProgress>();
    writer.write(TransitionOver);
}

/// Start system for `DissolveIn` — inserts fully opaque `TransitionEffect` on
/// camera and sends `TransitionReady`.
pub(crate) fn dissolve_in_start(
    mut commands: Commands,
    config: Res<DissolveInConfig>,
    cameras: Query<Entity, With<Camera2d>>,
    mut writer: MessageWriter<TransitionReady>,
) {
    if let Some(camera) = cameras.iter().next() {
        commands.entity(camera).insert(TransitionEffect {
            color: color_to_linear_vec4(config.color),
            direction: Vec4::ZERO,
            effect_type: EffectType::DISSOLVE,
            progress: 1.0,
        });
    }
    commands.insert_resource(TransitionProgress {
        elapsed: 0.0,
        duration: config.duration,
        completed: false,
    });
    commands.remove_resource::<DissolveInConfig>();
    writer.write(TransitionReady);
}

/// Run system for `DissolveIn` — decreases `TransitionEffect.progress`.
pub(crate) fn dissolve_in_run(
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
        effect.progress = (1.0 - t).max(0.0);
    }

    if t >= 1.0 {
        progress.completed = true;
        writer.write(TransitionRunComplete);
    }
}

/// End system for `DissolveIn` — removes `TransitionEffect` from camera and
/// sends `TransitionOver`.
pub(crate) fn dissolve_in_end(
    mut commands: Commands,
    cameras: Query<Entity, With<Camera2d>>,
    mut writer: MessageWriter<TransitionOver>,
) {
    if let Some(camera) = cameras.iter().next() {
        commands.entity(camera).remove::<TransitionEffect>();
    }
    commands.remove_resource::<TransitionProgress>();
    writer.write(TransitionOver);
}
