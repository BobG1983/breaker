use super::helpers::*;

// =========================================================================
// apply_pending_cell_effects
// =========================================================================

/// `apply_pending_cell_effects` applies pending entries to tagged cell
/// entities' `BoundEffects`.
/// Edge case: `Local<bool>` guard prevents re-application on second update.
#[test]
fn pending_cell_effects_applied_to_tagged_cell_entities() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.insert_resource(PendingCellEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 7 })),
    )]));

    app.add_systems(Update, apply_pending_cell_effects);

    let cell = app
        .world_mut()
        .spawn((
            ScenarioTagCell,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();

    app.update();

    let chains = app.world().get::<BoundEffects>(cell).unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "expected cell BoundEffects to have 1 entry from PendingCellEffects, got {}",
        chains.0.len()
    );
    assert_eq!(
        chains.0[0],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 7 }))
        ),
        "expected (\"\", Do(Piercing(7))), got {:?}",
        chains.0[0]
    );

    // PendingCellEffects should be cleared after application
    let pending = app.world().resource::<PendingCellEffects>();
    assert!(
        pending.0.is_empty(),
        "expected PendingCellEffects cleared after application, got {} entries",
        pending.0.len()
    );

    // --- Local<bool> guard: a second update must NOT re-apply new pending effects ---
    app.insert_resource(PendingCellEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 99 })),
    )]));

    app.update();

    let chains = app.world().get::<BoundEffects>(cell).unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "expected Local<bool> guard to prevent re-application; cell should still have 1 entry, got {}",
        chains.0.len()
    );
    assert_eq!(
        chains.0[0],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 7 }))
        ),
        "expected original (\"\", Do(Piercing(7))) preserved, got {:?}",
        chains.0[0]
    );
}

/// When `PendingCellEffects` has entries and two cell entities exist,
/// both cells must receive the effects.
#[test]
fn pending_cell_effects_applied_to_multiple_cell_entities() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.insert_resource(PendingCellEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 8 })),
    )]));

    app.add_systems(Update, apply_pending_cell_effects);

    let cell_a = app
        .world_mut()
        .spawn((
            ScenarioTagCell,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();
    let cell_b = app
        .world_mut()
        .spawn((
            ScenarioTagCell,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();

    app.update();

    let chains_a = app.world().get::<BoundEffects>(cell_a).unwrap();
    assert_eq!(
        chains_a.0.len(),
        1,
        "expected cell A BoundEffects to have 1 entry, got {}",
        chains_a.0.len()
    );
    assert_eq!(
        chains_a.0[0],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 8 }))
        ),
        "expected cell A to have (\"\", Do(Piercing(8))), got {:?}",
        chains_a.0[0]
    );

    let chains_b = app.world().get::<BoundEffects>(cell_b).unwrap();
    assert_eq!(
        chains_b.0.len(),
        1,
        "expected cell B BoundEffects to have 1 entry, got {}",
        chains_b.0.len()
    );
    assert_eq!(
        chains_b.0[0],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 8 }))
        ),
        "expected cell B to have (\"\", Do(Piercing(8))), got {:?}",
        chains_b.0[0]
    );
}

/// When `PendingCellEffects` resource is absent, `apply_pending_cell_effects`
/// must not panic and cell `BoundEffects` must remain empty.
#[test]
fn pending_cell_effects_noop_when_resource_absent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Do NOT insert PendingCellEffects resource
    app.add_systems(Update, apply_pending_cell_effects);

    let cell = app
        .world_mut()
        .spawn((ScenarioTagCell, BoundEffects::default()))
        .id();

    // Must not panic
    app.update();

    let chains = app.world().get::<BoundEffects>(cell).unwrap();
    assert!(
        chains.0.is_empty(),
        "expected cell BoundEffects to remain empty when PendingCellEffects absent, got {} entries",
        chains.0.len()
    );
}

/// `apply_pending_cell_effects` waits for tagged entities, then applies
/// when they appear. Two-phase: first update runs system + queues commands,
/// second update flushes.
#[test]
fn pending_cell_effects_waits_for_tagged_entities_then_applies() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.insert_resource(PendingCellEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 9 })),
    )]));

    app.add_systems(Update, apply_pending_cell_effects);

    // First update: no tagged cell entities exist, so system should not fire
    app.update();

    // PendingCellEffects should NOT be cleared yet
    let pending = app.world().resource::<PendingCellEffects>();
    assert!(
        !pending.0.is_empty(),
        "expected PendingCellEffects NOT cleared when no tagged entities exist, but it was cleared"
    );

    // Now spawn a tagged cell entity WITHOUT BoundEffects
    let cell = app.world_mut().spawn(ScenarioTagCell).id();

    // Two updates: first runs system + queues commands, second flushes
    app.update();
    app.update();

    // Cell should now have BoundEffects with the pending entry
    let chains = app.world().get::<BoundEffects>(cell);
    assert!(
        chains.is_some(),
        "expected BoundEffects to be inserted on cell entity"
    );
    let chains = chains.unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "expected cell BoundEffects to have 1 entry after deferred apply, got {}",
        chains.0.len()
    );
    assert_eq!(
        chains.0[0],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 9 }))
        ),
        "expected (\"\", Do(Piercing(9))), got {:?}",
        chains.0[0]
    );

    // PendingCellEffects should be cleared
    let pending = app.world().resource::<PendingCellEffects>();
    assert!(
        pending.0.is_empty(),
        "expected PendingCellEffects cleared after deferred apply, got {} entries",
        pending.0.len()
    );
}

/// `apply_pending_cell_effects` inserts `BoundEffects` and `StagedEffects`
/// on entities that lack them, using `Commands` + `insert_if_new`.
/// Edge case: entities that already have `BoundEffects` are extended, not overwritten.
#[test]
fn pending_cell_effects_inserts_bound_and_staged_if_absent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.insert_resource(PendingCellEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 10 })),
    )]));

    app.add_systems(Update, apply_pending_cell_effects);

    // Spawn with ONLY ScenarioTagCell — no BoundEffects, no StagedEffects
    let cell = app.world_mut().spawn(ScenarioTagCell).id();

    // Two updates: system queues commands, then commands flush
    app.update();
    app.update();

    // BoundEffects should have been inserted with the pending entry
    let chains = app.world().get::<BoundEffects>(cell);
    assert!(
        chains.is_some(),
        "expected BoundEffects to be inserted on cell entity that lacked it"
    );
    let chains = chains.unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "expected cell BoundEffects to have 1 entry, got {}",
        chains.0.len()
    );
    assert_eq!(
        chains.0[0],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 10 }))
        ),
        "expected (\"\", Do(Piercing(10))), got {:?}",
        chains.0[0]
    );

    // StagedEffects should have been inserted (empty default)
    let staged = app.world().get::<StagedEffects>(cell);
    assert!(
        staged.is_some(),
        "expected StagedEffects to be inserted on cell entity that lacked it"
    );

    // --- Edge case: entity with existing BoundEffects/StagedEffects ---
    // insert_if_new should NOT overwrite, only extend BoundEffects
    let mut app2 = App::new();
    app2.add_plugins(MinimalPlugins);

    app2.insert_resource(PendingCellEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 10 })),
    )]));

    app2.add_systems(Update, apply_pending_cell_effects);

    // Spawn with existing BoundEffects that has a pre-existing entry
    let existing_entries = vec![(
        "existing".to_owned(),
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: ordered_float::OrderedFloat(5.0),
        })),
    )];
    let cell2 = app2
        .world_mut()
        .spawn((
            ScenarioTagCell,
            BoundEffects(existing_entries),
            StagedEffects::default(),
        ))
        .id();

    app2.update();

    let chains2 = app2.world().get::<BoundEffects>(cell2).unwrap();
    // Should have the pre-existing entry PLUS the new pending entry
    assert_eq!(
        chains2.0.len(),
        2,
        "expected cell BoundEffects to have 2 entries (1 existing + 1 pending), got {}",
        chains2.0.len()
    );
    assert_eq!(
        chains2.0[0],
        (
            "existing".to_owned(),
            Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: ordered_float::OrderedFloat(5.0)
            }))
        ),
        "expected first entry to be the pre-existing one, got {:?}",
        chains2.0[0]
    );
    assert_eq!(
        chains2.0[1],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 10 }))
        ),
        "expected second entry to be the pending one, got {:?}",
        chains2.0[1]
    );
}
