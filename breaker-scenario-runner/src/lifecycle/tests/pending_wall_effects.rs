use super::helpers::*;

// =========================================================================
// apply_pending_wall_effects
// =========================================================================

/// `apply_pending_wall_effects` applies pending entries to tagged wall
/// entities' `BoundEffects`.
/// Edge case: `Local<bool>` guard prevents re-application on second update.
#[test]
fn pending_wall_effects_applied_to_tagged_wall_entities() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.insert_resource(PendingWallEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 11 })),
    )]));

    app.add_systems(Update, apply_pending_wall_effects);

    let wall = app
        .world_mut()
        .spawn((
            ScenarioTagWall,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();

    app.update();

    let chains = app.world().get::<BoundEffects>(wall).unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "expected wall BoundEffects to have 1 entry from PendingWallEffects, got {}",
        chains.0.len()
    );
    assert_eq!(
        chains.0[0],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 11 }))
        ),
        "expected (\"\", Do(Piercing(11))), got {:?}",
        chains.0[0]
    );

    // PendingWallEffects should be cleared after application
    let pending = app.world().resource::<PendingWallEffects>();
    assert!(
        pending.0.is_empty(),
        "expected PendingWallEffects cleared after application, got {} entries",
        pending.0.len()
    );

    // --- Local<bool> guard: a second update must NOT re-apply new pending effects ---
    app.insert_resource(PendingWallEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 99 })),
    )]));

    app.update();

    let chains = app.world().get::<BoundEffects>(wall).unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "expected Local<bool> guard to prevent re-application; wall should still have 1 entry, got {}",
        chains.0.len()
    );
    assert_eq!(
        chains.0[0],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 11 }))
        ),
        "expected original (\"\", Do(Piercing(11))) preserved, got {:?}",
        chains.0[0]
    );
}

/// When `PendingWallEffects` has entries and three wall entities exist,
/// all three walls must receive the effects.
#[test]
fn pending_wall_effects_applied_to_multiple_wall_entities() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.insert_resource(PendingWallEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 12 })),
    )]));

    app.add_systems(Update, apply_pending_wall_effects);

    let wall_a = app
        .world_mut()
        .spawn((
            ScenarioTagWall,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();
    let wall_b = app
        .world_mut()
        .spawn((
            ScenarioTagWall,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();
    let wall_c = app
        .world_mut()
        .spawn((
            ScenarioTagWall,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();

    app.update();

    for (name, entity) in [("A", wall_a), ("B", wall_b), ("C", wall_c)] {
        let chains = app.world().get::<BoundEffects>(entity).unwrap();
        assert_eq!(
            chains.0.len(),
            1,
            "expected wall {name} BoundEffects to have 1 entry, got {}",
            chains.0.len()
        );
        assert_eq!(
            chains.0[0],
            (
                String::new(),
                Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 12 }))
            ),
            "expected wall {name} to have (\"\", Do(Piercing(12))), got {:?}",
            chains.0[0]
        );
    }
}

/// When `PendingWallEffects` resource is absent, `apply_pending_wall_effects`
/// must not panic and wall `BoundEffects` must remain empty.
#[test]
fn pending_wall_effects_noop_when_resource_absent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Do NOT insert PendingWallEffects resource
    app.add_systems(Update, apply_pending_wall_effects);

    let wall = app
        .world_mut()
        .spawn((ScenarioTagWall, BoundEffects::default()))
        .id();

    // Must not panic
    app.update();

    let chains = app.world().get::<BoundEffects>(wall).unwrap();
    assert!(
        chains.0.is_empty(),
        "expected wall BoundEffects to remain empty when PendingWallEffects absent, got {} entries",
        chains.0.len()
    );
}

/// `apply_pending_wall_effects` waits for tagged entities, then applies
/// when they appear. Two-phase: first update runs system + queues commands,
/// second update flushes.
#[test]
fn pending_wall_effects_waits_for_tagged_entities_then_applies() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.insert_resource(PendingWallEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 13 })),
    )]));

    app.add_systems(Update, apply_pending_wall_effects);

    // First update: no tagged wall entities exist, so system should not fire
    app.update();

    // PendingWallEffects should NOT be cleared yet
    let pending = app.world().resource::<PendingWallEffects>();
    assert!(
        !pending.0.is_empty(),
        "expected PendingWallEffects NOT cleared when no tagged entities exist, but it was cleared"
    );

    // Now spawn a tagged wall entity WITHOUT BoundEffects
    let wall = app.world_mut().spawn(ScenarioTagWall).id();

    // Two updates: first runs system + queues commands, second flushes
    app.update();
    app.update();

    // Wall should now have BoundEffects with the pending entry
    let chains = app.world().get::<BoundEffects>(wall);
    assert!(
        chains.is_some(),
        "expected BoundEffects to be inserted on wall entity"
    );
    let chains = chains.unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "expected wall BoundEffects to have 1 entry after deferred apply, got {}",
        chains.0.len()
    );
    assert_eq!(
        chains.0[0],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 13 }))
        ),
        "expected (\"\", Do(Piercing(13))), got {:?}",
        chains.0[0]
    );

    // PendingWallEffects should be cleared
    let pending = app.world().resource::<PendingWallEffects>();
    assert!(
        pending.0.is_empty(),
        "expected PendingWallEffects cleared after deferred apply, got {} entries",
        pending.0.len()
    );
}

/// `apply_pending_wall_effects` inserts `BoundEffects` and `StagedEffects`
/// on entities that lack them, using `Commands` + `insert_if_new`.
/// Edge case: entities that already have `BoundEffects` are extended, not overwritten.
#[test]
fn pending_wall_effects_inserts_bound_and_staged_if_absent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.insert_resource(PendingWallEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 14 })),
    )]));

    app.add_systems(Update, apply_pending_wall_effects);

    // Spawn with ONLY ScenarioTagWall — no BoundEffects, no StagedEffects
    let wall = app.world_mut().spawn(ScenarioTagWall).id();

    // Two updates: system queues commands, then commands flush
    app.update();
    app.update();

    // BoundEffects should have been inserted with the pending entry
    let chains = app.world().get::<BoundEffects>(wall);
    assert!(
        chains.is_some(),
        "expected BoundEffects to be inserted on wall entity that lacked it"
    );
    let chains = chains.unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "expected wall BoundEffects to have 1 entry, got {}",
        chains.0.len()
    );
    assert_eq!(
        chains.0[0],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 14 }))
        ),
        "expected (\"\", Do(Piercing(14))), got {:?}",
        chains.0[0]
    );

    // StagedEffects should have been inserted (empty default)
    let staged = app.world().get::<StagedEffects>(wall);
    assert!(
        staged.is_some(),
        "expected StagedEffects to be inserted on wall entity that lacked it"
    );

    // --- Edge case: entity with existing BoundEffects/StagedEffects ---
    // insert_if_new should NOT overwrite, only extend BoundEffects
    let mut app2 = App::new();
    app2.add_plugins(MinimalPlugins);

    app2.insert_resource(PendingWallEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 14 })),
    )]));

    app2.add_systems(Update, apply_pending_wall_effects);

    // Spawn with existing BoundEffects that has a pre-existing entry
    let existing_entries = vec![(
        "existing".to_owned(),
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: ordered_float::OrderedFloat(5.0),
        })),
    )];
    let wall2 = app2
        .world_mut()
        .spawn((
            ScenarioTagWall,
            BoundEffects(existing_entries),
            StagedEffects::default(),
        ))
        .id();

    app2.update();

    let chains2 = app2.world().get::<BoundEffects>(wall2).unwrap();
    // Should have the pre-existing entry PLUS the new pending entry
    assert_eq!(
        chains2.0.len(),
        2,
        "expected wall BoundEffects to have 2 entries (1 existing + 1 pending), got {}",
        chains2.0.len()
    );
    assert_eq!(
        chains2.0[0],
        (
            "existing".to_owned(),
            Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: ordered_float::OrderedFloat(5.0),
            }))
        ),
        "expected first entry to be the pre-existing one, got {:?}",
        chains2.0[0]
    );
    assert_eq!(
        chains2.0[1],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 14 }))
        ),
        "expected second entry to be the pending one, got {:?}",
        chains2.0[1]
    );
}
