//! `ChainLightningConfig` — chain lightning arcs between cells.

use std::collections::HashSet;

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::Position2D;
use rantzsoft_stateflow::CleanupOnExit;
use serde::{Deserialize, Serialize};

use super::components::*;
use crate::{
    bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE},
    effect_v3::{
        components::EffectSourceChip, effects::DamageBoostConfig, stacking::EffectStack,
        traits::Fireable,
    },
    state::types::NodeState,
};

/// Configuration for chain lightning that arcs between nearby cells.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChainLightningConfig {
    /// Number of times the lightning jumps between cells.
    pub arcs:        u32,
    /// Maximum distance each arc can jump to find a new target.
    pub range:       OrderedFloat<f32>,
    /// Multiplier applied to base damage for each arc hit.
    pub damage_mult: OrderedFloat<f32>,
    /// How fast each lightning arc travels between cells in world units per second.
    pub arc_speed:   OrderedFloat<f32>,
}

impl Fireable for ChainLightningConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        let pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);
        let base_damage = world
            .get::<BoltBaseDamage>(entity)
            .map_or(DEFAULT_BOLT_BASE_DAMAGE, |d| d.0);
        let damage_boost = world
            .get::<EffectStack<DamageBoostConfig>>(entity)
            .map_or(1.0, EffectStack::aggregate);

        let chip = EffectSourceChip::from_source(source);

        world.spawn((
            ChainLightningChain {
                remaining_jumps: self.arcs,
                damage:          base_damage * self.damage_mult.0 * damage_boost,
                hit_set:         HashSet::new(),
                state:           ChainState::Idle,
                range:           self.range.0,
                arc_speed:       self.arc_speed.0,
                source_pos:      pos,
            },
            chip,
            CleanupOnExit::<NodeState>::default(),
        ));
    }

    fn register(app: &mut App) {
        use super::systems::tick_chain_lightning;
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            tick_chain_lightning.in_set(EffectV3Systems::Tick),
        );
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;
    use rantzsoft_spatial2d::components::Position2D;
    use rantzsoft_stateflow::CleanupOnExit;

    use super::*;
    use crate::{
        bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE},
        effect_v3::{
            components::EffectSourceChip,
            effects::DamageBoostConfig,
            stacking::EffectStack,
            traits::{Fireable, Reversible},
        },
        state::types::NodeState,
    };

    fn make_config() -> ChainLightningConfig {
        ChainLightningConfig {
            arcs:        3,
            range:       OrderedFloat(100.0),
            damage_mult: OrderedFloat(1.5),
            arc_speed:   OrderedFloat(500.0),
        }
    }

    // ── C2: ChainLightning base damage reads BoltBaseDamage from source entity ──

    #[test]
    fn chain_lightning_uses_bolt_base_damage_from_source_entity() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(20.0), Position2D(Vec2::new(50.0, 50.0))))
            .id();

        make_config().fire(source, "zap", &mut world);
        world.flush();

        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(chains.len(), 1, "expected 1 chain lightning entity");
        let expected_damage = 20.0 * 1.5;
        assert!(
            (chains[0].damage - expected_damage).abs() < f32::EPSILON,
            "chain damage should be 20.0 * 1.5 = {expected_damage}, got {}",
            chains[0].damage,
        );
    }

    #[test]
    fn chain_lightning_zero_bolt_base_damage() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(0.0), Position2D(Vec2::new(50.0, 50.0))))
            .id();

        make_config().fire(source, "zap", &mut world);
        world.flush();

        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(chains.len(), 1);
        assert!(
            chains[0].damage.abs() < f32::EPSILON,
            "chain damage should be 0.0 * 1.5 = 0.0, got {}",
            chains[0].damage,
        );
    }

    #[test]
    fn chain_lightning_falls_back_to_default_when_bolt_base_damage_absent() {
        let mut world = World::new();
        let source = world.spawn(Position2D(Vec2::new(50.0, 50.0))).id();

        make_config().fire(source, "zap", &mut world);
        world.flush();

        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(chains.len(), 1);
        let expected_damage = DEFAULT_BOLT_BASE_DAMAGE * 1.5;
        assert!(
            (chains[0].damage - expected_damage).abs() < f32::EPSILON,
            "chain damage should fall back to DEFAULT_BOLT_BASE_DAMAGE * 1.5 = {expected_damage}, got {}",
            chains[0].damage,
        );
    }

    #[test]
    fn chain_lightning_zero_damage_mult_produces_zero_damage() {
        let mut world = World::new();
        let source = world.spawn(Position2D(Vec2::new(50.0, 50.0))).id();

        let config = ChainLightningConfig {
            arcs:        3,
            range:       OrderedFloat(100.0),
            damage_mult: OrderedFloat(0.0),
            arc_speed:   OrderedFloat(500.0),
        };
        config.fire(source, "zap", &mut world);
        world.flush();

        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(chains.len(), 1);
        assert!(
            chains[0].damage.abs() < f32::EPSILON,
            "chain damage should be 10.0 * 0.0 = 0.0, got {}",
            chains[0].damage,
        );
    }

    // ── Spawn-time component presence ────────────────────────────────────

    #[test]
    fn chain_lightning_spawns_entity_with_cleanup_on_exit_node_state() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::new(50.0, 50.0))))
            .id();

        make_config().fire(source, "zap", &mut world);
        world.flush();

        let cleanup_count = world
            .query::<&CleanupOnExit<NodeState>>()
            .iter(&world)
            .count();
        assert_eq!(
            cleanup_count, 1,
            "expected exactly 1 entity with CleanupOnExit<NodeState>"
        );
    }

    #[test]
    fn chain_lightning_spawns_with_idle_state() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::new(50.0, 50.0))))
            .id();

        make_config().fire(source, "zap", &mut world);
        world.flush();

        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(chains.len(), 1);
        assert!(
            matches!(chains[0].state, ChainState::Idle),
            "chain state should be Idle, got {:?}",
            chains[0].state,
        );
    }

    #[test]
    fn chain_lightning_spawns_with_empty_hit_set() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::new(50.0, 50.0))))
            .id();

        make_config().fire(source, "zap", &mut world);
        world.flush();

        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(chains.len(), 1);
        assert!(
            chains[0].hit_set.is_empty(),
            "hit_set should be empty at spawn"
        );
    }

    // ── remaining_jumps ──────────────────────────────────────────────────

    #[test]
    fn chain_lightning_remaining_jumps_equals_config_arcs() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::new(50.0, 50.0))))
            .id();

        let config = ChainLightningConfig {
            arcs:        5,
            range:       OrderedFloat(100.0),
            damage_mult: OrderedFloat(1.5),
            arc_speed:   OrderedFloat(500.0),
        };
        config.fire(source, "zap", &mut world);
        world.flush();

        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(chains.len(), 1);
        assert_eq!(
            chains[0].remaining_jumps, 5,
            "remaining_jumps should match config arcs value"
        );
    }

    #[test]
    fn chain_lightning_zero_arcs_produces_zero_remaining_jumps() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::new(50.0, 50.0))))
            .id();

        let config = ChainLightningConfig {
            arcs:        0,
            range:       OrderedFloat(100.0),
            damage_mult: OrderedFloat(1.5),
            arc_speed:   OrderedFloat(500.0),
        };
        config.fire(source, "zap", &mut world);
        world.flush();

        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(chains.len(), 1);
        assert_eq!(
            chains[0].remaining_jumps, 0,
            "zero arcs config should produce zero remaining_jumps"
        );
    }

    // ── range and arc_speed ──────────────────────────────────────────────

    #[test]
    fn chain_lightning_copies_range_and_arc_speed_from_config() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::new(50.0, 50.0))))
            .id();

        let config = ChainLightningConfig {
            arcs:        3,
            range:       OrderedFloat(150.0),
            damage_mult: OrderedFloat(1.5),
            arc_speed:   OrderedFloat(750.0),
        };
        config.fire(source, "zap", &mut world);
        world.flush();

        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(chains.len(), 1);
        assert!(
            (chains[0].range - 150.0).abs() < f32::EPSILON,
            "range should be 150.0, got {}",
            chains[0].range,
        );
        assert!(
            (chains[0].arc_speed - 750.0).abs() < f32::EPSILON,
            "arc_speed should be 750.0, got {}",
            chains[0].arc_speed,
        );
    }

    // ── source_pos snapshot ──────────────────────────────────────────────

    #[test]
    fn chain_lightning_snapshots_position_from_source_entity() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::new(123.0, 456.0))))
            .id();

        make_config().fire(source, "zap", &mut world);
        world.flush();

        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(chains.len(), 1);
        assert_eq!(
            chains[0].source_pos,
            Vec2::new(123.0, 456.0),
            "source_pos should snapshot Position2D from source entity"
        );
    }

    #[test]
    fn chain_lightning_source_pos_falls_back_to_zero_without_position() {
        let mut world = World::new();
        let source = world.spawn(BoltBaseDamage(10.0)).id();

        make_config().fire(source, "zap", &mut world);
        world.flush();

        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(chains.len(), 1);
        assert_eq!(
            chains[0].source_pos,
            Vec2::ZERO,
            "source_pos should fall back to Vec2::ZERO when Position2D absent"
        );
    }

    // ── EffectSourceChip stamping ────────────────────────────────────────

    #[test]
    fn chain_lightning_stamps_effect_source_chip_from_source_name() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::new(50.0, 50.0))))
            .id();

        make_config().fire(source, "lightning_chip", &mut world);
        world.flush();

        let chips: Vec<&EffectSourceChip> =
            world.query::<&EffectSourceChip>().iter(&world).collect();
        // Filter to only non-DamageBoost chips (DamageBoost does not spawn EffectSourceChip)
        assert_eq!(chips.len(), 1, "expected 1 EffectSourceChip on the chain");
        assert_eq!(
            chips[0].0,
            Some("lightning_chip".to_string()),
            "EffectSourceChip should carry the source name"
        );
    }

    #[test]
    fn chain_lightning_empty_source_produces_none_effect_source_chip() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::new(50.0, 50.0))))
            .id();

        make_config().fire(source, "", &mut world);
        world.flush();

        let chips: Vec<&EffectSourceChip> =
            world.query::<&EffectSourceChip>().iter(&world).collect();
        assert_eq!(chips.len(), 1, "expected 1 EffectSourceChip on the chain");
        assert!(
            chips[0].0.is_none(),
            "empty source string should map to EffectSourceChip(None)"
        );
    }

    // ── Despawned source entity ──────────────────────────────────────────

    #[test]
    fn chain_lightning_fire_on_despawned_entity_spawns_with_fallbacks() {
        let mut world = World::new();
        let source = world.spawn_empty().id();
        world.despawn(source);

        make_config().fire(source, "zap", &mut world);
        world.flush();

        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(
            chains.len(),
            1,
            "chain should still be spawned even with despawned source"
        );
        assert_eq!(
            chains[0].source_pos,
            Vec2::ZERO,
            "source_pos should fall back to Vec2::ZERO for despawned entity"
        );
        // DEFAULT_BOLT_BASE_DAMAGE * damage_mult * 1.0 (no DamageBoost)
        let expected_damage = DEFAULT_BOLT_BASE_DAMAGE * 1.5;
        assert!(
            (chains[0].damage - expected_damage).abs() < f32::EPSILON,
            "damage should use DEFAULT_BOLT_BASE_DAMAGE fallback: expected {expected_damage}, got {}",
            chains[0].damage,
        );
    }

    // ── DamageBoost snapshot ─────────────────────────────────────────────

    #[test]
    fn chain_lightning_includes_single_damage_boost_in_damage() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::ZERO)))
            .id();

        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(source, "amp", &mut world);

        make_config().fire(source, "zap", &mut world);
        world.flush();

        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(chains.len(), 1);
        // 10.0 * 1.5 * 2.0 == 30.0
        let expected_damage = 30.0;
        assert!(
            (chains[0].damage - expected_damage).abs() < 1e-5,
            "damage should be 10.0 * 1.5 * 2.0 = {expected_damage}, got {}",
            chains[0].damage,
        );
    }

    #[test]
    fn chain_lightning_includes_two_damage_boosts_as_product() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::ZERO)))
            .id();

        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(source, "amp_a", &mut world);
        DamageBoostConfig {
            multiplier: OrderedFloat(3.0),
        }
        .fire(source, "amp_b", &mut world);

        make_config().fire(source, "zap", &mut world);
        world.flush();

        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(chains.len(), 1);
        // 10.0 * 1.5 * (2.0 * 3.0) == 10.0 * 1.5 * 6.0 == 90.0
        let expected_damage = 90.0;
        assert!(
            (chains[0].damage - expected_damage).abs() < 1e-5,
            "damage should be 10.0 * 1.5 * 6.0 = {expected_damage}, got {}",
            chains[0].damage,
        );
    }

    #[test]
    fn chain_lightning_damage_boost_snapshot_frozen_at_fire_time() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::ZERO)))
            .id();

        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(source, "amp", &mut world);

        make_config().fire(source, "zap", &mut world);
        world.flush();

        // Reverse the DamageBoost after chain lightning has already fired
        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .reverse(source, "amp", &mut world);

        // Verify the source entity's stack is now empty
        let stack = world
            .get::<EffectStack<DamageBoostConfig>>(source)
            .expect("stack component should still exist after reverse");
        assert_eq!(stack.len(), 0, "source stack should be empty after reverse");

        // Chain's damage should still reflect the snapshot taken at fire time
        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(chains.len(), 1);
        let expected_damage = 30.0; // 10.0 * 1.5 * 2.0
        assert!(
            (chains[0].damage - expected_damage).abs() < 1e-5,
            "damage should be frozen at {expected_damage} despite boost reversal, got {}",
            chains[0].damage,
        );
    }

    #[test]
    fn chain_lightning_no_damage_boost_stack_defaults_multiplier_to_one() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::ZERO)))
            .id();

        // No DamageBoostConfig fired — no EffectStack<DamageBoostConfig> component
        make_config().fire(source, "zap", &mut world);
        world.flush();

        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(chains.len(), 1);
        // 10.0 * 1.5 * 1.0 == 15.0
        let expected_damage = 15.0;
        assert!(
            (chains[0].damage - expected_damage).abs() < 1e-5,
            "damage should be 10.0 * 1.5 * 1.0 = {expected_damage} without DamageBoost, got {}",
            chains[0].damage,
        );
    }
}
