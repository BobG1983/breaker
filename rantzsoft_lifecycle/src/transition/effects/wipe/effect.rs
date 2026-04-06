//! Wipe transition effects — `WipeOut` (`OutTransition`) and `WipeIn` (`InTransition`).
//!
//! Wipe effects use a single `Sprite` that slides from off-screen to cover
//! (`WipeOut`) or retracts off-screen to reveal (`WipeIn`).

use bevy::prelude::*;

use super::super::shared::{ScreenSize, TransitionOverlay, TransitionProgress, WipeDirection};
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
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
    /// Direction the wipe slides from.
    pub direction: WipeDirection,
}

impl Default for WipeOut {
    fn default() -> Self {
        Self {
            duration: 0.3,
            color: Color::BLACK,
            direction: WipeDirection::Left,
        }
    }
}

impl Transition for WipeOut {
    fn insert_starting(&self, world: &mut World) {
        world.insert_resource(StartingTransition::<Self>::new());
        world.insert_resource(WipeOutConfig {
            color: self.color,
            duration: self.duration,
            direction: self.direction,
        });
    }
}
impl OutTransition for WipeOut {}

/// Wipe overlay retracts off-screen to reveal content.
pub struct WipeIn {
    /// Duration in seconds.
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
    /// Direction the wipe slides toward.
    pub direction: WipeDirection,
}

impl Default for WipeIn {
    fn default() -> Self {
        Self {
            duration: 0.3,
            color: Color::BLACK,
            direction: WipeDirection::Left,
        }
    }
}

impl Transition for WipeIn {
    fn insert_starting(&self, world: &mut World) {
        world.insert_resource(StartingTransition::<Self>::new());
        world.insert_resource(WipeInConfig {
            color: self.color,
            duration: self.duration,
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
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
    /// Direction the wipe slides from.
    pub direction: WipeDirection,
}

/// Configuration resource inserted by `WipeIn::insert_starting`.
#[derive(Resource)]
pub struct WipeInConfig {
    /// Duration in seconds.
    pub duration: f32,
    /// Overlay color.
    pub color: Color,
    /// Direction the wipe slides toward.
    pub direction: WipeDirection,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compute the off-screen start position for a wipe-out animation.
fn wipe_out_start_pos(direction: WipeDirection, screen: Vec2) -> Vec3 {
    match direction {
        WipeDirection::Left => Vec3::new(screen.x, 0.0, 0.0),
        WipeDirection::Right => Vec3::new(-screen.x, 0.0, 0.0),
        WipeDirection::Up => Vec3::new(0.0, -screen.y, 0.0),
        WipeDirection::Down => Vec3::new(0.0, screen.y, 0.0),
    }
}

/// Compute the off-screen end position for a wipe-in animation.
fn wipe_in_end_pos(direction: WipeDirection, screen: Vec2) -> Vec3 {
    match direction {
        WipeDirection::Left => Vec3::new(-screen.x, 0.0, 0.0),
        WipeDirection::Right => Vec3::new(screen.x, 0.0, 0.0),
        WipeDirection::Up => Vec3::new(0.0, screen.y, 0.0),
        WipeDirection::Down => Vec3::new(0.0, -screen.y, 0.0),
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Start system for `WipeOut` — spawns off-screen overlay and sends
/// `TransitionReady`.
pub(crate) fn wipe_out_start(
    mut commands: Commands,
    config: Res<WipeOutConfig>,
    screen: Res<ScreenSize>,
    mut writer: MessageWriter<TransitionReady>,
) {
    let start_pos = wipe_out_start_pos(config.direction, screen.0);
    commands.spawn((
        Sprite {
            color: config.color,
            custom_size: Some(screen.0),
            ..default()
        },
        Transform::from_translation(start_pos),
        GlobalZIndex(i32::MAX - 1),
        TransitionOverlay,
    ));
    commands.insert_resource(TransitionProgress {
        elapsed: 0.0,
        duration: config.duration,
        completed: false,
    });
    writer.write(TransitionReady);
}

/// Run system for `WipeOut` — slides overlay across screen.
pub(crate) fn wipe_out_run(
    mut overlays: Query<&mut Transform, With<TransitionOverlay>>,
    config: Res<WipeOutConfig>,
    screen: Res<ScreenSize>,
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

    let start_pos = wipe_out_start_pos(config.direction, screen.0);
    let end_pos = Vec3::ZERO;

    for mut transform in &mut overlays {
        transform.translation = start_pos.lerp(end_pos, t);
    }

    if t >= 1.0 {
        progress.completed = true;
        writer.write(TransitionRunComplete);
    }
}

/// End system for `WipeOut` — despawns overlay and sends `TransitionOver`.
pub(crate) fn wipe_out_end(
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

/// Start system for `WipeIn` — spawns full-coverage overlay and sends
/// `TransitionReady`.
pub(crate) fn wipe_in_start(
    mut commands: Commands,
    config: Res<WipeInConfig>,
    screen: Res<ScreenSize>,
    mut writer: MessageWriter<TransitionReady>,
) {
    commands.spawn((
        Sprite {
            color: config.color,
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
    writer.write(TransitionReady);
}

/// Run system for `WipeIn` — slides overlay off-screen.
pub(crate) fn wipe_in_run(
    mut overlays: Query<&mut Transform, With<TransitionOverlay>>,
    config: Res<WipeInConfig>,
    screen: Res<ScreenSize>,
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

    let start_pos = Vec3::ZERO;
    let end_pos = wipe_in_end_pos(config.direction, screen.0);

    for mut transform in &mut overlays {
        transform.translation = start_pos.lerp(end_pos, t);
    }

    if t >= 1.0 {
        progress.completed = true;
        writer.write(TransitionRunComplete);
    }
}

/// End system for `WipeIn` — despawns overlay and sends `TransitionOver`.
pub(crate) fn wipe_in_end(
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
