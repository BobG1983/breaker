//! Pixelate transition effects — `PixelateOut` (`OutTransition`) and `PixelateIn`
//! (`InTransition`).
//!
//! Pixelate effects use a single `Sprite` with a block-step alpha animation
//! curve, distinct from dissolve.

use bevy::prelude::*;

use super::shared::{ScreenSize, TransitionOverlay, TransitionProgress};
#[cfg(test)]
use crate::transition::resources::{EndingTransition, RunningTransition};
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn effect_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<TransitionReady>();
        app.add_message::<TransitionRunComplete>();
        app.add_message::<TransitionOver>();
        app.insert_resource(ScreenSize::default());
        app
    }

    // =======================================================================
    // Section 11: PixelateOut
    // =======================================================================

    // --- Behavior 49: PixelateOut implements Transition and OutTransition ---

    #[test]
    fn pixelate_out_satisfies_transition_and_out_transition() {
        let _effect: Box<dyn OutTransition> = Box::new(PixelateOut {
            duration: 0.6,
            color: Color::BLACK,
        });
    }

    // --- Behavior 50: PixelateOut start spawns overlay at zero alpha ---

    #[test]
    fn pixelate_out_start_spawns_overlay_at_zero_alpha() {
        let mut app = effect_test_app();
        app.insert_resource(PixelateOutConfig {
            duration: 0.6,
            color: Color::BLACK,
        });
        app.insert_resource(StartingTransition::<PixelateOut>::new());
        app.add_systems(Update, pixelate_out_start);
        app.update();

        let overlay_count = app
            .world_mut()
            .query_filtered::<Entity, With<TransitionOverlay>>()
            .iter(app.world())
            .count();
        assert_eq!(overlay_count, 1);

        let sprites: Vec<&Sprite> = app
            .world_mut()
            .query_filtered::<&Sprite, With<TransitionOverlay>>()
            .iter(app.world())
            .collect();
        let alpha = sprites[0].color.alpha();
        assert!(
            alpha.abs() < f32::EPSILON,
            "PixelateOut should start at alpha 0.0"
        );

        let size = sprites[0].custom_size.unwrap_or_default();
        assert!((size.x - 1920.0).abs() < f32::EPSILON);
        assert!((size.y - 1080.0).abs() < f32::EPSILON);

        let z_indices: Vec<&GlobalZIndex> = app
            .world_mut()
            .query_filtered::<&GlobalZIndex, With<TransitionOverlay>>()
            .iter(app.world())
            .collect();
        assert_eq!(z_indices[0].0, i32::MAX - 1);

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionReady>>();
        assert_eq!(msgs.iter_current_update_messages().count(), 1);
        assert!(app.world().contains_resource::<TransitionProgress>());
        let progress = app.world().resource::<TransitionProgress>();
        assert!((progress.duration - 0.6).abs() < f32::EPSILON);
    }

    // --- Behavior 51: PixelateOut run increases alpha with block-step curve ---

    #[test]
    fn pixelate_out_run_increases_alpha_at_mid_progress() {
        let mut app = effect_test_app();
        app.insert_resource(RunningTransition::<PixelateOut>::new());
        app.insert_resource(TransitionProgress {
            elapsed: 0.25,
            duration: 1.0,
            completed: false,
        });
        app.world_mut().spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(1920.0, 1080.0)),
                ..default()
            },
            TransitionOverlay,
        ));
        app.add_systems(Update, pixelate_out_run);
        app.update();

        let sprites: Vec<&Sprite> = app
            .world_mut()
            .query_filtered::<&Sprite, With<TransitionOverlay>>()
            .iter(app.world())
            .collect();
        let alpha = sprites[0].color.alpha();
        assert!(
            alpha > 0.0,
            "alpha should have increased at 25% progress, got {alpha}"
        );
        assert!(alpha <= 1.0, "alpha should not exceed 1.0, got {alpha}");
    }

    #[test]
    fn pixelate_out_run_sends_complete_at_full_progress() {
        let mut app = effect_test_app();
        app.insert_resource(RunningTransition::<PixelateOut>::new());
        app.insert_resource(TransitionProgress {
            elapsed: 1.0,
            duration: 1.0,
            completed: false,
        });
        app.world_mut().spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(1920.0, 1080.0)),
                ..default()
            },
            TransitionOverlay,
        ));
        app.add_systems(Update, pixelate_out_run);
        app.update();

        let sprites: Vec<&Sprite> = app
            .world_mut()
            .query_filtered::<&Sprite, With<TransitionOverlay>>()
            .iter(app.world())
            .collect();
        let alpha = sprites[0].color.alpha();
        assert!(
            (alpha - 1.0).abs() < f32::EPSILON,
            "alpha should be 1.0 at full progress"
        );

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
        assert_eq!(msgs.iter_current_update_messages().count(), 1);
    }

    #[test]
    fn pixelate_out_run_does_not_double_send_when_already_completed() {
        let mut app = effect_test_app();
        app.insert_resource(RunningTransition::<PixelateOut>::new());
        app.insert_resource(TransitionProgress {
            elapsed: 1.0,
            duration: 1.0,
            completed: true,
        });
        app.world_mut().spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 1.0),
                custom_size: Some(Vec2::new(1920.0, 1080.0)),
                ..default()
            },
            TransitionOverlay,
        ));
        app.add_systems(Update, pixelate_out_run);
        app.update();

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
        assert_eq!(msgs.iter_current_update_messages().count(), 0);
    }

    // --- Behavior 52: PixelateOut end ---

    #[test]
    fn pixelate_out_end_despawns_overlay_and_sends_transition_over() {
        let mut app = effect_test_app();
        app.insert_resource(EndingTransition::<PixelateOut>::new());
        app.insert_resource(TransitionProgress {
            elapsed: 0.6,
            duration: 0.6,
            completed: true,
        });
        app.world_mut()
            .spawn((Sprite::default(), TransitionOverlay));
        app.add_systems(Update, pixelate_out_end);
        app.update();

        let overlay_count = app
            .world_mut()
            .query_filtered::<Entity, With<TransitionOverlay>>()
            .iter(app.world())
            .count();
        assert_eq!(overlay_count, 0);
        assert!(!app.world().contains_resource::<TransitionProgress>());

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionOver>>();
        assert_eq!(msgs.iter_current_update_messages().count(), 1);
    }

    // =======================================================================
    // Section 12: PixelateIn
    // =======================================================================

    // --- Behavior 53: PixelateIn implements Transition and InTransition ---

    #[test]
    fn pixelate_in_satisfies_transition_and_in_transition() {
        let _effect: Box<dyn InTransition> = Box::new(PixelateIn {
            duration: 0.6,
            color: Color::BLACK,
        });
    }

    // --- Behavior 54: PixelateIn start spawns fully opaque overlay ---

    #[test]
    fn pixelate_in_start_spawns_fully_opaque_overlay() {
        let mut app = effect_test_app();
        app.insert_resource(PixelateInConfig {
            duration: 0.6,
            color: Color::BLACK,
        });
        app.insert_resource(StartingTransition::<PixelateIn>::new());
        app.add_systems(Update, pixelate_in_start);
        app.update();

        let overlay_count = app
            .world_mut()
            .query_filtered::<Entity, With<TransitionOverlay>>()
            .iter(app.world())
            .count();
        assert_eq!(overlay_count, 1);

        let sprites: Vec<&Sprite> = app
            .world_mut()
            .query_filtered::<&Sprite, With<TransitionOverlay>>()
            .iter(app.world())
            .collect();
        let alpha = sprites[0].color.alpha();
        assert!(
            (alpha - 1.0).abs() < f32::EPSILON,
            "PixelateIn should start at alpha 1.0"
        );

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionReady>>();
        assert_eq!(msgs.iter_current_update_messages().count(), 1);
        assert!(app.world().contains_resource::<TransitionProgress>());
        let progress = app.world().resource::<TransitionProgress>();
        assert!((progress.duration - 0.6).abs() < f32::EPSILON);
    }

    // --- Behavior 55: PixelateIn run decreases alpha with block-step curve ---

    #[test]
    fn pixelate_in_run_decreases_alpha_at_mid_progress() {
        let mut app = effect_test_app();
        app.insert_resource(RunningTransition::<PixelateIn>::new());
        app.insert_resource(TransitionProgress {
            elapsed: 0.5,
            duration: 1.0,
            completed: false,
        });
        app.world_mut().spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 1.0),
                custom_size: Some(Vec2::new(1920.0, 1080.0)),
                ..default()
            },
            TransitionOverlay,
        ));
        app.add_systems(Update, pixelate_in_run);
        app.update();

        let sprites: Vec<&Sprite> = app
            .world_mut()
            .query_filtered::<&Sprite, With<TransitionOverlay>>()
            .iter(app.world())
            .collect();
        let alpha = sprites[0].color.alpha();
        assert!(alpha >= 0.0, "alpha should not go negative, got {alpha}");
        assert!(
            alpha < 1.0,
            "alpha should have decreased from 1.0 at 50% progress, got {alpha}"
        );
    }

    #[test]
    fn pixelate_in_run_sends_complete_at_full_progress() {
        let mut app = effect_test_app();
        app.insert_resource(RunningTransition::<PixelateIn>::new());
        app.insert_resource(TransitionProgress {
            elapsed: 1.0,
            duration: 1.0,
            completed: false,
        });
        app.world_mut().spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 1.0),
                custom_size: Some(Vec2::new(1920.0, 1080.0)),
                ..default()
            },
            TransitionOverlay,
        ));
        app.add_systems(Update, pixelate_in_run);
        app.update();

        let sprites: Vec<&Sprite> = app
            .world_mut()
            .query_filtered::<&Sprite, With<TransitionOverlay>>()
            .iter(app.world())
            .collect();
        let alpha = sprites[0].color.alpha();
        assert!(
            alpha.abs() < f32::EPSILON,
            "alpha should be 0.0 at full progress"
        );

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
        assert_eq!(msgs.iter_current_update_messages().count(), 1);
    }

    #[test]
    fn pixelate_in_run_does_not_double_send_when_already_completed() {
        let mut app = effect_test_app();
        app.insert_resource(RunningTransition::<PixelateIn>::new());
        app.insert_resource(TransitionProgress {
            elapsed: 1.0,
            duration: 1.0,
            completed: true,
        });
        app.world_mut().spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(1920.0, 1080.0)),
                ..default()
            },
            TransitionOverlay,
        ));
        app.add_systems(Update, pixelate_in_run);
        app.update();

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
        assert_eq!(msgs.iter_current_update_messages().count(), 0);
    }

    // --- Behavior 56: PixelateIn end ---

    #[test]
    fn pixelate_in_end_despawns_overlay_and_sends_transition_over() {
        let mut app = effect_test_app();
        app.insert_resource(EndingTransition::<PixelateIn>::new());
        app.insert_resource(TransitionProgress {
            elapsed: 0.6,
            duration: 0.6,
            completed: true,
        });
        app.world_mut()
            .spawn((Sprite::default(), TransitionOverlay));
        app.add_systems(Update, pixelate_in_end);
        app.update();

        let overlay_count = app
            .world_mut()
            .query_filtered::<Entity, With<TransitionOverlay>>()
            .iter(app.world())
            .count();
        assert_eq!(overlay_count, 0);
        assert!(!app.world().contains_resource::<TransitionProgress>());

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionOver>>();
        assert_eq!(msgs.iter_current_update_messages().count(), 1);
    }

    // =======================================================================
    // Section 13: insert_starting overrides (behaviors 67-68)
    // =======================================================================

    #[test]
    fn pixelate_out_insert_starting_inserts_marker_and_config() {
        let mut world = World::new();
        let effect = PixelateOut {
            duration: 0.6,
            color: Color::BLACK,
        };
        effect.insert_starting(&mut world);

        assert!(world.contains_resource::<StartingTransition<PixelateOut>>());
        assert!(
            world.contains_resource::<PixelateOutConfig>(),
            "PixelateOutConfig should be inserted by insert_starting"
        );
    }

    #[test]
    fn pixelate_in_insert_starting_inserts_marker_and_config() {
        let mut world = World::new();
        let effect = PixelateIn {
            duration: 0.6,
            color: Color::BLACK,
        };
        effect.insert_starting(&mut world);

        assert!(world.contains_resource::<StartingTransition<PixelateIn>>());
        assert!(
            world.contains_resource::<PixelateInConfig>(),
            "PixelateInConfig should be inserted by insert_starting"
        );
    }
}
