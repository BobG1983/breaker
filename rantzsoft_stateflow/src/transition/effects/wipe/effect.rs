//! Wipe transition effects — `WipeOut` (`OutTransition`) and `WipeIn` (`InTransition`).
//!
//! Wipe effects use a post-process shader with directional threshold wipe.

use bevy::prelude::*;

use super::super::{
    post_process::{EffectType, TransitionEffect, color_to_linear_vec4, wipe_direction_to_vec4},
    shared::{TransitionProgress, WipeDirection},
};
use crate::transition::{
    messages::{TransitionOver, TransitionReady, TransitionRunComplete},
    resources::StartingTransition,
    traits::{InTransition, OutTransition, Transition},
};

// ---------------------------------------------------------------------------
// Effect structs
// ---------------------------------------------------------------------------

/// Wipe overlay slides in to cover the screen.
pub struct WipeOut {
    /// Duration in seconds.
    pub duration:  f32,
    /// Overlay color.
    pub color:     Color,
    /// Direction the wipe slides from.
    pub direction: WipeDirection,
}

impl Default for WipeOut {
    fn default() -> Self {
        Self {
            duration:  0.3,
            color:     Color::BLACK,
            direction: WipeDirection::Left,
        }
    }
}

impl Transition for WipeOut {
    fn insert_starting(&self, world: &mut World) {
        world.insert_resource(StartingTransition::<Self>::new());
        world.insert_resource(WipeOutConfig {
            color:     self.color,
            duration:  self.duration,
            direction: self.direction,
        });
    }
}
impl OutTransition for WipeOut {}

/// Wipe overlay retracts off-screen to reveal content.
pub struct WipeIn {
    /// Duration in seconds.
    pub duration:  f32,
    /// Overlay color.
    pub color:     Color,
    /// Direction the wipe slides toward.
    pub direction: WipeDirection,
}

impl Default for WipeIn {
    fn default() -> Self {
        Self {
            duration:  0.3,
            color:     Color::BLACK,
            direction: WipeDirection::Left,
        }
    }
}

impl Transition for WipeIn {
    fn insert_starting(&self, world: &mut World) {
        world.insert_resource(StartingTransition::<Self>::new());
        world.insert_resource(WipeInConfig {
            color:     self.color,
            duration:  self.duration,
            direction: self.direction,
        });
    }
}
impl InTransition for WipeIn {}

// ---------------------------------------------------------------------------
// Config resources
// ---------------------------------------------------------------------------

/// Configuration resource inserted by `WipeOut::insert_starting`.
#[derive(Resource)]
pub struct WipeOutConfig {
    /// Duration in seconds.
    pub duration:  f32,
    /// Overlay color.
    pub color:     Color,
    /// Direction the wipe slides from.
    pub direction: WipeDirection,
}

/// Configuration resource inserted by `WipeIn::insert_starting`.
#[derive(Resource)]
pub struct WipeInConfig {
    /// Duration in seconds.
    pub duration:  f32,
    /// Overlay color.
    pub color:     Color,
    /// Direction the wipe slides toward.
    pub direction: WipeDirection,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Start system for `WipeOut` — inserts `TransitionEffect` on camera with wipe
/// direction and sends `TransitionReady`.
pub(crate) fn wipe_out_start(
    mut commands: Commands,
    config: Res<WipeOutConfig>,
    cameras: Query<Entity, With<Camera2d>>,
    mut writer: MessageWriter<TransitionReady>,
) {
    if let Some(camera) = cameras.iter().next() {
        commands.entity(camera).insert(TransitionEffect {
            color:       color_to_linear_vec4(config.color),
            direction:   wipe_direction_to_vec4(&config.direction),
            effect_type: EffectType::WIPE,
            progress:    0.0,
        });
    }
    commands.insert_resource(TransitionProgress {
        elapsed:   0.0,
        duration:  config.duration,
        completed: false,
    });
    commands.remove_resource::<WipeOutConfig>();
    writer.write(TransitionReady);
}

/// Run system for `WipeOut` — increases `TransitionEffect.progress`.
pub(crate) fn wipe_out_run(
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

/// End system for `WipeOut` — removes `TransitionEffect` from camera and sends
/// `TransitionOver`.
pub(crate) fn wipe_out_end(
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

/// Start system for `WipeIn` — inserts `TransitionEffect` at full progress on
/// camera and sends `TransitionReady`.
pub(crate) fn wipe_in_start(
    mut commands: Commands,
    config: Res<WipeInConfig>,
    cameras: Query<Entity, With<Camera2d>>,
    mut writer: MessageWriter<TransitionReady>,
) {
    if let Some(camera) = cameras.iter().next() {
        commands.entity(camera).insert(TransitionEffect {
            color:       color_to_linear_vec4(config.color),
            direction:   wipe_direction_to_vec4(&config.direction),
            effect_type: EffectType::WIPE,
            progress:    1.0,
        });
    }
    commands.insert_resource(TransitionProgress {
        elapsed:   0.0,
        duration:  config.duration,
        completed: false,
    });
    commands.remove_resource::<WipeInConfig>();
    writer.write(TransitionReady);
}

/// Run system for `WipeIn` — decreases `TransitionEffect.progress`.
pub(crate) fn wipe_in_run(
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

/// End system for `WipeIn` — removes `TransitionEffect` from camera and sends
/// `TransitionOver`.
pub(crate) fn wipe_in_end(
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
