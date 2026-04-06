//! Pixelate transition effects — `PixelateOut` (`OutTransition`) and `PixelateIn`
//! (`InTransition`).
//!
//! Pixelate effects use a single `Sprite` with a block-step alpha animation
//! curve, distinct from dissolve.

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
// Helpers
// ---------------------------------------------------------------------------

/// Quantize linear progress into block-step alpha for pixelate visual.
fn pixelate_alpha(progress: f32) -> f32 {
    let steps = 5.0;
    (progress * steps).ceil() / steps
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Start system for `PixelateOut` — spawns overlay at zero alpha and sends
/// `TransitionReady`.
pub(crate) fn pixelate_out_start(
    mut commands: Commands,
    config: Res<PixelateOutConfig>,
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
    commands.remove_resource::<PixelateOutConfig>();
    writer.write(TransitionReady);
}

/// Run system for `PixelateOut` — increases alpha with block-step curve.
pub(crate) fn pixelate_out_run(
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

    let alpha = pixelate_alpha(t);
    for mut sprite in &mut overlays {
        sprite.color = sprite.color.with_alpha(alpha);
    }

    if t >= 1.0 {
        progress.completed = true;
        writer.write(TransitionRunComplete);
    }
}

/// End system for `PixelateOut` — despawns overlay and sends `TransitionOver`.
pub(crate) fn pixelate_out_end(
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

/// Start system for `PixelateIn` — spawns fully opaque overlay and sends
/// `TransitionReady`.
pub(crate) fn pixelate_in_start(
    mut commands: Commands,
    config: Res<PixelateInConfig>,
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
    commands.remove_resource::<PixelateInConfig>();
    writer.write(TransitionReady);
}

/// Run system for `PixelateIn` — decreases alpha with block-step curve.
pub(crate) fn pixelate_in_run(
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

    let alpha = (1.0 - pixelate_alpha(t)).max(0.0);
    for mut sprite in &mut overlays {
        sprite.color = sprite.color.with_alpha(alpha);
    }

    if t >= 1.0 {
        progress.completed = true;
        writer.write(TransitionRunComplete);
    }
}

/// End system for `PixelateIn` — despawns overlay and sends `TransitionOver`.
pub(crate) fn pixelate_in_end(
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
