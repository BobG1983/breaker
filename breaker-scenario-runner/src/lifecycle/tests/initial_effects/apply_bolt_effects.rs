use crate::lifecycle::tests::helpers::*;

// =========================================================================
// apply_pending_bolt_effects
// =========================================================================

/// `apply_pending_bolt_effects` applies pending entries to bolt `BoundEffects`.
#[test]
fn pending_bolt_effects_applied_to_bolt_entities() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Insert pending effects
    app.insert_resource(PendingBoltEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 3 })),
    )]));

    app.add_systems(Update, apply_pending_bolt_effects);

    // Spawn bolt with ScenarioTagBolt + BoundEffects
    let bolt = app
        .world_mut()
        .spawn((ScenarioTagBolt, BoundEffects::default()))
        .id();

    app.update();

    let chains = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "expected bolt BoundEffects to have 1 entry from PendingBoltEffects, got {}",
        chains.0.len()
    );
    assert_eq!(
        chains.0[0],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 3 }))
        ),
        "expected (\"\", Do(Piercing(3))), got {:?}",
        chains.0[0]
    );

    // PendingBoltEffects should be cleared after application
    let pending = app.world().resource::<PendingBoltEffects>();
    assert!(
        pending.0.is_empty(),
        "expected PendingBoltEffects cleared after application, got {} entries",
        pending.0.len()
    );

    // --- Local<bool> guard: a second update must NOT re-apply new pending effects ---
    app.insert_resource(PendingBoltEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 99 })),
    )]));

    app.update();

    // The bolt should still have only the original entry from the first application.
    let chains = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "expected Local<bool> guard to prevent re-application; bolt should still have 1 entry, got {}",
        chains.0.len()
    );
    assert_eq!(
        chains.0[0],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 3 }))
        ),
        "expected original (\"\", Do(Piercing(3))) preserved, got {:?}",
        chains.0[0]
    );
}

/// When `PendingBoltEffects` has entries and two bolt entities exist,
/// both bolts must receive the effects.
#[test]
fn pending_bolt_effects_applied_to_multiple_bolts() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Insert pending effects with one entry
    app.insert_resource(PendingBoltEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 5 })),
    )]));

    app.add_systems(Update, apply_pending_bolt_effects);

    // Spawn two bolt entities with ScenarioTagBolt + BoundEffects
    let bolt_a = app
        .world_mut()
        .spawn((ScenarioTagBolt, BoundEffects::default()))
        .id();
    let bolt_b = app
        .world_mut()
        .spawn((ScenarioTagBolt, BoundEffects::default()))
        .id();

    app.update();

    let chains_a = app.world().get::<BoundEffects>(bolt_a).unwrap();
    assert_eq!(
        chains_a.0.len(),
        1,
        "expected bolt A BoundEffects to have 1 entry, got {}",
        chains_a.0.len()
    );
    assert_eq!(
        chains_a.0[0],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 5 }))
        ),
        "expected bolt A to have (\"\", Do(Piercing(5))), got {:?}",
        chains_a.0[0]
    );

    let chains_b = app.world().get::<BoundEffects>(bolt_b).unwrap();
    assert_eq!(
        chains_b.0.len(),
        1,
        "expected bolt B BoundEffects to have 1 entry, got {}",
        chains_b.0.len()
    );
    assert_eq!(
        chains_b.0[0],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 5 }))
        ),
        "expected bolt B to have (\"\", Do(Piercing(5))), got {:?}",
        chains_b.0[0]
    );
}

/// When `PendingBoltEffects` resource is absent, `apply_pending_bolt_effects`
/// must not panic and bolt `BoundEffects` must remain empty.
#[test]
fn pending_bolt_effects_noop_when_resource_absent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Do NOT insert PendingBoltEffects resource
    app.add_systems(Update, apply_pending_bolt_effects);

    let bolt = app
        .world_mut()
        .spawn((ScenarioTagBolt, BoundEffects::default()))
        .id();

    // Must not panic
    app.update();

    let chains = app.world().get::<BoundEffects>(bolt).unwrap();
    assert!(
        chains.0.is_empty(),
        "expected bolt BoundEffects to remain empty when PendingBoltEffects absent, got {} entries",
        chains.0.len()
    );
}

// =========================================================================
// Regression: apply_pending_bolt_effects must insert BoundEffects on bolts
// that lack it (matching apply_pending_cell_effects / wall_effects pattern)
// =========================================================================

/// `apply_pending_bolt_effects` inserts `BoundEffects` on bolt entities that
/// lack it, using `Commands` + `insert_if_new` (matching cell/wall pattern).
/// Regression: the broken version queries `&mut BoundEffects, With<ScenarioTagBolt>`
/// which silently skips bolts without `BoundEffects`.
#[test]
fn apply_pending_bolt_effects_inserts_bound_effects_on_bolt_without_it() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.insert_resource(PendingBoltEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 15 })),
    )]));

    app.add_systems(Update, apply_pending_bolt_effects);

    // Spawn bolt with ONLY ScenarioTagBolt — NO BoundEffects
    let bolt = app.world_mut().spawn(ScenarioTagBolt).id();

    // Two updates: first runs system + queues commands, second flushes
    app.update();
    app.update();

    // BoundEffects should have been inserted with the pending entry
    let chains = app.world().get::<BoundEffects>(bolt);
    assert!(
        chains.is_some(),
        "expected BoundEffects to be inserted on bolt entity that lacked it"
    );
    let chains = chains.unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "expected bolt BoundEffects to have 1 entry, got {}",
        chains.0.len()
    );
    assert_eq!(
        chains.0[0],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 15 }))
        ),
        "expected (\"\", Do(Piercing(15))), got {:?}",
        chains.0[0]
    );

    // PendingBoltEffects should be cleared
    let pending = app.world().resource::<PendingBoltEffects>();
    assert!(
        pending.0.is_empty(),
        "expected PendingBoltEffects cleared after deferred apply, got {} entries",
        pending.0.len()
    );
}

/// When a bolt entity already has `BoundEffects`, `apply_pending_bolt_effects`
/// extends the existing entries rather than overwriting.
#[test]
fn apply_pending_bolt_effects_extends_existing_bound_effects() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.insert_resource(PendingBoltEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 16 })),
    )]));

    app.add_systems(Update, apply_pending_bolt_effects);

    // Spawn bolt WITH existing BoundEffects containing a pre-existing entry
    let existing_entries = vec![(
        "existing".to_owned(),
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: ordered_float::OrderedFloat(5.0),
        })),
    )];
    let bolt = app
        .world_mut()
        .spawn((
            ScenarioTagBolt,
            BoundEffects(existing_entries),
            StagedEffects::default(),
        ))
        .id();

    // Two updates to ensure deferred commands flush
    app.update();
    app.update();

    let chains = app.world().get::<BoundEffects>(bolt).unwrap();
    // Should have the pre-existing entry PLUS the new pending entry
    assert_eq!(
        chains.0.len(),
        2,
        "expected bolt BoundEffects to have 2 entries (1 existing + 1 pending), got {}",
        chains.0.len()
    );
    assert_eq!(
        chains.0[0],
        (
            "existing".to_owned(),
            Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: ordered_float::OrderedFloat(5.0)
            }))
        ),
        "expected first entry to be the pre-existing one, got {:?}",
        chains.0[0]
    );
    assert_eq!(
        chains.0[1],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 16 }))
        ),
        "expected second entry to be the pending one, got {:?}",
        chains.0[1]
    );
}
