//! Iris transition effects — `IrisOut` (`OutTransition`) and `IrisIn` (`InTransition`).
//!
//! Iris effects use a post-process shader with circle mask from center.

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

/// Iris overlay scales from zero to full screen (hides content).
pub struct IrisOut {
    /// Duration in seconds.
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
}

impl Default for IrisOut {
    fn default() -> Self {
        Self {
            duration: 0.3,
            color: Color::BLACK,
        }
    }
}

impl Transition for IrisOut {
    fn insert_starting(&self, world: &mut World) {
        world.insert_resource(StartingTransition::<Self>::new());
        world.insert_resource(IrisOutConfig {
            color: self.color,
            duration: self.duration,
        });
    }
}
impl OutTransition for IrisOut {}

/// Iris overlay scales from full screen to zero (reveals content).
pub struct IrisIn {
    /// Duration in seconds.
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
}

impl Default for IrisIn {
    fn default() -> Self {
        Self {
            duration: 0.3,
            color: Color::BLACK,
        }
    }
}

impl Transition for IrisIn {
    fn insert_starting(&self, world: &mut World) {
        world.insert_resource(StartingTransition::<Self>::new());
        world.insert_resource(IrisInConfig {
            color: self.color,
            duration: self.duration,
        });
    }
}
impl InTransition for IrisIn {}

// ---------------------------------------------------------------------------
// Config resources
// ---------------------------------------------------------------------------

/// Configuration resource inserted by `IrisOut::insert_starting`.
#[derive(Resource)]
pub struct IrisOutConfig {
    /// Duration in seconds.
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
}

/// Configuration resource inserted by `IrisIn::insert_starting`.
#[derive(Resource)]
pub struct IrisInConfig {
    /// Duration in seconds.
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Start system for `IrisOut` — inserts `TransitionEffect` on camera and sends
/// `TransitionReady`.
pub(crate) fn iris_out_start(
    mut commands: Commands,
    config: Res<IrisOutConfig>,
    cameras: Query<Entity, With<Camera2d>>,
    mut writer: MessageWriter<TransitionReady>,
) {
    if let Some(camera) = cameras.iter().next() {
        commands.entity(camera).insert(TransitionEffect {
            color: color_to_linear_vec4(config.color),
            direction: Vec4::ZERO,
            effect_type: EffectType::IRIS,
            progress: 0.0,
        });
    }
    commands.insert_resource(TransitionProgress {
        elapsed: 0.0,
        duration: config.duration,
        completed: false,
    });
    commands.remove_resource::<IrisOutConfig>();
    writer.write(TransitionReady);
}

/// Run system for `IrisOut` — increases `TransitionEffect.progress`.
pub(crate) fn iris_out_run(
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

/// End system for `IrisOut` — removes `TransitionEffect` from camera and sends
/// `TransitionOver`.
pub(crate) fn iris_out_end(
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

/// Start system for `IrisIn` — inserts `TransitionEffect` at full progress on
/// camera and sends `TransitionReady`.
pub(crate) fn iris_in_start(
    mut commands: Commands,
    config: Res<IrisInConfig>,
    cameras: Query<Entity, With<Camera2d>>,
    mut writer: MessageWriter<TransitionReady>,
) {
    if let Some(camera) = cameras.iter().next() {
        commands.entity(camera).insert(TransitionEffect {
            color: color_to_linear_vec4(config.color),
            direction: Vec4::ZERO,
            effect_type: EffectType::IRIS,
            progress: 1.0,
        });
    }
    commands.insert_resource(TransitionProgress {
        elapsed: 0.0,
        duration: config.duration,
        completed: false,
    });
    commands.remove_resource::<IrisInConfig>();
    writer.write(TransitionReady);
}

/// Run system for `IrisIn` — decreases `TransitionEffect.progress`.
pub(crate) fn iris_in_run(
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

/// End system for `IrisIn` — removes `TransitionEffect` from camera and sends
/// `TransitionOver`.
pub(crate) fn iris_in_end(
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
