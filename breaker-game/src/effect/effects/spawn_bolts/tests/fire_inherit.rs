//! Tests for `fire()` effect inheritance — `BoundEffects` must be copied from
//! the **primary bolt** (entity with `Bolt` and without `ExtraBolt`), not from
//! the fire target entity.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::super::effect::*;
use crate::{
    bolt::{
        components::{Bolt, ExtraBolt},
        resources::BoltConfig,
    },
    effect::{BoundEffects, EffectKind, EffectNode},
    shared::rng::GameRng,
};

fn world_with_bolt_config() -> World {
    let mut world = World::new();
    world.insert_resource(BoltConfig::default());
    world.insert_resource(GameRng::default());
    world
}

/// Behavior 1: inherit=true copies `BoundEffects` from the primary bolt, not
/// from the fire entity (breaker). The breaker has no `BoundEffects`.
#[test]
fn inherit_true_copies_bound_effects_from_primary_bolt_not_fire_entity() {
    let mut world = world_with_bolt_config();

    // Primary bolt — has Bolt (no ExtraBolt) with BoundEffects
    let _primary = world
        .spawn((
            Bolt,
            Position2D(Vec2::new(100.0, 200.0)),
            BoundEffects(vec![(
                "piercing_chip".to_string(),
                EffectNode::Do(EffectKind::Piercing(3)),
            )]),
        ))
        .id();

    // Breaker entity — fire target, NO BoundEffects
    let breaker = world.spawn((Position2D(Vec2::new(50.0, 10.0)),)).id();

    fire(breaker, 1, None, true, "", &mut world);

    let mut query = world.query_filtered::<&BoundEffects, (With<Bolt>, With<ExtraBolt>)>();
    let spawned_effects: Vec<&BoundEffects> = query.iter(&world).collect();
    assert_eq!(spawned_effects.len(), 1, "should spawn 1 extra bolt");

    let effects = spawned_effects[0];
    assert_eq!(
        effects.0.len(),
        1,
        "spawned bolt should have exactly 1 BoundEffects entry from primary bolt"
    );
    assert_eq!(effects.0[0].0, "piercing_chip");
    assert_eq!(effects.0[0].1, EffectNode::Do(EffectKind::Piercing(3)));
}

/// Behavior 1 edge case: fire entity (breaker) has its own `BoundEffects`, but
/// the spawned bolt must still get the primary bolt's effects, not the breaker's.
#[test]
fn inherit_true_uses_primary_bolt_effects_not_breaker_effects() {
    let mut world = world_with_bolt_config();

    // Primary bolt
    let _primary = world
        .spawn((
            Bolt,
            Position2D(Vec2::new(100.0, 200.0)),
            BoundEffects(vec![(
                "piercing_chip".to_string(),
                EffectNode::Do(EffectKind::Piercing(3)),
            )]),
        ))
        .id();

    // Breaker entity with its OWN BoundEffects — these must NOT appear on spawned bolt
    let breaker = world
        .spawn((
            Position2D(Vec2::new(50.0, 10.0)),
            BoundEffects(vec![(
                "breaker_chip".to_string(),
                EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 }),
            )]),
        ))
        .id();

    fire(breaker, 1, None, true, "", &mut world);

    let mut query = world.query_filtered::<&BoundEffects, (With<Bolt>, With<ExtraBolt>)>();
    let spawned_effects: Vec<&BoundEffects> = query.iter(&world).collect();
    assert_eq!(spawned_effects.len(), 1, "should spawn 1 extra bolt");

    let effects = spawned_effects[0];
    assert_eq!(
        effects.0.len(),
        1,
        "spawned bolt should have exactly 1 entry"
    );
    assert_eq!(
        effects.0[0].0, "piercing_chip",
        "spawned bolt should have primary bolt's chip, not breaker's"
    );
    assert_ne!(
        effects.0[0].0, "breaker_chip",
        "spawned bolt must NOT have the breaker's BoundEffects"
    );
}

/// Behavior 2: inherit=true copies from the primary bolt even when the fire
/// entity is an `ExtraBolt` with its own different `BoundEffects`.
#[test]
fn inherit_true_copies_from_primary_bolt_when_fire_entity_is_extra_bolt() {
    let mut world = world_with_bolt_config();

    // Primary bolt — Bolt, no ExtraBolt
    let _primary = world
        .spawn((
            Bolt,
            Position2D(Vec2::ZERO),
            BoundEffects(vec![(
                "damage_chip".to_string(),
                EffectNode::Do(EffectKind::DamageBoost(2.0)),
            )]),
        ))
        .id();

    // Extra bolt — Bolt + ExtraBolt with different effects
    let extra = world
        .spawn((
            Bolt,
            ExtraBolt,
            Position2D(Vec2::new(30.0, 40.0)),
            BoundEffects(vec![(
                "other_chip".to_string(),
                EffectNode::Do(EffectKind::Piercing(1)),
            )]),
        ))
        .id();

    fire(extra, 1, None, true, "", &mut world);

    // There are now 2 ExtraBolt entities: the original extra + the newly spawned one.
    // Find the newly spawned one (it won't be `extra`).
    let mut query =
        world.query_filtered::<(Entity, &BoundEffects), (With<Bolt>, With<ExtraBolt>)>();
    let all: Vec<(Entity, &BoundEffects)> = query.iter(&world).collect();
    assert_eq!(all.len(), 2, "should have original extra + newly spawned");

    let spawned = all.iter().find(|(e, _)| *e != extra).expect("new bolt");
    assert_eq!(
        spawned.1.0.len(),
        1,
        "spawned bolt should have exactly 1 BoundEffects entry"
    );
    assert_eq!(
        spawned.1.0[0].0, "damage_chip",
        "spawned bolt should have primary bolt's 'damage_chip', not extra bolt's 'other_chip'"
    );
    assert_eq!(
        spawned.1.0[0].1,
        EffectNode::Do(EffectKind::DamageBoost(2.0)),
        "spawned bolt should copy primary bolt's DamageBoost(2.0)"
    );
}

/// Behavior 3: inherit=true with no primary bolt in world produces no
/// `BoundEffects` on the spawned bolt (graceful degradation, no panic).
#[test]
fn inherit_true_with_no_primary_bolt_produces_no_bound_effects() {
    let mut world = world_with_bolt_config();

    // Only an ExtraBolt — no entity has Bolt without ExtraBolt
    let extra = world
        .spawn((
            Bolt,
            ExtraBolt,
            Position2D(Vec2::ZERO),
            BoundEffects(vec![(
                "extra_chip".to_string(),
                EffectNode::Do(EffectKind::Piercing(2)),
            )]),
        ))
        .id();

    // Should not panic
    fire(extra, 1, None, true, "", &mut world);

    // Find the newly spawned ExtraBolt (not the original `extra`)
    let mut query =
        world.query_filtered::<(Entity, Option<&BoundEffects>), (With<Bolt>, With<ExtraBolt>)>();
    let all: Vec<(Entity, Option<&BoundEffects>)> = query.iter(&world).collect();
    assert_eq!(all.len(), 2, "should have original extra + newly spawned");

    let spawned = all.iter().find(|(e, _)| *e != extra).expect("new bolt");
    assert!(
        spawned.1.is_none(),
        "spawned bolt should NOT have BoundEffects when no primary bolt exists"
    );
}

/// Behavior 4: inherit=true with primary bolt that has no `BoundEffects`
/// component does not panic, and spawned bolt has no `BoundEffects` — even when
/// the fire entity (breaker) has its own `BoundEffects`.
#[test]
fn inherit_true_with_primary_bolt_without_bound_effects_does_not_panic() {
    let mut world = world_with_bolt_config();

    // Primary bolt — has Bolt but NO BoundEffects
    let _primary = world.spawn((Bolt, Position2D(Vec2::ZERO))).id();

    // Fire entity is a breaker-like WITH BoundEffects (must not be copied)
    let breaker = world
        .spawn((
            Position2D(Vec2::new(50.0, 10.0)),
            BoundEffects(vec![(
                "breaker_only".to_string(),
                EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 }),
            )]),
        ))
        .id();

    // Should not panic
    fire(breaker, 1, None, true, "", &mut world);

    let mut query = world.query_filtered::<Option<&BoundEffects>, (With<Bolt>, With<ExtraBolt>)>();
    let results: Vec<Option<&BoundEffects>> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "should spawn 1 bolt");

    assert!(
        results[0].is_none(),
        "spawned bolt should NOT have BoundEffects when primary bolt lacks them"
    );
}

/// Behavior 5: inherit=true with multiple primary bolts (degenerate case)
/// does not panic — spawns bolt with `BoundEffects` from one of them.
#[test]
fn inherit_true_with_multiple_primary_bolts_does_not_panic() {
    let mut world = world_with_bolt_config();

    // Two primary bolts (both Bolt, no ExtraBolt) with different effects
    let _primary_a = world
        .spawn((
            Bolt,
            Position2D(Vec2::ZERO),
            BoundEffects(vec![(
                "chip_a".to_string(),
                EffectNode::Do(EffectKind::DamageBoost(1.0)),
            )]),
        ))
        .id();
    let _primary_b = world
        .spawn((
            Bolt,
            Position2D(Vec2::new(10.0, 10.0)),
            BoundEffects(vec![(
                "chip_b".to_string(),
                EffectNode::Do(EffectKind::Piercing(1)),
            )]),
        ))
        .id();

    // Fire entity is a non-bolt
    let fire_entity = world.spawn((Position2D(Vec2::ZERO),)).id();

    // Must not panic
    fire(fire_entity, 1, None, true, "", &mut world);

    let mut query = world.query_filtered::<&BoundEffects, (With<Bolt>, With<ExtraBolt>)>();
    let spawned: Vec<&BoundEffects> = query.iter(&world).collect();
    assert_eq!(spawned.len(), 1, "should spawn 1 bolt");

    let effects = spawned[0];
    assert_eq!(
        effects.0.len(),
        1,
        "spawned bolt should have exactly 1 BoundEffects entry from one of the primary bolts"
    );
    // Either chip_a or chip_b is acceptable
    let chip_name = &effects.0[0].0;
    assert!(
        chip_name == "chip_a" || chip_name == "chip_b",
        "spawned bolt should get effects from one of the primary bolts, got '{chip_name}'"
    );
}

/// Behavior 7: inherit=true spawns multiple bolts and each one gets the same
/// `BoundEffects` from the primary bolt.
#[test]
fn inherit_true_spawns_multiple_bolts_each_with_primary_bound_effects() {
    let mut world = world_with_bolt_config();

    // Primary bolt
    let _primary = world
        .spawn((
            Bolt,
            Position2D(Vec2::ZERO),
            BoundEffects(vec![(
                "multi_chip".to_string(),
                EffectNode::Do(EffectKind::DamageBoost(3.0)),
            )]),
        ))
        .id();

    // Fire entity is a breaker
    let breaker = world.spawn((Position2D(Vec2::new(50.0, 10.0)),)).id();

    fire(breaker, 3, None, true, "", &mut world);

    let mut query = world.query_filtered::<&BoundEffects, (With<Bolt>, With<ExtraBolt>)>();
    let spawned: Vec<&BoundEffects> = query.iter(&world).collect();
    assert_eq!(spawned.len(), 3, "should spawn 3 extra bolts");

    for (i, effects) in spawned.iter().enumerate() {
        assert_eq!(
            effects.0.len(),
            1,
            "bolt {i} should have exactly 1 BoundEffects entry"
        );
        assert_eq!(
            effects.0[0].0, "multi_chip",
            "bolt {i} should have 'multi_chip' from primary bolt"
        );
        assert_eq!(
            effects.0[0].1,
            EffectNode::Do(EffectKind::DamageBoost(3.0)),
            "bolt {i} should have DamageBoost(3.0) from primary bolt"
        );
    }
}
