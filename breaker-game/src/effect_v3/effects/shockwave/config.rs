//! `ShockwaveConfig` — expanding damage shockwave.

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

/// Configuration for an expanding radial shockwave that damages cells.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShockwaveConfig {
    /// How far the shockwave ring expands before disappearing.
    pub base_range:      OrderedFloat<f32>,
    /// Extra range added per stack beyond the first.
    pub range_per_level: OrderedFloat<f32>,
    /// Current stack count — effective range is `base_range` + `range_per_level` * (stacks - 1).
    pub stacks:          u32,
    /// How fast the ring expands outward in world units per second.
    pub speed:           OrderedFloat<f32>,
}

impl Fireable for ShockwaveConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        // Snapshot position from the source entity
        let pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);

        // Snapshot damage multiplier from source entity's active boosts
        let damage_mult = world
            .get::<EffectStack<DamageBoostConfig>>(entity)
            .map_or(1.0, EffectStack::aggregate);

        // Calculate effective max radius from stacking
        let stacks_f32 = self.stacks.saturating_sub(1) as f32;
        let max_radius = self
            .range_per_level
            .0
            .mul_add(stacks_f32, self.base_range.0);

        let chip = EffectSourceChip::from_source(source);

        // Spawn the shockwave entity
        world.spawn((
            ShockwaveSource,
            ShockwaveRadius(0.0),
            ShockwaveMaxRadius(max_radius),
            ShockwaveSpeed(self.speed.0),
            ShockwaveDamaged(HashSet::new()),
            ShockwaveBaseDamage(
                world
                    .get::<BoltBaseDamage>(entity)
                    .map_or(DEFAULT_BOLT_BASE_DAMAGE, |d| d.0),
            ),
            ShockwaveDamageMultiplier(damage_mult),
            Position2D(pos),
            chip,
            CleanupOnExit::<NodeState>::default(),
        ));
    }

    fn register(app: &mut App) {
        use super::systems::*;
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            (
                tick_shockwave,
                apply_shockwave_damage,
                despawn_finished_shockwave,
            )
                .chain()
                .in_set(EffectV3Systems::Tick),
        );
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;
    use rantzsoft_spatial2d::components::Position2D;

    use super::*;
    use crate::{
        bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE},
        effect_v3::{effects::DamageBoostConfig, stacking::EffectStack, traits::Fireable},
    };

    fn make_config() -> ShockwaveConfig {
        ShockwaveConfig {
            base_range:      OrderedFloat(64.0),
            range_per_level: OrderedFloat(16.0),
            stacks:          1,
            speed:           OrderedFloat(200.0),
        }
    }

    // ── C1: Shockwave base damage reads BoltBaseDamage from source entity ──

    #[test]
    fn shockwave_uses_bolt_base_damage_from_source_entity() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(25.0), Position2D(Vec2::new(100.0, 200.0))))
            .id();

        make_config().fire(source, "test_chip", &mut world);
        world.flush();

        let base_dmg: Vec<f32> = world
            .query::<&ShockwaveBaseDamage>()
            .iter(&world)
            .map(|d| d.0)
            .collect();
        assert_eq!(base_dmg.len(), 1, "expected 1 shockwave entity");
        assert!(
            (base_dmg[0] - 25.0).abs() < f32::EPSILON,
            "shockwave base damage should be 25.0 (from BoltBaseDamage), got {}",
            base_dmg[0],
        );
    }

    #[test]
    fn shockwave_uses_zero_bolt_base_damage() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(0.0), Position2D(Vec2::new(100.0, 200.0))))
            .id();

        make_config().fire(source, "test_chip", &mut world);
        world.flush();

        let base_dmg: Vec<f32> = world
            .query::<&ShockwaveBaseDamage>()
            .iter(&world)
            .map(|d| d.0)
            .collect();
        assert_eq!(base_dmg.len(), 1);
        assert!(
            base_dmg[0].abs() < f32::EPSILON,
            "shockwave base damage should be 0.0 for BoltBaseDamage(0.0), got {}",
            base_dmg[0],
        );
    }

    #[test]
    fn shockwave_falls_back_to_default_when_bolt_base_damage_absent() {
        let mut world = World::new();
        let source = world.spawn(Position2D(Vec2::new(100.0, 200.0))).id();

        make_config().fire(source, "test_chip", &mut world);
        world.flush();

        let base_dmg: Vec<f32> = world
            .query::<&ShockwaveBaseDamage>()
            .iter(&world)
            .map(|d| d.0)
            .collect();
        assert_eq!(base_dmg.len(), 1);
        assert!(
            (base_dmg[0] - DEFAULT_BOLT_BASE_DAMAGE).abs() < f32::EPSILON,
            "shockwave base damage should fall back to DEFAULT_BOLT_BASE_DAMAGE ({DEFAULT_BOLT_BASE_DAMAGE}), got {}",
            base_dmg[0],
        );
    }

    #[test]
    fn shockwave_falls_back_when_source_entity_despawned() {
        let mut world = World::new();
        let source = world.spawn_empty().id();
        world.despawn(source);

        make_config().fire(source, "test_chip", &mut world);
        world.flush();

        let base_dmg: Vec<f32> = world
            .query::<&ShockwaveBaseDamage>()
            .iter(&world)
            .map(|d| d.0)
            .collect();
        assert_eq!(base_dmg.len(), 1);
        assert!(
            (base_dmg[0] - DEFAULT_BOLT_BASE_DAMAGE).abs() < f32::EPSILON,
            "shockwave should fall back to DEFAULT_BOLT_BASE_DAMAGE when entity despawned, got {}",
            base_dmg[0],
        );
    }

    // ── E. range formula ──────────────────────────────────────────────────

    // #19
    #[test]
    fn shockwave_max_radius_equals_base_range_for_stacks_one() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::ZERO)))
            .id();

        let config = ShockwaveConfig {
            base_range:      OrderedFloat(64.0),
            range_per_level: OrderedFloat(16.0),
            stacks:          1,
            speed:           OrderedFloat(200.0),
        };
        config.fire(source, "test_chip", &mut world);
        world.flush();

        let max_radii: Vec<f32> = world
            .query::<&ShockwaveMaxRadius>()
            .iter(&world)
            .map(|r| r.0)
            .collect();
        assert_eq!(max_radii.len(), 1, "expected 1 shockwave entity");
        assert!(
            (max_radii[0] - 64.0).abs() < f32::EPSILON,
            "stacks==1 must yield max_radius == base_range (64.0), got {}",
            max_radii[0],
        );
    }

    // #20
    #[test]
    fn shockwave_max_radius_adds_range_per_level_for_stacks_three() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::ZERO)))
            .id();

        let config = ShockwaveConfig {
            base_range:      OrderedFloat(64.0),
            range_per_level: OrderedFloat(16.0),
            stacks:          3,
            speed:           OrderedFloat(200.0),
        };
        config.fire(source, "test_chip", &mut world);
        world.flush();

        let max_radii: Vec<f32> = world
            .query::<&ShockwaveMaxRadius>()
            .iter(&world)
            .map(|r| r.0)
            .collect();
        assert_eq!(max_radii.len(), 1);
        // 64.0 + (3 - 1) * 16.0 == 64.0 + 32.0 == 96.0
        assert!(
            (max_radii[0] - 96.0).abs() < f32::EPSILON,
            "stacks==3 must yield max_radius == 96.0, got {}",
            max_radii[0],
        );
    }

    // #21
    #[test]
    fn shockwave_max_radius_uses_saturating_sub_for_stacks_zero() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::ZERO)))
            .id();

        let config = ShockwaveConfig {
            base_range:      OrderedFloat(64.0),
            range_per_level: OrderedFloat(16.0),
            stacks:          0,
            speed:           OrderedFloat(200.0),
        };
        config.fire(source, "test_chip", &mut world);
        world.flush();

        let max_radii: Vec<f32> = world
            .query::<&ShockwaveMaxRadius>()
            .iter(&world)
            .map(|r| r.0)
            .collect();
        assert_eq!(max_radii.len(), 1);
        // saturating_sub(1) on 0u32 -> 0 -> (stacks_f32 == 0.0) -> max_radius == base_range
        assert!(
            (max_radii[0] - 64.0).abs() < f32::EPSILON,
            "stacks==0 must collapse to base_range via saturating_sub, got {}",
            max_radii[0],
        );
    }

    // ── F. damage multiplier snapshot ─────────────────────────────────────

    // #22
    #[test]
    fn shockwave_damage_multiplier_snapshots_single_entry_stack() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::ZERO)))
            .id();

        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(source, "amp", &mut world);

        make_config().fire(source, "test_chip", &mut world);
        world.flush();

        let mults: Vec<f32> = world
            .query::<&ShockwaveDamageMultiplier>()
            .iter(&world)
            .map(|m| m.0)
            .collect();
        assert_eq!(mults.len(), 1);
        assert!(
            (mults[0] - 2.0).abs() < 1e-5,
            "single-entry stack aggregate should be 2.0, got {}",
            mults[0],
        );

        let stack = world
            .get::<EffectStack<DamageBoostConfig>>(source)
            .expect("source should carry an EffectStack<DamageBoostConfig>");
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 2.0).abs() < 1e-5);
    }

    // #23
    #[test]
    fn shockwave_damage_multiplier_snapshots_two_entry_stack_as_product() {
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

        make_config().fire(source, "test_chip", &mut world);
        world.flush();

        let mults: Vec<f32> = world
            .query::<&ShockwaveDamageMultiplier>()
            .iter(&world)
            .map(|m| m.0)
            .collect();
        assert_eq!(mults.len(), 1);
        // 2.0 * 3.0 == 6.0 — unambiguous two-entry product.
        assert!(
            (mults[0] - 6.0).abs() < 1e-5,
            "two-entry stack aggregate should be 2.0 * 3.0 == 6.0, got {}",
            mults[0],
        );

        let stack = world
            .get::<EffectStack<DamageBoostConfig>>(source)
            .expect("source should carry an EffectStack<DamageBoostConfig>");
        assert_eq!(stack.len(), 2);
        assert!((stack.aggregate() - 6.0).abs() < 1e-5);
    }

    // #24
    #[test]
    fn shockwave_damage_multiplier_defaults_to_one_without_stack() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::ZERO)))
            .id();

        // No DamageBoostConfig.fire(...) on the source — no stack component.
        make_config().fire(source, "test_chip", &mut world);
        world.flush();

        let mults: Vec<f32> = world
            .query::<&ShockwaveDamageMultiplier>()
            .iter(&world)
            .map(|m| m.0)
            .collect();
        assert_eq!(mults.len(), 1);
        assert!(
            (mults[0] - 1.0).abs() < 1e-5,
            "missing-stack fallback should yield multiplier 1.0, got {}",
            mults[0],
        );
    }
}
