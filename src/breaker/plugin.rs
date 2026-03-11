//! Breaker plugin registration.

use bevy::prelude::*;

use crate::breaker::messages::BumpPerformed;
use crate::breaker::resources::BreakerConfig;
use crate::breaker::systems::{
    animate_bump_visual, grade_bump, move_breaker, perfect_bump_dash_cancel, spawn_breaker,
    trigger_bump_visual, update_breaker_state, update_bump,
};
use crate::shared::{GameState, PlayingState};

/// Plugin for the breaker domain.
///
/// Owns breaker components, state machine, and bump system.
pub struct BreakerPlugin;

impl Plugin for BreakerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BumpPerformed>();
        app.init_resource::<BreakerConfig>();
        app.add_systems(OnEnter(GameState::Playing), spawn_breaker);
        app.add_systems(
            FixedUpdate,
            (
                update_bump,
                move_breaker.after(update_bump),
                update_breaker_state.after(move_breaker),
                grade_bump.after(update_breaker_state),
                perfect_bump_dash_cancel.after(grade_bump),
                trigger_bump_visual.after(update_bump),
                animate_bump_visual
                    .after(trigger_bump_visual)
                    .after(move_breaker),
            )
                .run_if(in_state(PlayingState::Active)),
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
        // BreakerPlugin reads BoltHitBreaker messages from the physics domain
        app.add_message::<crate::physics::messages::BoltHitBreaker>();
        app.add_plugins(BreakerPlugin);
        app.update();
    }
}
