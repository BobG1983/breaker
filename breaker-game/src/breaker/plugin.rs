//! Breaker plugin registration.

use bevy::prelude::*;

use crate::{
    bolt::BoltSystems,
    breaker::{
        BreakerSystems, ForceBumpGrade, SelectedBreaker,
        messages::{BumpPerformed, BumpWhiffed},
        resources::BreakerConfig,
        systems::{
            animate_bump_visual, animate_tilt_visual, apply_entity_scale_to_breaker,
            detect_breaker_cell_collision, detect_breaker_wall_collision, grade_bump,
            init_breaker_params, move_breaker, perfect_bump_dash_cancel, reset_breaker,
            spawn_breaker, spawn_bump_grade_text, spawn_whiff_text, trigger_bump_visual,
            update_breaker_state, update_bump, width_boost_visual,
        },
    },
    run::node::sets::NodeSystems,
    shared::{GameState, PlayingState},
};

/// Plugin for the breaker domain.
///
/// Owns breaker components, state machine, and bump system.
pub struct BreakerPlugin;

impl Plugin for BreakerPlugin {
    fn build(&self, app: &mut App) {
        use crate::breaker::messages::{
            BreakerDestroyedAt, BreakerImpactCell, BreakerImpactWall, BreakerSpawned,
            RequestBreakerDestroyed,
        };
        app.add_message::<BumpPerformed>()
            .add_message::<BumpWhiffed>()
            .add_message::<BreakerSpawned>()
            .add_message::<RequestBreakerDestroyed>()
            .add_message::<BreakerDestroyedAt>()
            .add_message::<BreakerImpactCell>()
            .add_message::<BreakerImpactWall>()
            .init_resource::<BreakerConfig>()
            .init_resource::<SelectedBreaker>()
            .init_resource::<ForceBumpGrade>()
            .add_systems(
                OnEnter(GameState::Playing),
                (
                    spawn_breaker,
                    ApplyDeferred,
                    init_breaker_params.in_set(BreakerSystems::InitParams),
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(GameState::Playing),
                apply_entity_scale_to_breaker
                    .after(BreakerSystems::InitParams)
                    .after(NodeSystems::Spawn),
            )
            .add_systems(
                OnEnter(GameState::Playing),
                reset_breaker
                    .after(BreakerSystems::InitParams)
                    .in_set(BreakerSystems::Reset),
            )
            .add_systems(
                FixedUpdate,
                (
                    update_bump,
                    move_breaker.after(update_bump).in_set(BreakerSystems::Move),
                    update_breaker_state
                        .after(move_breaker)
                        .in_set(BreakerSystems::UpdateState),
                    grade_bump
                        .after(update_bump)
                        .after(BoltSystems::BreakerCollision)
                        .in_set(BreakerSystems::GradeBump),
                    (
                        perfect_bump_dash_cancel,
                        spawn_bump_grade_text,
                        spawn_whiff_text,
                    )
                        .after(grade_bump)
                        .before(BreakerSystems::UpdateState),
                    trigger_bump_visual.after(update_bump),
                    // Collision detection for effect triggers
                    detect_breaker_cell_collision.after(BreakerSystems::Move),
                    detect_breaker_wall_collision.after(BreakerSystems::Move),
                )
                    .run_if(in_state(PlayingState::Active)),
            )
            .add_systems(
                Update,
                (animate_bump_visual, animate_tilt_visual, width_boost_visual)
                    .run_if(in_state(PlayingState::Active)),
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
            .add_plugins(bevy::asset::AssetPlugin::default())
            .init_asset::<crate::breaker::BreakerDefaults>()
            .init_state::<GameState>()
            .add_sub_state::<PlayingState>()
            .init_resource::<crate::shared::PlayfieldConfig>()
            // InputPlugin owns InputActions — init resources it provides
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<bevy::input::keyboard::KeyboardInput>()
            .add_plugins(crate::input::InputPlugin)
            // BreakerPlugin reads BoltImpactBreaker from the bolt domain
            .add_message::<crate::bolt::messages::BoltImpactBreaker>()
            .add_plugins(BreakerPlugin)
            .update();
    }
}
