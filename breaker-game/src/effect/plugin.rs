//! `EffectPlugin` — wires archetype init, bridge systems, and observers.

use bevy::prelude::*;

use super::{
    active::ActiveEffects,
    bridges::{
        bridge_bolt_lost, bridge_breaker_impact, bridge_bump, bridge_bump_whiff,
        bridge_cell_destroyed, bridge_cell_impact, bridge_wall_impact,
    },
    effects::{
        attraction::handle_attraction,
        bolt_size_boost::handle_bolt_size_boost,
        bolt_speed_boost::handle_bolt_speed_boost,
        breaker_speed_boost::handle_breaker_speed_boost,
        bump_force_boost::handle_bump_force_boost,
        chain_bolt::handle_chain_bolt,
        chain_hit::handle_chain_hit,
        chain_lightning::handle_chain_lightning,
        damage_boost::handle_damage_boost,
        gravity_well::handle_gravity_well,
        life_lost::{LivesDisplay, handle_life_lost, spawn_lives_display, update_lives_display},
        multi_bolt::handle_multi_bolt,
        piercing::handle_piercing,
        piercing_beam::handle_piercing_beam,
        second_wind::handle_second_wind,
        shield::{handle_shield, tick_shield},
        shockwave::{
            ShockwaveRadius, animate_shockwave, handle_shockwave, shockwave_collision,
            tick_shockwave,
        },
        spawn_bolt::handle_spawn_bolt,
        spawn_phantom::handle_spawn_phantom,
        speed_boost::handle_speed_boost,
        tilt_control_boost::handle_tilt_control_boost,
        time_penalty::handle_time_penalty,
        width_boost::handle_width_boost,
    },
    init::{apply_archetype_config_overrides, init_archetype},
    registry::ArchetypeRegistry,
    sets::EffectSystems,
};
use crate::{
    bolt::BoltSystems,
    breaker::BreakerSystems,
    shared::{GameState, PlayingState},
    ui::UiSystems,
};

/// Plugin for the effect system.
///
/// Registers:
/// - Archetype init systems (config overrides, component stamping)
/// - Per-trigger bridge systems (message → effect event)
/// - Effect observers (event → game effect)
/// - Lives HUD
pub(crate) struct EffectPlugin;

impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ArchetypeRegistry>()
            .init_resource::<ActiveEffects>()
            // Effect observers — triggered effects (fired by bridge systems)
            .add_observer(handle_life_lost)
            .add_observer(handle_time_penalty)
            .add_observer(handle_spawn_bolt)
            .add_observer(handle_shockwave)
            .add_observer(handle_speed_boost)
            .add_observer(handle_chain_bolt)
            .add_observer(handle_multi_bolt)
            .add_observer(handle_shield)
            .add_observer(handle_chain_lightning)
            .add_observer(handle_spawn_phantom)
            .add_observer(handle_piercing_beam)
            .add_observer(handle_gravity_well)
            .add_observer(handle_second_wind)
            // Passive handler observers (moved from ChipsPlugin)
            .add_observer(handle_piercing)
            .add_observer(handle_damage_boost)
            .add_observer(handle_bolt_speed_boost)
            .add_observer(handle_chain_hit)
            .add_observer(handle_bolt_size_boost)
            .add_observer(handle_width_boost)
            .add_observer(handle_breaker_speed_boost)
            .add_observer(handle_bump_force_boost)
            .add_observer(handle_tilt_control_boost)
            .add_observer(handle_attraction)
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
                        .after(BoltSystems::BoltLost)
                        .in_set(EffectSystems::Bridge),
                    bridge_bump
                        .after(BreakerSystems::GradeBump)
                        .in_set(EffectSystems::Bridge),
                    bridge_bump_whiff
                        .after(BreakerSystems::GradeBump)
                        .in_set(EffectSystems::Bridge),
                    bridge_cell_impact
                        .after(BoltSystems::BreakerCollision)
                        .in_set(EffectSystems::Bridge),
                    bridge_breaker_impact
                        .after(BoltSystems::BreakerCollision)
                        .in_set(EffectSystems::Bridge),
                    bridge_wall_impact
                        .after(BoltSystems::BreakerCollision)
                        .in_set(EffectSystems::Bridge),
                    bridge_cell_destroyed.in_set(EffectSystems::Bridge),
                )
                    .run_if(in_state(PlayingState::Active)),
            )
            // Shockwave expansion + collision
            .add_systems(
                FixedUpdate,
                (tick_shockwave, shockwave_collision.after(tick_shockwave))
                    .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree)
                    .run_if(in_state(PlayingState::Active)),
            )
            // Shield tick (decrement + remove)
            .add_systems(
                FixedUpdate,
                tick_shield
                    .after(EffectSystems::Bridge)
                    .run_if(in_state(PlayingState::Active)),
            )
            // HUD + shockwave visual update
            .add_systems(
                Update,
                (
                    update_lives_display.run_if(any_with_component::<LivesDisplay>),
                    animate_shockwave.run_if(any_with_component::<ShockwaveRadius>),
                )
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
            .add_message::<crate::bolt::messages::BoltHitBreaker>()
            .add_message::<crate::bolt::messages::BoltHitCell>()
            .add_message::<crate::bolt::messages::BoltHitWall>()
            .add_message::<crate::bolt::messages::BoltLost>()
            .add_message::<crate::cells::messages::CellDestroyed>()
            .add_message::<crate::breaker::messages::BumpWhiffed>()
            .add_plugins(BreakerPlugin)
            .add_plugins(EffectPlugin)
            .update();
    }

    // =========================================================================
    // B12d Behavior 21: EffectPlugin registers passive handler observers
    // =========================================================================

    /// Verifies that `EffectPlugin` registers the `handle_piercing` observer.
    /// When `PiercingApplied` is triggered with `EffectPlugin`, a bolt entity
    /// should gain the `Piercing` component.
    #[test]
    fn effect_plugin_registers_handle_piercing_observer() {
        use crate::{
            bolt::components::Bolt, chips::components::Piercing,
            effect::typed_events::PiercingApplied,
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
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
            .add_message::<crate::bolt::messages::BoltHitBreaker>()
            .add_message::<crate::bolt::messages::BoltHitCell>()
            .add_message::<crate::bolt::messages::BoltHitWall>()
            .add_message::<crate::bolt::messages::BoltLost>()
            .add_message::<crate::cells::messages::CellDestroyed>()
            .add_message::<crate::breaker::messages::BumpWhiffed>()
            .add_plugins(BreakerPlugin)
            .add_plugins(EffectPlugin);

        let bolt = app.world_mut().spawn(Bolt).id();

        app.world_mut().commands().trigger(PiercingApplied {
            per_stack: 1,
            max_stacks: 3,
            chip_name: "test".to_owned(),
        });
        app.world_mut().flush();

        let p = app.world().entity(bolt).get::<Piercing>().expect(
            "EffectPlugin should register handle_piercing — bolt should gain Piercing component",
        );
        assert_eq!(p.0, 1);
    }

    /// Verifies that `EffectPlugin` registers the `handle_damage_boost` observer.
    #[test]
    fn effect_plugin_registers_handle_damage_boost_observer() {
        use crate::{
            bolt::components::Bolt, chips::components::DamageBoost,
            effect::typed_events::DamageBoostApplied,
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
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
            .add_message::<crate::bolt::messages::BoltHitBreaker>()
            .add_message::<crate::bolt::messages::BoltHitCell>()
            .add_message::<crate::bolt::messages::BoltHitWall>()
            .add_message::<crate::bolt::messages::BoltLost>()
            .add_message::<crate::cells::messages::CellDestroyed>()
            .add_message::<crate::breaker::messages::BumpWhiffed>()
            .add_plugins(BreakerPlugin)
            .add_plugins(EffectPlugin);

        let bolt = app.world_mut().spawn(Bolt).id();

        app.world_mut().commands().trigger(DamageBoostApplied {
            per_stack: 0.5,
            max_stacks: 3,
            chip_name: "test".to_owned(),
        });
        app.world_mut().flush();

        let d = app
            .world()
            .entity(bolt)
            .get::<DamageBoost>()
            .expect("EffectPlugin should register handle_damage_boost — bolt should gain DamageBoost component");
        assert!(
            (d.0 - 0.5).abs() < f32::EPSILON,
            "DamageBoost should be 0.5, got {}",
            d.0
        );
    }
}
