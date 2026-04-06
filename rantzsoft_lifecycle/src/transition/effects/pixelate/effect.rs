//! Pixelate transition effects — `PixelateOut` (`OutTransition`) and `PixelateIn`
//! (`InTransition`).
//!
//! Pixelate effects use a post-process shader with UV grid snap.

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

/// Pixelate overlay covers the screen with a block-step curve (hides content).
pub struct PixelateOut {
    /// Duration in seconds.
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
}

impl Default for PixelateOut {
    fn default() -> Self {
        Self {
            duration: 0.3,
            color: Color::BLACK,
        }
    }
}

impl Transition for PixelateOut {
    fn insert_starting(&self, world: &mut World) {
        world.insert_resource(StartingTransition::<Self>::new());
        world.insert_resource(PixelateOutConfig {
            color: self.color,
            duration: self.duration,
        });
    }
}
impl OutTransition for PixelateOut {}

/// Pixelate overlay reveals the screen with a block-step curve (reveals
/// content).
pub struct PixelateIn {
    /// Duration in seconds.
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
}

impl Default for PixelateIn {
    fn default() -> Self {
        Self {
            duration: 0.3,
            color: Color::BLACK,
        }
    }
}

impl Transition for PixelateIn {
    fn insert_starting(&self, world: &mut World) {
        world.insert_resource(StartingTransition::<Self>::new());
        world.insert_resource(PixelateInConfig {
            color: self.color,
            duration: self.duration,
        });
    }
}
impl InTransition for PixelateIn {}

// ---------------------------------------------------------------------------
// Config resources
// ---------------------------------------------------------------------------

/// Configuration resource inserted by `PixelateOut::insert_starting`.
#[derive(Resource)]
pub struct PixelateOutConfig {
    /// Duration in seconds.
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
}

/// Configuration resource inserted by `PixelateIn::insert_starting`.
#[derive(Resource)]
pub struct PixelateInConfig {
    /// Duration in seconds.
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Start system for `PixelateOut` — inserts `TransitionEffect` on camera and
/// sends `TransitionReady`.
pub(crate) fn pixelate_out_start(
    mut commands: Commands,
    config: Res<PixelateOutConfig>,
    cameras: Query<Entity, With<Camera2d>>,
    mut writer: MessageWriter<TransitionReady>,
) {
    if let Some(camera) = cameras.iter().next() {
        commands.entity(camera).insert(TransitionEffect {
            color: color_to_linear_vec4(config.color),
            direction: Vec4::ZERO,
            effect_type: EffectType::PIXELATE,
            progress: 0.0,
        });
    }
    commands.insert_resource(TransitionProgress {
        elapsed: 0.0,
        duration: config.duration,
        completed: false,
    });
    commands.remove_resource::<PixelateOutConfig>();
    writer.write(TransitionReady);
}

/// Run system for `PixelateOut` — increases `TransitionEffect.progress`.
pub(crate) fn pixelate_out_run(
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

/// End system for `PixelateOut` — removes `TransitionEffect` from camera and
/// sends `TransitionOver`.
pub(crate) fn pixelate_out_end(
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

/// Start system for `PixelateIn` — inserts fully opaque `TransitionEffect` on
/// camera and sends `TransitionReady`.
pub(crate) fn pixelate_in_start(
    mut commands: Commands,
    config: Res<PixelateInConfig>,
    cameras: Query<Entity, With<Camera2d>>,
    mut writer: MessageWriter<TransitionReady>,
) {
    if let Some(camera) = cameras.iter().next() {
        commands.entity(camera).insert(TransitionEffect {
            color: color_to_linear_vec4(config.color),
            direction: Vec4::ZERO,
            effect_type: EffectType::PIXELATE,
            progress: 1.0,
        });
    }
    commands.insert_resource(TransitionProgress {
        elapsed: 0.0,
        duration: config.duration,
        completed: false,
    });
    commands.remove_resource::<PixelateInConfig>();
    writer.write(TransitionReady);
}

/// Run system for `PixelateIn` — decreases `TransitionEffect.progress`.
pub(crate) fn pixelate_in_run(
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

/// End system for `PixelateIn` — removes `TransitionEffect` from camera and
/// sends `TransitionOver`.
pub(crate) fn pixelate_in_end(
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
