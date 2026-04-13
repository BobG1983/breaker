//! `GravityWellConfig` — gravity well pulling bolts.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::Position2D;
use rantzsoft_stateflow::CleanupOnExit;
use serde::{Deserialize, Serialize};

use super::components::*;
use crate::{
    effect_v3::{components::EffectSourceChip, traits::Fireable},
    state::types::NodeState,
};

/// Configuration for a point attractor that pulls bolts toward it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GravityWellConfig {
    /// How strongly bolts are pulled toward the well center per tick.
    pub strength: OrderedFloat<f32>,
    /// How long the well exists before despawning.
    pub duration: OrderedFloat<f32>,
    /// How far from the well center bolts are affected.
    pub radius:   OrderedFloat<f32>,
    /// Maximum active wells per owner entity — oldest removed when exceeded.
    pub max:      u32,
}

impl Fireable for GravityWellConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        let pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);

        let chip = EffectSourceChip(if source.is_empty() {
            None
        } else {
            Some(source.to_owned())
        });

        // Calculate next spawn order BEFORE eviction (so evicted well's order is included).
        let next_order = world
            .query_filtered::<&GravityWellSpawnOrder, With<GravityWellSource>>()
            .iter(world)
            .map(|s| s.0)
            .max()
            .map_or(0, |m| m + 1);

        // Enforce max active wells per owner — despawn oldest by spawn order.
        let mut owned: Vec<(Entity, u32)> = world
            .query_filtered::<(Entity, &GravityWellOwner, &GravityWellSpawnOrder), With<GravityWellSource>>()
            .iter(world)
            .filter(|(_, owner, _)| owner.0 == entity)
            .map(|(e, _, order)| (e, order.0))
            .collect();
        owned.sort_by_key(|(_, order)| *order);

        if owned.len() >= self.max as usize
            && let Some(&(oldest, _)) = owned.first()
        {
            world.despawn(oldest);
        }

        world.spawn((
            GravityWellSource,
            GravityWellOwner(entity),
            GravityWellStrength(self.strength.0),
            GravityWellRadius(self.radius.0),
            GravityWellLifetime(self.duration.0),
            GravityWellSpawnOrder(next_order),
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
            (tick_gravity_well, despawn_expired_wells)
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
    use crate::effect_v3::traits::Fireable;

    fn make_config(max: u32) -> GravityWellConfig {
        GravityWellConfig {
            strength: OrderedFloat(100.0),
            duration: OrderedFloat(5.0),
            radius: OrderedFloat(80.0),
            max,
        }
    }

    // ── C7: GravityWell FIFO eviction by spawn order ───────────────────────

    #[test]
    fn gravity_well_first_fire_spawns_with_spawn_order_zero() {
        let mut world = World::new();
        let owner = world.spawn(Position2D(Vec2::new(10.0, 20.0))).id();

        make_config(5).fire(owner, "gravity", &mut world);
        world.flush();

        let wells: Vec<&GravityWellSpawnOrder> = world
            .query_filtered::<&GravityWellSpawnOrder, With<GravityWellSource>>()
            .iter(&world)
            .collect();
        assert_eq!(wells.len(), 1, "expected 1 gravity well");
        assert_eq!(
            wells[0].0, 0,
            "first gravity well should have GravityWellSpawnOrder(0), got {}",
            wells[0].0,
        );
    }

    #[test]
    fn gravity_well_evicts_oldest_by_spawn_order_when_max_reached() {
        let mut world = World::new();
        let owner = world.spawn(Position2D(Vec2::new(10.0, 20.0))).id();

        // IMPORTANT: Spawn in REVERSE order — well_2 (spawn order 1) is inserted
        // first, well_1 (spawn order 0) is inserted second. This ensures the test
        // catches the bug where `owned.first()` evicts by insertion order instead of
        // by lowest GravityWellSpawnOrder.
        let well_2 = world
            .spawn((
                GravityWellSource,
                GravityWellOwner(owner),
                GravityWellStrength(100.0),
                GravityWellRadius(80.0),
                GravityWellLifetime(5.0),
                GravityWellSpawnOrder(1),
                Position2D(Vec2::new(10.0, 20.0)),
            ))
            .id();
        let well_1 = world
            .spawn((
                GravityWellSource,
                GravityWellOwner(owner),
                GravityWellStrength(100.0),
                GravityWellRadius(80.0),
                GravityWellLifetime(5.0),
                GravityWellSpawnOrder(0),
                Position2D(Vec2::new(10.0, 20.0)),
            ))
            .id();

        // Fire with max=2 — should evict well_1 (lowest spawn order 0), NOT well_2.
        make_config(2).fire(owner, "gravity", &mut world);
        world.flush();

        // well_1 (spawn order 0) should be despawned — it's the oldest by spawn order.
        assert!(
            world.get_entity(well_1).is_err(),
            "well_1 (spawn order 0) should be despawned as oldest by spawn order"
        );
        // well_2 (spawn order 1) should still be alive.
        assert!(
            world.get_entity(well_2).is_ok(),
            "well_2 (spawn order 1) should still exist"
        );

        // New well should exist with spawn order 2.
        let spawn_orders: Vec<u32> = world
            .query_filtered::<&GravityWellSpawnOrder, With<GravityWellSource>>()
            .iter(&world)
            .map(|s| s.0)
            .collect();
        assert_eq!(spawn_orders.len(), 2, "should have 2 wells total");
        assert!(
            spawn_orders.contains(&1),
            "well_2 with spawn order 1 should exist"
        );
        assert!(
            spawn_orders.contains(&2),
            "new well with spawn order 2 should exist"
        );
    }

    #[test]
    fn gravity_well_spawn_order_increments_across_fires() {
        let mut world = World::new();
        let owner = world.spawn(Position2D(Vec2::new(10.0, 20.0))).id();

        make_config(5).fire(owner, "gravity", &mut world);
        world.flush();
        make_config(5).fire(owner, "gravity", &mut world);
        world.flush();
        make_config(5).fire(owner, "gravity", &mut world);
        world.flush();

        let mut spawn_orders: Vec<u32> = world
            .query_filtered::<&GravityWellSpawnOrder, With<GravityWellSource>>()
            .iter(&world)
            .map(|s| s.0)
            .collect();
        spawn_orders.sort_unstable();

        assert_eq!(spawn_orders.len(), 3, "should have 3 gravity wells");
        assert_eq!(
            spawn_orders,
            vec![0, 1, 2],
            "spawn orders should be monotonically increasing: 0, 1, 2"
        );
    }

    #[test]
    fn gravity_well_max_one_evicts_on_each_fire() {
        let mut world = World::new();
        let owner = world.spawn(Position2D(Vec2::new(10.0, 20.0))).id();

        make_config(1).fire(owner, "gravity", &mut world);
        world.flush();
        make_config(1).fire(owner, "gravity", &mut world);
        world.flush();
        make_config(1).fire(owner, "gravity", &mut world);
        world.flush();

        let spawn_orders: Vec<u32> = world
            .query_filtered::<&GravityWellSpawnOrder, With<GravityWellSource>>()
            .iter(&world)
            .map(|s| s.0)
            .collect();
        assert_eq!(spawn_orders.len(), 1, "max=1 should have exactly 1 well");
        assert_eq!(
            spawn_orders[0], 2,
            "after 3 fires with max=1, only spawn order 2 should remain"
        );
    }

    #[test]
    fn gravity_well_does_not_evict_another_owners_wells() {
        let mut world = World::new();
        let owner_a = world.spawn(Position2D(Vec2::new(10.0, 20.0))).id();
        let owner_b = world.spawn(Position2D(Vec2::new(30.0, 40.0))).id();

        // B's existing well.
        let b_well = world
            .spawn((
                GravityWellSource,
                GravityWellOwner(owner_b),
                GravityWellStrength(100.0),
                GravityWellRadius(80.0),
                GravityWellLifetime(5.0),
                GravityWellSpawnOrder(0),
                Position2D(Vec2::new(30.0, 40.0)),
            ))
            .id();

        // A fires with max=1 — this is A's first well, so no eviction.
        make_config(1).fire(owner_a, "gravity", &mut world);
        world.flush();

        // B's well should be untouched.
        assert!(
            world.get_entity(b_well).is_ok(),
            "B's well should not be evicted by A's fire"
        );

        let total_wells = world
            .query_filtered::<Entity, With<GravityWellSource>>()
            .iter(&world)
            .count();
        assert_eq!(
            total_wells, 2,
            "should have 2 wells total (B's existing + A's new)"
        );
    }
}
