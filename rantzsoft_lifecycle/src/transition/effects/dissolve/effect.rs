//! Dissolve transition effects — `DissolveOut` (`OutTransition`) and `DissolveIn`
//! (`InTransition`).
//!
//! Dissolve effects use a single `Sprite` with a non-linear (stepped/dithered)
//! alpha animation curve.

use bevy::prelude::*;

use super::super::shared::{ScreenSize, TransitionOverlay, TransitionProgress};
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
// Helpers
// ---------------------------------------------------------------------------

/// Quantize linear progress into discrete steps for dissolve visual.
fn dissolve_alpha(progress: f32) -> f32 {
    let steps = 10.0;
    (progress * steps).ceil() / steps
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Start system for `DissolveOut` — spawns overlay at zero alpha and sends
/// `TransitionReady`.
pub(crate) fn dissolve_out_start(
    mut commands: Commands,
    config: Res<DissolveOutConfig>,
    screen: Res<ScreenSize>,
    mut writer: MessageWriter<TransitionReady>,
) {
    commands.spawn((
        Sprite {
            color: config.color.with_alpha(0.0),
            custom_size: Some(screen.0),
            ..default()
        },
        Transform::from_translation(Vec3::ZERO),
        GlobalZIndex(i32::MAX - 1),
        TransitionOverlay,
    ));
    commands.insert_resource(TransitionProgress {
        elapsed: 0.0,
        duration: config.duration,
        completed: false,
    });
    commands.remove_resource::<DissolveOutConfig>();
    writer.write(TransitionReady);
}

/// Run system for `DissolveOut` — increases alpha with stepped curve.
pub(crate) fn dissolve_out_run(
    mut overlays: Query<&mut Sprite, With<TransitionOverlay>>,
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

    let alpha = dissolve_alpha(t);
    for mut sprite in &mut overlays {
        sprite.color = sprite.color.with_alpha(alpha);
    }

    if t >= 1.0 {
        progress.completed = true;
        writer.write(TransitionRunComplete);
    }
}

/// End system for `DissolveOut` — despawns overlay and sends `TransitionOver`.
pub(crate) fn dissolve_out_end(
    mut commands: Commands,
    overlays: Query<Entity, With<TransitionOverlay>>,
    mut writer: MessageWriter<TransitionOver>,
) {
    for entity in &overlays {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<TransitionProgress>();
    writer.write(TransitionOver);
}

/// Start system for `DissolveIn` — spawns fully opaque overlay and sends
/// `TransitionReady`.
pub(crate) fn dissolve_in_start(
    mut commands: Commands,
    config: Res<DissolveInConfig>,
    screen: Res<ScreenSize>,
    mut writer: MessageWriter<TransitionReady>,
) {
    commands.spawn((
        Sprite {
            color: config.color.with_alpha(1.0),
            custom_size: Some(screen.0),
            ..default()
        },
        Transform::from_translation(Vec3::ZERO),
        GlobalZIndex(i32::MAX - 1),
        TransitionOverlay,
    ));
    commands.insert_resource(TransitionProgress {
        elapsed: 0.0,
        duration: config.duration,
        completed: false,
    });
    commands.remove_resource::<DissolveInConfig>();
    writer.write(TransitionReady);
}

/// Run system for `DissolveIn` — decreases alpha with stepped curve.
pub(crate) fn dissolve_in_run(
    mut overlays: Query<&mut Sprite, With<TransitionOverlay>>,
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

    let alpha = (1.0 - dissolve_alpha(t)).max(0.0);
    for mut sprite in &mut overlays {
        sprite.color = sprite.color.with_alpha(alpha);
    }

    if t >= 1.0 {
        progress.completed = true;
        writer.write(TransitionRunComplete);
    }
}

/// End system for `DissolveIn` — despawns overlay and sends `TransitionOver`.
pub(crate) fn dissolve_in_end(
    mut commands: Commands,
    overlays: Query<Entity, With<TransitionOverlay>>,
    mut writer: MessageWriter<TransitionOver>,
) {
    for entity in &overlays {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<TransitionProgress>();
    writer.write(TransitionOver);
}
