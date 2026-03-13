//! Breaker plugin registration.

use bevy::prelude::*;

use crate::{
    breaker::{
        BreakerSystems,
        messages::{BumpPerformed, BumpWhiffed},
        resources::BreakerConfig,
        systems::{
            animate_bump_visual, animate_tilt_visual, grade_bump, init_breaker_params,
            move_breaker, perfect_bump_dash_cancel, reset_breaker, spawn_breaker,
            spawn_bump_grade_text, spawn_whiff_text, trigger_bump_visual, update_breaker_state,
            update_bump,
        },
    },
    physics::PhysicsSystems,
    shared::{GameState, PlayingState},
};

/// Plugin for the breaker domain.
///
/// Owns breaker components, state machine, and bump system.
pub struct BreakerPlugin;

impl Plugin for BreakerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BumpPerformed>()
            .add_message::<BumpWhiffed>()
            .init_resource::<BreakerConfig>()
            .add_systems(
                OnEnter(GameState::Playing),
                (
                    spawn_breaker,
                    init_breaker_params.after(spawn_breaker),
                    reset_breaker.after(init_breaker_params),
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    update_bump,
                    move_breaker.after(update_bump).in_set(BreakerSystems::Move),
                    update_breaker_state.after(move_breaker),
                    grade_bump
                        .after(update_bump)
                        .after(PhysicsSystems::BreakerCollision),
                    (
                        perfect_bump_dash_cancel,
                        spawn_bump_grade_text,
                        spawn_whiff_text,
                    )
                        .after(grade_bump),
                    trigger_bump_visual.after(update_bump),
                )
                    .run_if(in_state(PlayingState::Active)),
            )
            .add_systems(
                Update,
                (animate_bump_visual, animate_tilt_visual).run_if(in_state(PlayingState::Active)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<GameState>()
            .add_sub_state::<PlayingState>()
            .init_resource::<crate::shared::PlayfieldConfig>()
            // InputPlugin owns InputActions — init resources it provides
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<bevy::input::keyboard::KeyboardInput>()
            .add_plugins(crate::input::InputPlugin)
            // BreakerPlugin reads BoltHitBreaker messages from the physics domain
            .add_message::<crate::physics::messages::BoltHitBreaker>()
            .add_plugins(BreakerPlugin)
            .update();
    }
}
