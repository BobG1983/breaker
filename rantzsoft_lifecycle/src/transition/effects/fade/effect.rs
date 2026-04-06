//! Fade transition effects — `FadeOut` (`OutTransition`) and `FadeIn` (`InTransition`).

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

/// Start system for `FadeOut` — spawns overlay sprite and sends `TransitionReady`.
pub(crate) fn fade_out_start(
    mut commands: Commands,
    config: Res<FadeOutConfig>,
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
    commands.remove_resource::<FadeOutConfig>();
    writer.write(TransitionReady);
}

/// Run system for `FadeOut` — advances overlay alpha based on progress.
pub(crate) fn fade_out_run(
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

    for mut sprite in &mut overlays {
        sprite.color = sprite.color.with_alpha(t);
    }

    if t >= 1.0 {
        progress.completed = true;
        writer.write(TransitionRunComplete);
    }
}

/// End system for `FadeOut` — despawns overlay, removes progress, sends
/// `TransitionOver`.
pub(crate) fn fade_out_end(
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

/// Start system for `FadeIn` — spawns overlay at full opacity and sends
/// `TransitionReady`.
pub(crate) fn fade_in_start(
    mut commands: Commands,
    config: Res<FadeInConfig>,
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
    commands.remove_resource::<FadeInConfig>();
    writer.write(TransitionReady);
}

/// Run system for `FadeIn` — decreases overlay alpha based on progress.
pub(crate) fn fade_in_run(
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

    for mut sprite in &mut overlays {
        sprite.color = sprite.color.with_alpha((1.0 - t).max(0.0));
    }

    if t >= 1.0 {
        progress.completed = true;
        writer.write(TransitionRunComplete);
    }
}

/// End system for `FadeIn` — despawns overlay, removes progress, sends
/// `TransitionOver`.
pub(crate) fn fade_in_end(
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
