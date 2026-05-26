//! Phantom-bolt component vocabulary and canonical switch functions.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::{
    bolt::components::Bolt,
    shared::{Lifespan, PhantomFlicker},
};

/// Marker identifying an entity as a phantom bolt.
///
/// Single source of truth; the old declaration at
/// `effect_v3/effects/phantom_bolt/components.rs` is a `pub use` re-export of
/// this type.
#[derive(Component, Debug, Clone, Copy)]
pub struct PhantomBolt;

/// Identifies the origin of a phantom bolt for dedup and attribution.
///
/// NOT `Copy` because the `Chip` variant owns a `String`.
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhantomDedupKey {
    /// Afterimage phantom — spawned from a Perfect Bump on this real bolt.
    Bolt(Entity),
    /// Chip-effect phantom — spawned by `SpawnPhantomConfig::fire`.
    Chip {
        /// Name of the chip effect that created this phantom.
        chip:       String,
        /// Entity the chip effect fired from.
        fired_from: Entity,
    },
}

/// Per-phantom-lifetime set of cell entities already damaged by this phantom.
///
/// Inserted empty by `Bolt::become_phantom`. Mutated by `bolt_cell_collision`
/// in Wave 3B. Removed by `PhantomBolt::become_normal`.
#[derive(Component, Debug, Clone, Default)]
pub struct PhantomDamagedCells(pub HashSet<Entity>);

/// Decides what happens when `Lifespan` reaches zero.
///
/// Read by `tick_bolt_lifespan` in Wave 3A.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifetimeEndBehavior {
    /// Emit `DespawnEntity` to the death pipeline.
    Despawn,
    /// Strip the phantom component set; bolt continues as normal.
    RevertToNormalBolt,
}

/// Parameter bundle threaded through the bolt builder's `.phantom(...)` method.
///
/// NOT a `Component` — the dedup key flows out as the actual
/// `PhantomDedupKey` component via `Bolt::become_phantom`.
#[derive(Debug, Clone)]
pub struct PhantomParams {
    /// Identifier used to dedup phantom spawns per source.
    pub dedup_key: PhantomDedupKey,
}

impl Bolt {
    /// Stamps an entity as a phantom bolt.
    ///
    /// Inserts `PhantomBolt`, the supplied `PhantomDedupKey`, and an empty
    /// `PhantomDamagedCells` on `bolt`. No other components are touched.
    pub fn become_phantom(commands: &mut Commands, bolt: Entity, dedup_key: PhantomDedupKey) {
        commands
            .entity(bolt)
            .insert((PhantomBolt, dedup_key, PhantomDamagedCells::default()));
    }
}

impl PhantomBolt {
    /// Reverts a phantom bolt to a normal bolt.
    ///
    /// Removes `PhantomBolt`, `PhantomDedupKey`, `PhantomDamagedCells`,
    /// `PhantomFlicker`, `Lifespan`, and `LifetimeEndBehavior` from `phantom`.
    /// No other components are touched. Safe to call on entities that lack
    /// some or all of the six components.
    pub fn become_normal(commands: &mut Commands, phantom: Entity) {
        commands.entity(phantom).remove::<(
            Self,
            PhantomDedupKey,
            PhantomDamagedCells,
            PhantomFlicker,
            Lifespan,
            LifetimeEndBehavior,
        )>();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use bevy::prelude::*;

    use super::*;
    use crate::{
        bolt::components::{BoltBaseDamage, ExtraBolt, PiercingRemaining, PrimaryBolt},
        prelude::*,
        shared::{Lifespan, PhantomFlicker},
    };

    // ── helpers ─────────────────────────────────────────────────

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    // ── Behavior 1: become_phantom inserts PhantomBolt ──────────

    #[test]
    fn become_phantom_inserts_phantom_bolt_marker() {
        let mut app = test_app();
        let e = app.world_mut().spawn(Bolt).id();
        app.add_systems(Update, move |mut commands: Commands| {
            Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
        });
        app.update();
        assert!(
            app.world().get::<PhantomBolt>(e).is_some(),
            "PhantomBolt marker should be present after become_phantom"
        );
    }

    #[test]
    fn become_phantom_twice_is_idempotent_phantom_bolt_still_present() {
        let mut app = test_app();
        let e = app.world_mut().spawn(Bolt).id();
        app.add_systems(Update, move |mut commands: Commands| {
            Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
            Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
        });
        app.update();
        assert!(
            app.world().get::<PhantomBolt>(e).is_some(),
            "PhantomBolt should still be present after double become_phantom"
        );
    }

    // ── Behavior 2: become_phantom inserts PhantomDedupKey::Bolt ─

    #[test]
    fn become_phantom_inserts_dedup_key_bolt_variant() {
        let mut app = test_app();
        let e = app.world_mut().spawn(Bolt).id();
        app.add_systems(Update, move |mut commands: Commands| {
            Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
        });
        app.update();
        let key = app
            .world()
            .get::<PhantomDedupKey>(e)
            .expect("PhantomDedupKey should be present after become_phantom");
        assert_eq!(
            *key,
            PhantomDedupKey::Bolt(e),
            "PhantomDedupKey should be Bolt(e), got {key:?}"
        );
    }

    #[test]
    fn become_phantom_preserves_different_entity_in_bolt_key() {
        let mut app = test_app();
        let e = app.world_mut().spawn(Bolt).id();
        let other = app.world_mut().spawn_empty().id();
        app.add_systems(Update, move |mut commands: Commands| {
            Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(other));
        });
        app.update();
        let key = app
            .world()
            .get::<PhantomDedupKey>(e)
            .expect("PhantomDedupKey should be present");
        assert_eq!(
            *key,
            PhantomDedupKey::Bolt(other),
            "become_phantom must not rewrite the supplied entity in the key"
        );
    }

    // ── Behavior 3: become_phantom inserts PhantomDedupKey::Chip ──

    #[test]
    fn become_phantom_inserts_dedup_key_chip_variant() {
        let mut app = test_app();
        let e = app.world_mut().spawn(Bolt).id();
        let src = app.world_mut().spawn_empty().id();
        app.add_systems(Update, move |mut commands: Commands| {
            Bolt::become_phantom(
                &mut commands,
                e,
                PhantomDedupKey::Chip {
                    chip:       "phantom_bolt".to_string(),
                    fired_from: src,
                },
            );
        });
        app.update();
        let key = app
            .world()
            .get::<PhantomDedupKey>(e)
            .expect("PhantomDedupKey should be present");
        assert_eq!(
            *key,
            PhantomDedupKey::Chip {
                chip:       "phantom_bolt".to_string(),
                fired_from: src,
            },
            "PhantomDedupKey should be Chip variant with correct fields"
        );
    }

    #[test]
    fn become_phantom_preserves_empty_chip_string() {
        let mut app = test_app();
        let e = app.world_mut().spawn(Bolt).id();
        let src = app.world_mut().spawn_empty().id();
        app.add_systems(Update, move |mut commands: Commands| {
            Bolt::become_phantom(
                &mut commands,
                e,
                PhantomDedupKey::Chip {
                    chip:       String::new(),
                    fired_from: src,
                },
            );
        });
        app.update();
        let key = app
            .world()
            .get::<PhantomDedupKey>(e)
            .expect("PhantomDedupKey should be present");
        assert_eq!(
            *key,
            PhantomDedupKey::Chip {
                chip:       String::new(),
                fired_from: src,
            },
            "empty chip string should be preserved verbatim"
        );
    }

    // ── Behavior 4: become_phantom inserts empty PhantomDamagedCells ──

    #[test]
    fn become_phantom_inserts_empty_damaged_cells() {
        let mut app = test_app();
        let e = app.world_mut().spawn(Bolt).id();
        app.add_systems(Update, move |mut commands: Commands| {
            Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
        });
        app.update();
        let cells = app
            .world()
            .get::<PhantomDamagedCells>(e)
            .expect("PhantomDamagedCells should be present after become_phantom");
        assert!(
            cells.0.is_empty(),
            "PhantomDamagedCells should be an empty HashSet, got {} entries",
            cells.0.len()
        );
    }

    #[test]
    fn become_phantom_overwrites_existing_damaged_cells_with_empty_set() {
        let mut app = test_app();
        let cell_a = app.world_mut().spawn_empty().id();
        let cell_b = app.world_mut().spawn_empty().id();
        let mut pre_populated = HashSet::new();
        pre_populated.insert(cell_a);
        pre_populated.insert(cell_b);
        let e = app
            .world_mut()
            .spawn((Bolt, PhantomDamagedCells(pre_populated)))
            .id();
        app.add_systems(Update, move |mut commands: Commands| {
            Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
        });
        app.update();
        let cells = app
            .world()
            .get::<PhantomDamagedCells>(e)
            .expect("PhantomDamagedCells should be present after become_phantom");
        assert!(
            cells.0.is_empty(),
            "become_phantom should overwrite pre-existing PhantomDamagedCells with empty set"
        );
    }

    // ── Behavior 5: become_phantom does NOT insert PhantomFlicker/Lifespan/LifetimeEndBehavior ──

    #[test]
    fn become_phantom_does_not_insert_phantom_flicker() {
        let mut app = test_app();
        let e = app.world_mut().spawn(Bolt).id();
        app.add_systems(Update, move |mut commands: Commands| {
            Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
        });
        app.update();
        assert!(
            app.world().get::<PhantomBolt>(e).is_some(),
            "become_phantom did not execute — PhantomBolt not present"
        );
        assert!(
            app.world().get::<PhantomFlicker>(e).is_none(),
            "become_phantom must not insert PhantomFlicker"
        );
    }

    #[test]
    fn become_phantom_does_not_insert_lifespan() {
        let mut app = test_app();
        let e = app.world_mut().spawn(Bolt).id();
        app.add_systems(Update, move |mut commands: Commands| {
            Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
        });
        app.update();
        assert!(
            app.world().get::<PhantomBolt>(e).is_some(),
            "become_phantom did not execute — PhantomBolt not present"
        );
        assert!(
            app.world().get::<Lifespan>(e).is_none(),
            "become_phantom must not insert Lifespan"
        );
    }

    #[test]
    fn become_phantom_does_not_insert_lifetime_end_behavior() {
        let mut app = test_app();
        let e = app.world_mut().spawn(Bolt).id();
        app.add_systems(Update, move |mut commands: Commands| {
            Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
        });
        app.update();
        assert!(
            app.world().get::<PhantomBolt>(e).is_some(),
            "become_phantom did not execute — PhantomBolt not present"
        );
        assert!(
            app.world().get::<LifetimeEndBehavior>(e).is_none(),
            "become_phantom must not insert LifetimeEndBehavior"
        );
    }

    #[test]
    fn become_phantom_does_not_remove_preexisting_lifespan() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                Bolt,
                Lifespan { remaining: 2.5 },
                LifetimeEndBehavior::Despawn,
            ))
            .id();
        app.add_systems(Update, move |mut commands: Commands| {
            Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
        });
        app.update();
        assert!(
            app.world().get::<PhantomBolt>(e).is_some(),
            "become_phantom did not execute — PhantomBolt not present"
        );
        let lifespan = app
            .world()
            .get::<Lifespan>(e)
            .expect("pre-existing Lifespan should remain after become_phantom");
        assert!(
            (lifespan.remaining - 2.5).abs() < f32::EPSILON,
            "Lifespan.remaining should still be 2.5"
        );
        let leb = app
            .world()
            .get::<LifetimeEndBehavior>(e)
            .expect("pre-existing LifetimeEndBehavior should remain after become_phantom");
        assert_eq!(
            *leb,
            LifetimeEndBehavior::Despawn,
            "LifetimeEndBehavior should still be Despawn"
        );
    }

    // ── Behavior 6: become_phantom does NOT mutate role/velocity/position/cleanup ──

    #[test]
    fn become_phantom_preserves_primary_bolt_and_components() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                Bolt,
                PrimaryBolt,
                Velocity2D(Vec2::new(0.0, 400.0)),
                Position2D(Vec2::new(10.0, 20.0)),
                CleanupOnExit::<RunState>::default(),
                BoltBaseDamage(10.0),
                PiercingRemaining(3),
            ))
            .id();
        app.add_systems(Update, move |mut commands: Commands| {
            Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
        });
        app.update();
        assert!(
            app.world().get::<PhantomBolt>(e).is_some(),
            "become_phantom did not execute — PhantomBolt not present"
        );
        assert!(
            app.world().get::<PrimaryBolt>(e).is_some(),
            "PrimaryBolt should still be present"
        );
        let vel = app
            .world()
            .get::<Velocity2D>(e)
            .expect("Velocity2D should still be present");
        assert_eq!(
            vel.0,
            Vec2::new(0.0, 400.0),
            "Velocity2D should be unchanged"
        );
        let pos = app
            .world()
            .get::<Position2D>(e)
            .expect("Position2D should still be present");
        assert_eq!(
            pos.0,
            Vec2::new(10.0, 20.0),
            "Position2D should be unchanged"
        );
        assert!(
            app.world().get::<CleanupOnExit<RunState>>(e).is_some(),
            "CleanupOnExit<RunState> should still be present"
        );
        let dmg = app
            .world()
            .get::<BoltBaseDamage>(e)
            .expect("BoltBaseDamage should still be present");
        assert!(
            (dmg.0 - 10.0).abs() < f32::EPSILON,
            "BoltBaseDamage should still be 10.0"
        );
        let piercing = app
            .world()
            .get::<PiercingRemaining>(e)
            .expect("PiercingRemaining should still be present");
        assert_eq!(piercing.0, 3, "PiercingRemaining should still be 3");
    }

    #[test]
    fn become_phantom_preserves_extra_bolt_and_node_cleanup() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                Bolt,
                ExtraBolt,
                Velocity2D(Vec2::new(0.0, 400.0)),
                Position2D(Vec2::new(10.0, 20.0)),
                CleanupOnExit::<NodeState>::default(),
                BoltBaseDamage(10.0),
                PiercingRemaining(3),
            ))
            .id();
        app.add_systems(Update, move |mut commands: Commands| {
            Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
        });
        app.update();
        assert!(
            app.world().get::<PhantomBolt>(e).is_some(),
            "become_phantom did not execute — PhantomBolt not present"
        );
        assert!(
            app.world().get::<ExtraBolt>(e).is_some(),
            "ExtraBolt should still be present"
        );
        assert!(
            app.world().get::<CleanupOnExit<NodeState>>(e).is_some(),
            "CleanupOnExit<NodeState> should still be present"
        );
    }

    // ── Behavior 7: become_normal strips PhantomBolt ─────────────

    #[test]
    fn become_normal_strips_phantom_bolt_marker() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                Bolt,
                PhantomBolt,
                PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
                PhantomDamagedCells::default(),
            ))
            .id();
        // must patch key with real entity after spawn
        let key = PhantomDedupKey::Bolt(e);
        app.world_mut().entity_mut(e).insert(key);
        app.add_systems(Update, move |mut commands: Commands| {
            PhantomBolt::become_normal(&mut commands, e);
        });
        app.update();
        assert!(
            app.world().get::<PhantomBolt>(e).is_none(),
            "PhantomBolt should be removed after become_normal"
        );
    }

    #[test]
    fn become_normal_on_non_phantom_bolt_does_not_panic() {
        let mut app = test_app();
        let e = app.world_mut().spawn(Bolt).id();
        app.add_systems(Update, move |mut commands: Commands| {
            PhantomBolt::become_normal(&mut commands, e);
        });
        app.update();
        assert!(
            app.world().get::<Bolt>(e).is_some(),
            "Bolt marker should still be present"
        );
        assert!(
            app.world().get::<PhantomBolt>(e).is_none(),
            "PhantomBolt should remain absent"
        );
    }

    // ── Behavior 8: become_normal strips PhantomDedupKey ─────────

    #[test]
    fn become_normal_strips_dedup_key_bolt_variant() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                Bolt,
                PhantomBolt,
                PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
                PhantomDamagedCells::default(),
            ))
            .id();
        let key = PhantomDedupKey::Bolt(e);
        app.world_mut().entity_mut(e).insert(key);
        app.add_systems(Update, move |mut commands: Commands| {
            PhantomBolt::become_normal(&mut commands, e);
        });
        app.update();
        assert!(
            app.world().get::<PhantomDedupKey>(e).is_none(),
            "PhantomDedupKey should be removed after become_normal"
        );
    }

    #[test]
    fn become_normal_strips_dedup_key_chip_variant() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                Bolt,
                PhantomBolt,
                PhantomDedupKey::Chip {
                    chip:       "phantom_bolt".to_string(),
                    fired_from: Entity::PLACEHOLDER,
                },
                PhantomDamagedCells::default(),
            ))
            .id();
        let src = e;
        app.world_mut().entity_mut(e).insert(PhantomDedupKey::Chip {
            chip:       "phantom_bolt".to_string(),
            fired_from: src,
        });
        app.add_systems(Update, move |mut commands: Commands| {
            PhantomBolt::become_normal(&mut commands, e);
        });
        app.update();
        assert!(
            app.world().get::<PhantomDedupKey>(e).is_none(),
            "PhantomDedupKey Chip variant should be removed after become_normal"
        );
    }

    // ── Behavior 9: become_normal strips PhantomDamagedCells ─────

    #[test]
    fn become_normal_strips_empty_damaged_cells() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                Bolt,
                PhantomBolt,
                PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
                PhantomDamagedCells::default(),
            ))
            .id();
        app.world_mut()
            .entity_mut(e)
            .insert(PhantomDedupKey::Bolt(e));
        app.add_systems(Update, move |mut commands: Commands| {
            PhantomBolt::become_normal(&mut commands, e);
        });
        app.update();
        assert!(
            app.world().get::<PhantomDamagedCells>(e).is_none(),
            "PhantomDamagedCells should be removed after become_normal"
        );
    }

    #[test]
    fn become_normal_strips_populated_damaged_cells() {
        let mut app = test_app();
        let cell_a = app.world_mut().spawn_empty().id();
        let cell_b = app.world_mut().spawn_empty().id();
        let cell_c = app.world_mut().spawn_empty().id();
        let mut cells = HashSet::new();
        cells.insert(cell_a);
        cells.insert(cell_b);
        cells.insert(cell_c);
        let e = app
            .world_mut()
            .spawn((
                Bolt,
                PhantomBolt,
                PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
                PhantomDamagedCells(cells),
            ))
            .id();
        app.world_mut()
            .entity_mut(e)
            .insert(PhantomDedupKey::Bolt(e));
        app.add_systems(Update, move |mut commands: Commands| {
            PhantomBolt::become_normal(&mut commands, e);
        });
        app.update();
        assert!(
            app.world().get::<PhantomDamagedCells>(e).is_none(),
            "Populated PhantomDamagedCells should be fully removed after become_normal"
        );
    }

    // ── Behavior 10: become_normal strips PhantomFlicker ─────────

    #[test]
    fn become_normal_strips_phantom_flicker() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                Bolt,
                PhantomBolt,
                PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
                PhantomDamagedCells::default(),
                PhantomFlicker {
                    frequency: 4.0,
                    min_alpha: 0.3,
                },
            ))
            .id();
        app.world_mut()
            .entity_mut(e)
            .insert(PhantomDedupKey::Bolt(e));
        app.add_systems(Update, move |mut commands: Commands| {
            PhantomBolt::become_normal(&mut commands, e);
        });
        app.update();
        assert!(
            app.world().get::<PhantomFlicker>(e).is_none(),
            "PhantomFlicker should be removed after become_normal"
        );
    }

    #[test]
    fn become_normal_without_phantom_flicker_does_not_panic() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                Bolt,
                PhantomBolt,
                PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
                PhantomDamagedCells::default(),
            ))
            .id();
        app.world_mut()
            .entity_mut(e)
            .insert(PhantomDedupKey::Bolt(e));
        app.add_systems(Update, move |mut commands: Commands| {
            PhantomBolt::become_normal(&mut commands, e);
        });
        app.update();
        assert!(
            app.world().get::<PhantomFlicker>(e).is_none(),
            "PhantomFlicker should remain absent when not present"
        );
    }

    // ── Behavior 11: become_normal strips Lifespan ───────────────

    #[test]
    fn become_normal_strips_lifespan() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                Bolt,
                PhantomBolt,
                PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
                PhantomDamagedCells::default(),
                Lifespan { remaining: 2.5 },
            ))
            .id();
        app.world_mut()
            .entity_mut(e)
            .insert(PhantomDedupKey::Bolt(e));
        app.add_systems(Update, move |mut commands: Commands| {
            PhantomBolt::become_normal(&mut commands, e);
        });
        app.update();
        assert!(
            app.world().get::<Lifespan>(e).is_none(),
            "Lifespan should be removed after become_normal"
        );
    }

    #[test]
    fn become_normal_strips_lifespan_with_zero_remaining() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                Bolt,
                PhantomBolt,
                PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
                PhantomDamagedCells::default(),
                Lifespan { remaining: 0.0 },
            ))
            .id();
        app.world_mut()
            .entity_mut(e)
            .insert(PhantomDedupKey::Bolt(e));
        app.add_systems(Update, move |mut commands: Commands| {
            PhantomBolt::become_normal(&mut commands, e);
        });
        app.update();
        assert!(
            app.world().get::<Lifespan>(e).is_none(),
            "Lifespan with remaining=0.0 should also be removed"
        );
    }

    // ── Behavior 12: become_normal strips LifetimeEndBehavior ────

    #[test]
    fn become_normal_strips_lifetime_end_behavior_revert() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                Bolt,
                PhantomBolt,
                PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
                PhantomDamagedCells::default(),
                LifetimeEndBehavior::RevertToNormalBolt,
            ))
            .id();
        app.world_mut()
            .entity_mut(e)
            .insert(PhantomDedupKey::Bolt(e));
        app.add_systems(Update, move |mut commands: Commands| {
            PhantomBolt::become_normal(&mut commands, e);
        });
        app.update();
        assert!(
            app.world().get::<LifetimeEndBehavior>(e).is_none(),
            "LifetimeEndBehavior::RevertToNormalBolt should be removed"
        );
    }

    #[test]
    fn become_normal_strips_lifetime_end_behavior_despawn() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                Bolt,
                PhantomBolt,
                PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
                PhantomDamagedCells::default(),
                LifetimeEndBehavior::Despawn,
            ))
            .id();
        app.world_mut()
            .entity_mut(e)
            .insert(PhantomDedupKey::Bolt(e));
        app.add_systems(Update, move |mut commands: Commands| {
            PhantomBolt::become_normal(&mut commands, e);
        });
        app.update();
        assert!(
            app.world().get::<LifetimeEndBehavior>(e).is_none(),
            "LifetimeEndBehavior::Despawn should be removed"
        );
    }

    // ── Behavior 13: become_normal preserves role/velocity/position/cleanup ──

    #[test]
    fn become_normal_preserves_primary_bolt_and_components() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                Bolt,
                PrimaryBolt,
                Velocity2D(Vec2::new(0.0, 400.0)),
                Position2D(Vec2::new(10.0, 20.0)),
                CleanupOnExit::<RunState>::default(),
                BoltBaseDamage(10.0),
                PiercingRemaining(3),
                PhantomBolt,
                PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
                PhantomDamagedCells::default(),
                PhantomFlicker {
                    frequency: 4.0,
                    min_alpha: 0.3,
                },
                Lifespan { remaining: 2.5 },
                LifetimeEndBehavior::RevertToNormalBolt,
            ))
            .id();
        app.world_mut()
            .entity_mut(e)
            .insert(PhantomDedupKey::Bolt(e));
        app.add_systems(Update, move |mut commands: Commands| {
            PhantomBolt::become_normal(&mut commands, e);
        });
        app.update();
        assert!(
            app.world().get::<PhantomBolt>(e).is_none(),
            "become_normal did not execute — PhantomBolt still present"
        );
        assert!(
            app.world().get::<Bolt>(e).is_some(),
            "Bolt should still be present"
        );
        assert!(
            app.world().get::<PrimaryBolt>(e).is_some(),
            "PrimaryBolt should still be present"
        );
        let vel = app
            .world()
            .get::<Velocity2D>(e)
            .expect("Velocity2D should still be present");
        assert_eq!(
            vel.0,
            Vec2::new(0.0, 400.0),
            "Velocity2D should be unchanged"
        );
        let pos = app
            .world()
            .get::<Position2D>(e)
            .expect("Position2D should still be present");
        assert_eq!(
            pos.0,
            Vec2::new(10.0, 20.0),
            "Position2D should be unchanged"
        );
        assert!(
            app.world().get::<CleanupOnExit<RunState>>(e).is_some(),
            "CleanupOnExit<RunState> should still be present"
        );
        let dmg = app
            .world()
            .get::<BoltBaseDamage>(e)
            .expect("BoltBaseDamage should still be present");
        assert!(
            (dmg.0 - 10.0).abs() < f32::EPSILON,
            "BoltBaseDamage should still be 10.0"
        );
        let piercing = app
            .world()
            .get::<PiercingRemaining>(e)
            .expect("PiercingRemaining should still be present");
        assert_eq!(piercing.0, 3, "PiercingRemaining should still be 3");
    }

    #[test]
    fn become_normal_preserves_extra_bolt_and_node_cleanup() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                Bolt,
                ExtraBolt,
                Velocity2D(Vec2::new(0.0, 400.0)),
                Position2D(Vec2::new(10.0, 20.0)),
                CleanupOnExit::<NodeState>::default(),
                BoltBaseDamage(10.0),
                PiercingRemaining(3),
                PhantomBolt,
                PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
                PhantomDamagedCells::default(),
            ))
            .id();
        app.world_mut()
            .entity_mut(e)
            .insert(PhantomDedupKey::Bolt(e));
        app.add_systems(Update, move |mut commands: Commands| {
            PhantomBolt::become_normal(&mut commands, e);
        });
        app.update();
        assert!(
            app.world().get::<PhantomBolt>(e).is_none(),
            "become_normal did not execute — PhantomBolt still present"
        );
        assert!(
            app.world().get::<ExtraBolt>(e).is_some(),
            "ExtraBolt should still be present"
        );
        assert!(
            app.world().get::<CleanupOnExit<NodeState>>(e).is_some(),
            "CleanupOnExit<NodeState> should still be present after become_normal"
        );
    }

    // ── Behavior 14: round-trip become_phantom → become_normal ───

    #[test]
    fn round_trip_become_phantom_then_become_normal_leaves_clean_bolt() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                Bolt,
                PrimaryBolt,
                Velocity2D(Vec2::new(0.0, 400.0)),
                Position2D(Vec2::new(10.0, 20.0)),
            ))
            .id();

        // Phase 0 — become_phantom
        app.world_mut().insert_resource(PhaseGate(0_u32));
        app.add_systems(
            Update,
            move |mut commands: Commands, phase: Res<PhaseGate>| {
                if phase.0 == 0 {
                    Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
                } else if phase.0 == 1 {
                    PhantomBolt::become_normal(&mut commands, e);
                }
            },
        );
        app.update();

        // Verify intermediate phantom state
        assert!(
            app.world().get::<PhantomBolt>(e).is_some(),
            "PhantomBolt should be present after phase 0"
        );
        assert!(
            app.world().get::<PhantomDedupKey>(e).is_some(),
            "PhantomDedupKey should be present after phase 0"
        );
        assert!(
            app.world().get::<PhantomDamagedCells>(e).is_some(),
            "PhantomDamagedCells should be present after phase 0"
        );

        // Advance to phase 1 — become_normal
        app.world_mut().resource_mut::<PhaseGate>().0 = 1;
        app.update();

        // All phantom components gone, normal bolt components intact
        assert!(
            app.world().get::<PhantomBolt>(e).is_none(),
            "PhantomBolt should be gone after phase 1"
        );
        assert!(
            app.world().get::<PhantomDedupKey>(e).is_none(),
            "PhantomDedupKey should be gone after phase 1"
        );
        assert!(
            app.world().get::<PhantomDamagedCells>(e).is_none(),
            "PhantomDamagedCells should be gone after phase 1"
        );
        assert!(
            app.world().get::<Bolt>(e).is_some(),
            "Bolt should still be present after round-trip"
        );
        assert!(
            app.world().get::<PrimaryBolt>(e).is_some(),
            "PrimaryBolt should still be present after round-trip"
        );
        let vel = app
            .world()
            .get::<Velocity2D>(e)
            .expect("Velocity2D should still be present after round-trip");
        assert_eq!(
            vel.0,
            Vec2::new(0.0, 400.0),
            "Velocity2D should be unchanged after round-trip"
        );
        let pos = app
            .world()
            .get::<Position2D>(e)
            .expect("Position2D should still be present after round-trip");
        assert_eq!(
            pos.0,
            Vec2::new(10.0, 20.0),
            "Position2D should be unchanged after round-trip"
        );
    }

    /// Phase gate resource for round-trip test — drives which closure fires.
    #[derive(Resource)]
    struct PhaseGate(u32);

    // ── Behavior 15: old import path re-exports the same type ────

    #[test]
    fn old_import_path_is_same_type_as_new_import_path() {
        use std::any::TypeId;

        use crate::{
            bolt::components::PhantomBolt as NewPath,
            effect_v3::effects::phantom_bolt::components::PhantomBolt as OldPath,
        };

        assert_eq!(
            TypeId::of::<OldPath>(),
            TypeId::of::<NewPath>(),
            "OldPath and NewPath must refer to the same PhantomBolt type (re-export, not parallel definition)"
        );
    }
}
