//! Fade transition effects — `FadeOut` (`OutTransition`) and `FadeIn` (`InTransition`).

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

/// Fade from transparent overlay to opaque overlay (hides content).
pub struct FadeOut {
    /// Duration in seconds.
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
}

impl Default for FadeOut {
    fn default() -> Self {
        Self {
            duration: 0.3,
            color: Color::BLACK,
        }
    }
}

impl Transition for FadeOut {
    fn insert_starting(&self, world: &mut World) {
        world.insert_resource(StartingTransition::<Self>::new());
        world.insert_resource(FadeOutConfig {
            color: self.color,
            duration: self.duration,
        });
    }
}
impl OutTransition for FadeOut {}

/// Fade from opaque overlay to transparent overlay (reveals content).
pub struct FadeIn {
    /// Duration in seconds.
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
}

impl Default for FadeIn {
    fn default() -> Self {
        Self {
            duration: 0.3,
            color: Color::BLACK,
        }
    }
}

impl Transition for FadeIn {
    fn insert_starting(&self, world: &mut World) {
        world.insert_resource(StartingTransition::<Self>::new());
        world.insert_resource(FadeInConfig {
            color: self.color,
            duration: self.duration,
        });
    }
}
impl InTransition for FadeIn {}

// ---------------------------------------------------------------------------
// Config resources
// ---------------------------------------------------------------------------

/// Configuration resource inserted by `FadeOut::insert_starting`.
#[derive(Resource)]
pub struct FadeOutConfig {
    /// Duration in seconds.
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
}

/// Configuration resource inserted by `FadeIn::insert_starting`.
#[derive(Resource)]
pub struct FadeInConfig {
    /// Duration in seconds.
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Start system for `FadeOut` — inserts `TransitionEffect` on camera and sends
/// `TransitionReady`.
pub(crate) fn fade_out_start(
    mut commands: Commands,
    config: Res<FadeOutConfig>,
    cameras: Query<Entity, With<Camera2d>>,
    mut writer: MessageWriter<TransitionReady>,
) {
    if let Some(camera) = cameras.iter().next() {
        commands.entity(camera).insert(TransitionEffect {
            color: color_to_linear_vec4(config.color),
            direction: Vec4::ZERO,
            effect_type: EffectType::FADE,
            progress: 0.0,
        });
    }
    commands.insert_resource(TransitionProgress {
        elapsed: 0.0,
        duration: config.duration,
        completed: false,
    });
    commands.remove_resource::<FadeOutConfig>();
    writer.write(TransitionReady);
}

/// Run system for `FadeOut` — advances `TransitionEffect.progress` based on
/// elapsed time.
pub(crate) fn fade_out_run(
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

/// End system for `FadeOut` — removes `TransitionEffect` from camera, removes
/// progress, sends `TransitionOver`.
pub(crate) fn fade_out_end(
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

/// Start system for `FadeIn` — inserts `TransitionEffect` at full progress on
/// camera and sends `TransitionReady`.
pub(crate) fn fade_in_start(
    mut commands: Commands,
    config: Res<FadeInConfig>,
    cameras: Query<Entity, With<Camera2d>>,
    mut writer: MessageWriter<TransitionReady>,
) {
    if let Some(camera) = cameras.iter().next() {
        commands.entity(camera).insert(TransitionEffect {
            color: color_to_linear_vec4(config.color),
            direction: Vec4::ZERO,
            effect_type: EffectType::FADE,
            progress: 1.0,
        });
    }
    commands.insert_resource(TransitionProgress {
        elapsed: 0.0,
        duration: config.duration,
        completed: false,
    });
    commands.remove_resource::<FadeInConfig>();
    writer.write(TransitionReady);
}

/// Run system for `FadeIn` — decreases `TransitionEffect.progress` based on
/// elapsed time.
pub(crate) fn fade_in_run(
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

/// End system for `FadeIn` — removes `TransitionEffect` from camera, removes
/// progress, sends `TransitionOver`.
pub(crate) fn fade_in_end(
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
