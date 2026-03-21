//! `BehaviorsPlugin` — wires archetype init, bridge systems, and observers.

use bevy::prelude::*;

use super::{
    active::ActiveChains,
    bridges::{
        bridge_bolt_lost, bridge_breaker_impact, bridge_bump, bridge_bump_whiff,
        bridge_cell_destroyed, bridge_cell_impact, bridge_wall_impact,
    },
    effects::{
        life_lost::{LivesDisplay, handle_life_lost, spawn_lives_display, update_lives_display},
        shockwave::handle_shockwave,
        spawn_bolt::handle_spawn_bolt,
        time_penalty::handle_time_penalty,
    },
    init::{apply_archetype_config_overrides, init_archetype},
    registry::ArchetypeRegistry,
    sets::BehaviorSystems,
};
use crate::{
    breaker::BreakerSystems,
    physics::PhysicsSystems,
    shared::{GameState, PlayingState},
    ui::UiSystems,
};

/// Plugin for the behavior system.
///
/// Registers:
/// - Archetype init systems (config overrides, component stamping)
/// - Per-trigger bridge systems (message → effect event)
/// - Effect observers (event → game effect)
/// - Lives HUD
pub(crate) struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ArchetypeRegistry>()
            .init_resource::<ActiveChains>()
            // Effect observers
            .add_observer(handle_life_lost)
            .add_observer(handle_time_penalty)
            .add_observer(handle_spawn_bolt)
            .add_observer(handle_shockwave)
            // Init systems — run on entering Playing state
            .add_systems(
                OnEnter(GameState::Playing),
                (
                    apply_archetype_config_overrides.before(BreakerSystems::InitParams),
                    init_archetype.after(BreakerSystems::InitParams),
                    spawn_lives_display
                        .after(init_archetype)
                        .after(UiSystems::SpawnTimerHud),
                ),
            )
            // Bridge systems — each reads one message type, run in parallel
            .add_systems(
                FixedUpdate,
                (
                    bridge_bolt_lost
                        .after(PhysicsSystems::BoltLost)
                        .in_set(BehaviorSystems::Bridge),
                    bridge_bump
                        .after(BreakerSystems::GradeBump)
                        .in_set(BehaviorSystems::Bridge),
                    bridge_bump_whiff
                        .after(BreakerSystems::GradeBump)
                        .in_set(BehaviorSystems::Bridge),
                    bridge_cell_impact
                        .after(PhysicsSystems::BreakerCollision)
                        .in_set(BehaviorSystems::Bridge),
                    bridge_breaker_impact
                        .after(PhysicsSystems::BreakerCollision)
                        .in_set(BehaviorSystems::Bridge),
                    bridge_wall_impact
                        .after(PhysicsSystems::BreakerCollision)
                        .in_set(BehaviorSystems::Bridge),
                    bridge_cell_destroyed.in_set(BehaviorSystems::Bridge),
                )
                    .run_if(in_state(PlayingState::Active)),
            )
            // HUD update
            .add_systems(
                Update,
                update_lives_display
                    .run_if(any_with_component::<LivesDisplay>)
                    .run_if(in_state(PlayingState::Active)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        breaker::BreakerPlugin,
        shared::{PlayfieldConfig, SelectedArchetype},
    };

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .add_plugins(bevy::asset::AssetPlugin::default())
            .init_asset::<crate::breaker::BreakerDefaults>()
            .init_state::<GameState>()
            .add_sub_state::<PlayingState>()
            .init_resource::<PlayfieldConfig>()
            .init_resource::<SelectedArchetype>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<bevy::input::keyboard::KeyboardInput>()
            .add_plugins(crate::input::InputPlugin)
            .add_message::<crate::physics::messages::BoltHitBreaker>()
            .add_message::<crate::physics::messages::BoltHitCell>()
            .add_message::<crate::physics::messages::BoltHitWall>()
            .add_message::<crate::physics::messages::BoltLost>()
            .add_message::<crate::cells::messages::CellDestroyed>()
            .add_message::<crate::breaker::messages::BumpWhiffed>()
            .add_plugins(BreakerPlugin)
            .add_plugins(BehaviorsPlugin)
            .update();
    }
}
