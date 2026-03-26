//! `EffectPlugin` — wires breaker init, bridge systems, and observers.

use bevy::prelude::*;

use super::{
    effect_nodes::until::{check_until_triggers, tick_until_timers},
    effects,
    sets::EffectSystems,
    triggers::{
        bridge_bolt_death, bridge_bolt_lost, bridge_breaker_impact, bridge_bump, bridge_bump_whiff,
        bridge_cell_death, bridge_cell_impact, bridge_no_bump, bridge_timer_threshold,
        bridge_wall_impact, cleanup_destroyed_bolts, cleanup_destroyed_cells,
    },
};
use crate::{
    bolt::BoltSystems,
    breaker::{
        BreakerRegistry, BreakerSystems,
        systems::{apply_breaker_config_overrides, init_breaker},
    },
    shared::{GameState, PlayingState},
    ui::UiSystems,
};

/// Plugin for the effect system.
///
/// Registers:
/// - Breaker init systems (config overrides, component stamping)
/// - Per-trigger bridge systems (message -> effect event)
/// - Effect observers (event -> game effect) via per-effect self-registration
/// - Lives HUD
pub(crate) struct EffectPlugin;

impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BreakerRegistry>()
            .init_resource::<crate::effect::active::ActiveEffects>();

        // ── Effect self-registration ────────────────────────────────────
        // Each effect file owns its observer and system registrations.
        // Triggered effects
        effects::life_lost::register(app);
        effects::time_penalty::register(app);
        effects::spawn_bolt::register(app);
        effects::shockwave::register(app);
        effects::speed_boost::register(app);
        effects::chain_bolt::register(app);
        effects::multi_bolt::register(app);
        effects::shield::register(app);
        effects::chain_lightning::register(app);
        effects::spawn_phantom::register(app);
        effects::piercing_beam::register(app);
        effects::gravity_well::register(app);
        effects::second_wind::register(app);
        effects::random_effect::register(app);
        effects::entropy_engine::register(app);
        // Passive effects
        effects::ramping_damage::register(app);
        effects::piercing::register(app);
        effects::damage_boost::register(app);
        effects::bolt_speed_boost::register(app);
        effects::chain_hit::register(app);
        effects::bolt_size_boost::register(app);
        effects::width_boost::register(app);
        effects::breaker_speed_boost::register(app);
        effects::bump_force_boost::register(app);
        effects::tilt_control_boost::register(app);
        effects::attraction::register(app);

        app
            // Init systems — run on entering Playing state
            .add_systems(
                OnEnter(GameState::Playing),
                (
                    apply_breaker_config_overrides.before(BreakerSystems::InitParams),
                    init_breaker.after(BreakerSystems::InitParams),
                    effects::life_lost::spawn_lives_display
                        .after(init_breaker)
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
                    bridge_no_bump
                        .after(bridge_breaker_impact)
                        .after(bridge_bump)
                        .in_set(EffectSystems::Bridge),
                    bridge_cell_death.in_set(EffectSystems::Bridge),
                    bridge_bolt_death.in_set(EffectSystems::Bridge),
                    bridge_timer_threshold.in_set(EffectSystems::Bridge),
                )
                    .run_if(in_state(PlayingState::Active)),
            )
            // Until timer/trigger systems
            .add_systems(
                FixedUpdate,
                (tick_until_timers, check_until_triggers)
                    .after(EffectSystems::Bridge)
                    .run_if(in_state(PlayingState::Active)),
            )
            // Cleanup systems — despawn entities after bridges evaluate
            .add_systems(
                FixedUpdate,
                (cleanup_destroyed_cells, cleanup_destroyed_bolts)
                    .after(EffectSystems::Bridge)
                    .run_if(in_state(PlayingState::Active)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        breaker::BreakerPlugin,
        shared::{PlayfieldConfig, SelectedBreaker},
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
            .init_resource::<SelectedBreaker>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<bevy::input::keyboard::KeyboardInput>()
            .add_plugins(crate::input::InputPlugin)
            .add_message::<crate::bolt::messages::BoltHitBreaker>()
            .add_message::<crate::bolt::messages::BoltHitCell>()
            .add_message::<crate::bolt::messages::BoltHitWall>()
            .add_message::<crate::bolt::messages::BoltLost>()
            .add_message::<crate::cells::messages::CellDestroyedAt>()
            .add_message::<crate::cells::messages::RequestCellDestroyed>()
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
            .init_resource::<SelectedBreaker>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<bevy::input::keyboard::KeyboardInput>()
            .add_plugins(crate::input::InputPlugin)
            .add_message::<crate::bolt::messages::BoltHitBreaker>()
            .add_message::<crate::bolt::messages::BoltHitCell>()
            .add_message::<crate::bolt::messages::BoltHitWall>()
            .add_message::<crate::bolt::messages::BoltLost>()
            .add_message::<crate::cells::messages::CellDestroyedAt>()
            .add_message::<crate::cells::messages::RequestCellDestroyed>()
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
            .init_resource::<SelectedBreaker>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<bevy::input::keyboard::KeyboardInput>()
            .add_plugins(crate::input::InputPlugin)
            .add_message::<crate::bolt::messages::BoltHitBreaker>()
            .add_message::<crate::bolt::messages::BoltHitCell>()
            .add_message::<crate::bolt::messages::BoltHitWall>()
            .add_message::<crate::bolt::messages::BoltLost>()
            .add_message::<crate::cells::messages::CellDestroyedAt>()
            .add_message::<crate::cells::messages::RequestCellDestroyed>()
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
