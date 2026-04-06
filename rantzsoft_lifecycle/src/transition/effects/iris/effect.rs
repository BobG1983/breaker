//! Iris transition effects — `IrisOut` (`OutTransition`) and `IrisIn` (`InTransition`).
//!
//! Iris effects scale a single `Sprite` overlay from center. `IrisOut` grows from
//! zero to full screen (covering content). `IrisIn` shrinks from full screen to
//! zero (revealing content).

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

/// Start system for `IrisOut` — spawns overlay at zero scale and sends
/// `TransitionReady`.
pub(crate) fn iris_out_start(
    mut commands: Commands,
    config: Res<IrisOutConfig>,
    screen: Res<ScreenSize>,
    mut writer: MessageWriter<TransitionReady>,
) {
    commands.spawn((
        Sprite {
            color: config.color,
            custom_size: Some(screen.0),
            ..default()
        },
        Transform::from_scale(Vec3::ZERO),
        GlobalZIndex(i32::MAX - 1),
        TransitionOverlay,
    ));
    commands.insert_resource(TransitionProgress {
        elapsed: 0.0,
        duration: config.duration,
        completed: false,
    });
    commands.remove_resource::<IrisOutConfig>();
    writer.write(TransitionReady);
}

/// Run system for `IrisOut` — grows overlay scale.
pub(crate) fn iris_out_run(
    mut overlays: Query<&mut Transform, With<TransitionOverlay>>,
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

    for mut transform in &mut overlays {
        transform.scale = Vec3::splat(t);
    }

    if t >= 1.0 {
        progress.completed = true;
        writer.write(TransitionRunComplete);
    }
}

/// End system for `IrisOut` — despawns overlay and sends `TransitionOver`.
pub(crate) fn iris_out_end(
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

/// Start system for `IrisIn` — spawns overlay at full scale and sends
/// `TransitionReady`.
pub(crate) fn iris_in_start(
    mut commands: Commands,
    config: Res<IrisInConfig>,
    screen: Res<ScreenSize>,
    mut writer: MessageWriter<TransitionReady>,
) {
    commands.spawn((
        Sprite {
            color: config.color,
            custom_size: Some(screen.0),
            ..default()
        },
        Transform::from_scale(Vec3::ONE),
        GlobalZIndex(i32::MAX - 1),
        TransitionOverlay,
    ));
    commands.insert_resource(TransitionProgress {
        elapsed: 0.0,
        duration: config.duration,
        completed: false,
    });
    commands.remove_resource::<IrisInConfig>();
    writer.write(TransitionReady);
}

/// Run system for `IrisIn` — shrinks overlay scale.
pub(crate) fn iris_in_run(
    mut overlays: Query<&mut Transform, With<TransitionOverlay>>,
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

    for mut transform in &mut overlays {
        transform.scale = Vec3::splat((1.0 - t).max(0.0));
    }

    if t >= 1.0 {
        progress.completed = true;
        writer.write(TransitionRunComplete);
    }
}

/// End system for `IrisIn` — despawns overlay and sends `TransitionOver`.
pub(crate) fn iris_in_end(
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
