//! Timed transition animations between nodes.

use bevy::prelude::*;
use rand::Rng;
use rantzsoft_defaults::GameConfig;
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
pub(super) const fn overlay_color(
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
