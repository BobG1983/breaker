//! Timed transition animations between nodes.
//!
//! Provides visual transitions (flash, sweep) when entering/leaving a node.
//! Spawns a full-screen overlay entity with a [`TransitionTimer`] that drives
//! the animation, then transitions to the next [`GameState`] on completion.

use bevy::prelude::*;
use breaker_derive::GameConfig;
use rand::Rng;
use serde::Deserialize;

/// Visual style for a node transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransitionStyle {
    /// Full-screen overlay fades in/out.
    Flash,
    /// Full-screen rect sweeps across screen.
    Sweep,
}

/// Direction of a transition animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransitionDirection {
    /// Transitioning out of current node.
    Out,
    /// Transitioning into next node.
    In,
}

/// Timer component driving a transition animation.
#[derive(Component, Debug)]
pub(crate) struct TransitionTimer {
    /// Remaining time in seconds.
    pub remaining: f32,
    /// Total duration in seconds.
    pub duration: f32,
    /// Visual style of this transition.
    pub style: TransitionStyle,
    /// Direction (out or in).
    pub direction: TransitionDirection,
}

/// Cleanup marker for transition overlay entities.
#[derive(Component)]
pub(crate) struct TransitionOverlay;

/// Transition defaults loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug, GameConfig)]
#[game_config(name = "TransitionConfig")]
pub(crate) struct TransitionDefaults {
    /// Duration of the transition-out animation in seconds.
    pub out_duration: f32,
    /// Duration of the transition-in animation in seconds.
    pub in_duration: f32,
    /// RGB color for the flash transition style.
    pub flash_color_rgb: [f32; 3],
    /// RGB color for the sweep transition style.
    pub sweep_color_rgb: [f32; 3],
}

impl Default for TransitionDefaults {
    fn default() -> Self {
        Self {
            out_duration: 0.5,
            in_duration: 0.3,
            flash_color_rgb: [1.0, 1.0, 1.0],
            sweep_color_rgb: [0.0, 0.8, 1.0],
        }
    }
}

/// Spawns a transition overlay for the out phase.
pub(crate) fn spawn_transition_out(
    mut commands: Commands,
    config: Res<TransitionConfig>,
    mut rng: ResMut<crate::shared::GameRng>,
) {
    let style = pick_style(&mut rng);
    let duration = config.out_duration;
    let color = overlay_color(&config, style, TransitionDirection::Out);

    commands.spawn((
        TransitionOverlay,
        TransitionTimer {
            remaining: duration,
            duration,
            style,
            direction: TransitionDirection::Out,
        },
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(color),
    ));
}

/// Spawns a transition overlay for the in phase.
pub(crate) fn spawn_transition_in(
    mut commands: Commands,
    config: Res<TransitionConfig>,
    mut rng: ResMut<crate::shared::GameRng>,
) {
    let style = pick_style(&mut rng);
    let duration = config.in_duration;
    let color = overlay_color(&config, style, TransitionDirection::In);

    commands.spawn((
        TransitionOverlay,
        TransitionTimer {
            remaining: duration,
            duration,
            style,
            direction: TransitionDirection::In,
        },
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(color),
    ));
}

/// Ticks the transition timer and updates overlay visuals. On completion,
/// transitions to the next [`GameState`].
pub(crate) fn animate_transition(
    mut timer_query: Query<(&mut TransitionTimer, &mut BackgroundColor, &mut Node)>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<crate::shared::GameState>>,
) {
    for (mut timer, mut bg_color, mut node) in &mut timer_query {
        timer.remaining -= time.delta_secs();

        if timer.remaining <= 0.0 {
            match timer.direction {
                TransitionDirection::Out => {
                    next_state.set(crate::shared::GameState::ChipSelect);
                }
                TransitionDirection::In => {
                    next_state.set(crate::shared::GameState::Playing);
                }
            }
        } else {
            let progress = 1.0 - (timer.remaining / timer.duration);
            match timer.style {
                TransitionStyle::Flash => {
                    let alpha = match timer.direction {
                        TransitionDirection::Out => progress,
                        TransitionDirection::In => 1.0 - progress,
                    };
                    bg_color.0 = bg_color.0.with_alpha(alpha);
                }
                TransitionStyle::Sweep => {
                    let left = match timer.direction {
                        TransitionDirection::Out => Val::Percent(progress * 100.0),
                        TransitionDirection::In => Val::Percent((1.0 - progress) * 100.0),
                    };
                    node.left = left;
                }
            }
        }
    }
}

/// Despawns all [`TransitionOverlay`] entities.
pub(crate) fn cleanup_transition(
    mut commands: Commands,
    query: Query<Entity, With<TransitionOverlay>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Picks a random [`TransitionStyle`] from the game RNG.
fn pick_style(rng: &mut ResMut<crate::shared::GameRng>) -> TransitionStyle {
    if rng.0.random_range(0..2) == 0 {
        TransitionStyle::Flash
    } else {
        TransitionStyle::Sweep
    }
}

/// Returns the initial overlay color for the given style and direction.
fn overlay_color(
    config: &TransitionConfig,
    style: TransitionStyle,
    direction: TransitionDirection,
) -> Color {
    let rgb = match style {
        TransitionStyle::Flash => config.flash_color_rgb,
        TransitionStyle::Sweep => config.sweep_color_rgb,
    };
    let alpha = match (style, direction) {
        (TransitionStyle::Flash, TransitionDirection::Out) => 0.0,
        (TransitionStyle::Flash, TransitionDirection::In) | (TransitionStyle::Sweep, _) => 1.0,
    };
    Color::srgba(rgb[0], rgb[1], rgb[2], alpha)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::{state::app::StatesPlugin, time::TimeUpdateStrategy};

    use super::*;
    use crate::shared::{GameRng, GameState};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<GameState>()
            .insert_resource(TransitionConfig::default())
            .insert_resource(GameRng::from_seed(42));
        app
    }

    #[test]
    fn spawn_transition_out_creates_overlay_entity() {
        let mut app = test_app();
        app.add_systems(Update, spawn_transition_out);
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<TransitionOverlay>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "expected exactly 1 TransitionOverlay entity");
    }

    #[test]
    fn spawn_transition_out_sets_timer_direction_out() {
        let mut app = test_app();
        app.add_systems(Update, spawn_transition_out);
        app.update();

        let timer = app
            .world_mut()
            .query::<&TransitionTimer>()
            .iter(app.world())
            .next()
            .expect("expected a TransitionTimer entity");
        assert_eq!(
            timer.direction,
            TransitionDirection::Out,
            "transition-out should set direction to Out"
        );
        assert!(
            (timer.duration - 0.5).abs() < f32::EPSILON,
            "expected duration 0.5 from default config, got {}",
            timer.duration
        );
    }

    #[test]
    fn spawn_transition_in_creates_overlay_entity() {
        let mut app = test_app();
        app.add_systems(Update, spawn_transition_in);
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<TransitionOverlay>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "expected exactly 1 TransitionOverlay entity");
    }

    #[test]
    fn spawn_transition_in_sets_timer_direction_in() {
        let mut app = test_app();
        app.add_systems(Update, spawn_transition_in);
        app.update();

        let timer = app
            .world_mut()
            .query::<&TransitionTimer>()
            .iter(app.world())
            .next()
            .expect("expected a TransitionTimer entity");
        assert_eq!(
            timer.direction,
            TransitionDirection::In,
            "transition-in should set direction to In"
        );
        assert!(
            (timer.duration - 0.3).abs() < f32::EPSILON,
            "expected duration 0.3 from default config, got {}",
            timer.duration
        );
    }

    #[test]
    fn animate_transition_ticks_timer_down() {
        let mut app = test_app();
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            0.1,
        )));
        app.add_systems(Update, animate_transition);

        app.world_mut().spawn((
            TransitionTimer {
                remaining: 0.5,
                duration: 0.5,
                style: TransitionStyle::Flash,
                direction: TransitionDirection::Out,
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
            Node::default(),
        ));

        // First update initializes time, second advances it by 0.1s
        app.update();
        app.update();

        let timer = app
            .world_mut()
            .query::<&TransitionTimer>()
            .iter(app.world())
            .next()
            .expect("timer entity should still exist");
        assert!(
            (timer.remaining - 0.4).abs() < 0.02,
            "expected remaining ~0.4 after 0.1s tick, got {}",
            timer.remaining
        );
    }

    #[test]
    fn animate_transition_out_completion_transitions_to_chip_select() {
        let mut app = test_app();
        app.add_systems(Update, animate_transition);

        app.world_mut().spawn((
            TransitionTimer {
                remaining: 0.0,
                duration: 0.5,
                style: TransitionStyle::Flash,
                direction: TransitionDirection::Out,
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
            Node::default(),
        ));

        app.update();

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("ChipSelect"),
            "expected ChipSelect after out-transition completes, got: {next:?}"
        );
    }

    #[test]
    fn animate_transition_in_completion_transitions_to_playing() {
        let mut app = test_app();
        app.add_systems(Update, animate_transition);

        app.world_mut().spawn((
            TransitionTimer {
                remaining: 0.0,
                duration: 0.3,
                style: TransitionStyle::Flash,
                direction: TransitionDirection::In,
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
            Node::default(),
        ));

        app.update();

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("Playing"),
            "expected Playing after in-transition completes, got: {next:?}"
        );
    }

    #[test]
    fn cleanup_transition_despawns_overlay_entities() {
        let mut app = test_app();
        app.add_systems(Update, cleanup_transition);

        // Spawn one overlay entity and one non-overlay entity
        app.world_mut().spawn(TransitionOverlay);
        let other = app.world_mut().spawn(Name::new("not-an-overlay")).id();

        app.update();

        let overlay_count = app
            .world_mut()
            .query_filtered::<Entity, With<TransitionOverlay>>()
            .iter(app.world())
            .count();
        assert_eq!(
            overlay_count, 0,
            "all TransitionOverlay entities should be despawned"
        );
        assert!(
            app.world().get_entity(other).is_ok(),
            "non-overlay entity should still exist"
        );
    }

    #[test]
    fn same_seed_produces_same_style() {
        // First run
        let mut app1 = test_app();
        app1.add_systems(Update, spawn_transition_out);
        app1.update();

        let style1 = app1
            .world_mut()
            .query::<&TransitionTimer>()
            .iter(app1.world())
            .next()
            .expect("expected a TransitionTimer entity from first run")
            .style;

        // Second run with same seed
        let mut app2 = test_app();
        app2.add_systems(Update, spawn_transition_out);
        app2.update();

        let style2 = app2
            .world_mut()
            .query::<&TransitionTimer>()
            .iter(app2.world())
            .next()
            .expect("expected a TransitionTimer entity from second run")
            .style;

        assert_eq!(
            style1, style2,
            "same seed should produce the same TransitionStyle"
        );
    }

    #[test]
    fn transition_duration_configurable() {
        let mut app = test_app();
        app.insert_resource(TransitionConfig {
            out_duration: 1.5,
            in_duration: 0.3,
            flash_color_rgb: [1.0, 1.0, 1.0],
            sweep_color_rgb: [0.0, 0.8, 1.0],
        });
        app.add_systems(Update, spawn_transition_out);
        app.update();

        let timer = app
            .world_mut()
            .query::<&TransitionTimer>()
            .iter(app.world())
            .next()
            .expect("expected a TransitionTimer entity");
        assert!(
            (timer.duration - 1.5).abs() < f32::EPSILON,
            "expected duration 1.5 from custom config, got {}",
            timer.duration
        );
    }
}
