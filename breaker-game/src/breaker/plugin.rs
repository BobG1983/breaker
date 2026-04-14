//! Breaker plugin registration.

use bevy::prelude::*;

use crate::{
    bolt::BoltSystems,
    breaker::{
        BreakerSystems, ForceBumpGrade, SelectedBreaker,
        messages::{
            BreakerImpactCell, BreakerImpactWall, BreakerSpawned, BumpPerformed, BumpWhiffed,
            NoBump,
        },
        systems::{
            animate_bump_visual, animate_tilt_visual, breaker_cell_collision,
            breaker_wall_collision, grade_bump, move_breaker, perfect_bump_dash_cancel,
            spawn_bump_grade_text, spawn_whiff_text, sync_breaker_scale, trigger_bump_visual,
            update_breaker_state, update_bump,
        },
    },
    effect_v3::EffectV3Systems,
    prelude::*,
    state::run::node::{
        sets::NodeSystems,
        systems::{apply_node_scale_to_breaker, reset_breaker},
    },
};

/// Plugin for the breaker domain.
///
/// Owns breaker components, state machine, and bump system.
pub struct BreakerPlugin;

impl Plugin for BreakerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BumpPerformed>()
            .add_message::<BumpWhiffed>()
            .add_message::<NoBump>()
            .add_message::<BreakerSpawned>()
            .add_message::<BreakerImpactCell>()
            .add_message::<BreakerImpactWall>()
            .init_resource::<SelectedBreaker>()
            .init_resource::<ForceBumpGrade>()
            .add_systems(
                OnEnter(NodeState::Loading),
                apply_node_scale_to_breaker
                    .after(BreakerSystems::Reset)
                    .after(NodeSystems::Spawn),
            )
            .add_systems(
                OnEnter(NodeState::Loading),
                reset_breaker.in_set(BreakerSystems::Reset),
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
                        .in_set(BreakerSystems::GradeBump)
                        .before(EffectV3Systems::Bridge),
                    (
                        perfect_bump_dash_cancel,
                        spawn_bump_grade_text,
                        spawn_whiff_text,
                    )
                        .after(grade_bump)
                        .before(BreakerSystems::UpdateState),
                    trigger_bump_visual.after(update_bump),
                    // Collision detection for effect triggers
                    breaker_cell_collision.after(BreakerSystems::Move),
                    breaker_wall_collision.after(BreakerSystems::Move),
                )
                    .run_if(in_state(NodeState::Playing)),
            )
            .add_systems(
                Update,
                (animate_bump_visual, animate_tilt_visual, sync_breaker_scale)
                    .run_if(in_state(NodeState::Playing)),
            );
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_physics2d::resources::CollisionQuadtree;

    use super::*;
    use crate::state::types::{AppState, GameState, RunState};

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .add_plugins(bevy::asset::AssetPlugin::default())
            .init_asset::<Mesh>()
            .init_asset::<ColorMaterial>()
            .init_state::<AppState>()
            .add_sub_state::<GameState>()
            .add_sub_state::<RunState>()
            .add_sub_state::<NodeState>()
            .init_resource::<crate::shared::PlayfieldConfig>()
            .init_resource::<crate::breaker::BreakerRegistry>()
            .init_resource::<SelectedBreaker>()
            // InputPlugin owns InputActions — init resources it provides
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<bevy::input::keyboard::KeyboardInput>()
            .add_plugins(crate::input::InputPlugin)
            // BreakerPlugin reads BoltImpactBreaker from the bolt domain
            .add_message::<crate::bolt::messages::BoltImpactBreaker>()
            .insert_resource(CollisionQuadtree::default())
            .add_plugins(BreakerPlugin)
            .update();
    }
}
