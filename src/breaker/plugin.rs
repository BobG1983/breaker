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
        app.add_message::<BumpPerformed>();
        app.add_message::<BumpWhiffed>();
        app.init_resource::<BreakerConfig>();
        app.add_systems(
            OnEnter(GameState::Playing),
            (
                spawn_breaker,
                init_breaker_params.after(spawn_breaker),
                reset_breaker.after(init_breaker_params),
            ),
        );
        app.add_systems(
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
        );
        app.add_systems(
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
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.add_sub_state::<PlayingState>();
        app.init_resource::<crate::shared::PlayfieldConfig>();
        // InputPlugin owns InputActions — init resources it provides
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_message::<bevy::input::keyboard::KeyboardInput>();
        app.add_plugins(crate::input::InputPlugin);
        // BreakerPlugin reads BoltHitBreaker messages from the physics domain
        app.add_message::<crate::physics::messages::BoltHitBreaker>();
        app.add_plugins(BreakerPlugin);
        app.update();
    }
}
