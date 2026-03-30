use super::helpers::*;

// =========================================================================
// initial_effects Dispatch
// =========================================================================

/// Breaker-targeted effects are stored in `PendingBreakerEffects` (deferred
/// because no breaker entity exists when `bypass_menu_to_playing` runs).
#[test]
fn initial_effects_breaker_target_pushed_to_effect_chains() {
    let mut definition = make_scenario(100);
    definition.initial_effects = Some(vec![RootEffect::On {
        target: Target::Breaker,
        then: vec![EffectNode::Do(EffectKind::Piercing(1))],
    }]);

    let mut app = bypass_app(definition);
    app.update();

    // Breaker effects are deferred to PendingBreakerEffects
    let pending = app.world().get_resource::<PendingBreakerEffects>();
    assert!(
        pending.is_some(),
        "expected PendingBreakerEffects resource to be inserted for Breaker target"
    );
    let pending = pending.unwrap();
    assert_eq!(
        pending.0.len(),
        1,
        "expected PendingBreakerEffects to have 1 entry from initial_effects, got {}",
        pending.0.len()
    );
    // The On wrapper is unwrapped — only inner `then` children are pushed
    assert_eq!(
        pending.0[0],
        (String::new(), EffectNode::Do(EffectKind::Piercing(1))),
        "expected (\"\", Do(Piercing(1))), got {:?}",
        pending.0[0]
    );
}

/// Bolt-targeted effects stored in `PendingBoltEffects` resource.
#[test]
fn initial_effects_bolt_target_stored_in_pending_bolt_effects() {
    let mut definition = make_scenario(100);
    definition.initial_effects = Some(vec![RootEffect::On {
        target: Target::Bolt,
        then: vec![EffectNode::Do(EffectKind::Piercing(2))],
    }]);

    let mut app = bypass_app(definition);
    app.update();

    let pending = app.world().get_resource::<PendingBoltEffects>();
    assert!(
        pending.is_some(),
        "expected PendingBoltEffects resource to be inserted"
    );
    let pending = pending.unwrap();
    assert_eq!(
        pending.0.len(),
        1,
        "expected 1 pending bolt effect, got {}",
        pending.0.len()
    );
    assert_eq!(
        pending.0[0],
        (String::new(), EffectNode::Do(EffectKind::Piercing(2))),
        "expected (\"\", Do(Piercing(2))), got {:?}",
        pending.0[0]
    );
}

/// When `initial_effects = None`, `BoundEffects` stays empty and
/// no pending resources are inserted.
#[test]
fn initial_effects_none_leaves_effect_chains_empty() {
    let definition = make_scenario(100); // initial_effects is None

    let mut app = bypass_app(definition);

    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();

    app.update();

    let chains = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(
        chains.0.is_empty(),
        "expected BoundEffects empty when initial_effects is None, got {} entries",
        chains.0.len()
    );

    let pending = app.world().get_resource::<PendingBoltEffects>();
    assert!(
        pending.is_none(),
        "expected PendingBoltEffects not inserted when initial_effects is None"
    );

    // Behavior 26: PendingCellEffects and PendingWallEffects must also not be inserted
    let pending_cells = app.world().get_resource::<PendingCellEffects>();
    assert!(
        pending_cells.is_none(),
        "expected PendingCellEffects not inserted when initial_effects is None"
    );

    let pending_walls = app.world().get_resource::<PendingWallEffects>();
    assert!(
        pending_walls.is_none(),
        "expected PendingWallEffects not inserted when initial_effects is None"
    );
}

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
        EffectNode::Do(EffectKind::Piercing(3)),
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
        (String::new(), EffectNode::Do(EffectKind::Piercing(3))),
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
        EffectNode::Do(EffectKind::Piercing(99)),
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
        (String::new(), EffectNode::Do(EffectKind::Piercing(3))),
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
        EffectNode::Do(EffectKind::Piercing(5)),
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
        (String::new(), EffectNode::Do(EffectKind::Piercing(5))),
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
        (String::new(), EffectNode::Do(EffectKind::Piercing(5))),
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
// AllBolts, Cell, AllCells, Wall, AllWalls target routing
// =========================================================================

/// `Target::AllBolts` effects are stored in `PendingBoltEffects`, same as `Target::Bolt`.
/// Edge case: `AllBolts` with empty `then` does not insert `PendingBoltEffects`.
#[test]
fn initial_effects_all_bolts_target_stored_in_pending_bolt_effects() {
    let mut definition = make_scenario(100);
    definition.initial_effects = Some(vec![RootEffect::On {
        target: Target::AllBolts,
        then: vec![EffectNode::Do(EffectKind::Piercing(4))],
    }]);

    let mut app = bypass_app(definition);
    app.update();

    let pending = app.world().get_resource::<PendingBoltEffects>();
    assert!(
        pending.is_some(),
        "expected PendingBoltEffects resource to be inserted for AllBolts target"
    );
    let pending = pending.unwrap();
    assert_eq!(
        pending.0.len(),
        1,
        "expected 1 pending bolt effect for AllBolts, got {}",
        pending.0.len()
    );
    assert_eq!(
        pending.0[0],
        (String::new(), EffectNode::Do(EffectKind::Piercing(4))),
        "expected (\"\", Do(Piercing(4))), got {:?}",
        pending.0[0]
    );

    // Edge case: AllBolts with empty then should not insert PendingBoltEffects
    let mut empty_def = make_scenario(100);
    empty_def.initial_effects = Some(vec![RootEffect::On {
        target: Target::AllBolts,
        then: vec![],
    }]);
    let mut empty_app = bypass_app(empty_def);
    empty_app.update();

    assert!(
        empty_app
            .world()
            .get_resource::<PendingBoltEffects>()
            .is_none(),
        "expected PendingBoltEffects not inserted when AllBolts has empty then"
    );
}

/// `Target::Cell` effects are stored in `PendingCellEffects`.
/// Edge case: breaker `BoundEffects` must remain empty (not misrouted).
#[test]
fn initial_effects_cell_target_stored_in_pending_cell_effects() {
    let mut definition = make_scenario(100);
    definition.initial_effects = Some(vec![RootEffect::On {
        target: Target::Cell,
        then: vec![EffectNode::Do(EffectKind::Piercing(5))],
    }]);

    let mut app = bypass_app(definition);

    // Spawn a breaker to verify no cross-contamination
    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();

    app.update();

    let pending = app.world().get_resource::<PendingCellEffects>();
    assert!(
        pending.is_some(),
        "expected PendingCellEffects resource to be inserted for Cell target"
    );
    let pending = pending.unwrap();
    assert_eq!(
        pending.0.len(),
        1,
        "expected 1 pending cell effect, got {}",
        pending.0.len()
    );
    assert_eq!(
        pending.0[0],
        (String::new(), EffectNode::Do(EffectKind::Piercing(5))),
        "expected (\"\", Do(Piercing(5))), got {:?}",
        pending.0[0]
    );

    // Edge case: breaker BoundEffects must remain empty
    let chains = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(
        chains.0.is_empty(),
        "expected breaker BoundEffects empty when Cell target used, got {} entries (misrouted!)",
        chains.0.len()
    );
}

/// `Target::AllCells` effects are stored in `PendingCellEffects`.
/// Edge case: breaker `BoundEffects` must remain empty.
#[test]
fn initial_effects_all_cells_target_stored_in_pending_cell_effects() {
    let mut definition = make_scenario(100);
    definition.initial_effects = Some(vec![RootEffect::On {
        target: Target::AllCells,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
    }]);

    let mut app = bypass_app(definition);

    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();

    app.update();

    let pending = app.world().get_resource::<PendingCellEffects>();
    assert!(
        pending.is_some(),
        "expected PendingCellEffects resource to be inserted for AllCells target"
    );
    let pending = pending.unwrap();
    assert_eq!(
        pending.0.len(),
        1,
        "expected 1 pending cell effect for AllCells, got {}",
        pending.0.len()
    );
    assert_eq!(
        pending.0[0],
        (String::new(), EffectNode::Do(EffectKind::DamageBoost(1.5))),
        "expected (\"\", Do(DamageBoost(1.5))), got {:?}",
        pending.0[0]
    );

    // Edge case: breaker BoundEffects must remain empty
    let chains = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(
        chains.0.is_empty(),
        "expected breaker BoundEffects empty when AllCells target used, got {} entries (misrouted!)",
        chains.0.len()
    );
}

/// `Target::AllCells` and `Target::Cell` with empty `then` do not insert
/// `PendingCellEffects`.
#[test]
fn initial_effects_cell_targets_empty_then_does_not_insert_pending() {
    // AllCells with empty then
    let mut definition = make_scenario(100);
    definition.initial_effects = Some(vec![RootEffect::On {
        target: Target::AllCells,
        then: vec![],
    }]);
    let mut app = bypass_app(definition);
    app.update();

    assert!(
        app.world().get_resource::<PendingCellEffects>().is_none(),
        "expected PendingCellEffects not inserted when AllCells has empty then"
    );

    // Cell with empty then
    let mut definition2 = make_scenario(100);
    definition2.initial_effects = Some(vec![RootEffect::On {
        target: Target::Cell,
        then: vec![],
    }]);
    let mut app2 = bypass_app(definition2);
    app2.update();

    assert!(
        app2.world().get_resource::<PendingCellEffects>().is_none(),
        "expected PendingCellEffects not inserted when Cell has empty then"
    );
}

/// `Target::Wall` effects are stored in `PendingWallEffects`.
/// Edge case: breaker `BoundEffects` must remain empty.
#[test]
fn initial_effects_wall_target_stored_in_pending_wall_effects() {
    let mut definition = make_scenario(100);
    definition.initial_effects = Some(vec![RootEffect::On {
        target: Target::Wall,
        then: vec![EffectNode::Do(EffectKind::Piercing(6))],
    }]);

    let mut app = bypass_app(definition);

    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();

    app.update();

    let pending = app.world().get_resource::<PendingWallEffects>();
    assert!(
        pending.is_some(),
        "expected PendingWallEffects resource to be inserted for Wall target"
    );
    let pending = pending.unwrap();
    assert_eq!(
        pending.0.len(),
        1,
        "expected 1 pending wall effect, got {}",
        pending.0.len()
    );
    assert_eq!(
        pending.0[0],
        (String::new(), EffectNode::Do(EffectKind::Piercing(6))),
        "expected (\"\", Do(Piercing(6))), got {:?}",
        pending.0[0]
    );

    // Edge case: breaker BoundEffects must remain empty
    let chains = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(
        chains.0.is_empty(),
        "expected breaker BoundEffects empty when Wall target used, got {} entries (misrouted!)",
        chains.0.len()
    );
}

/// `Target::AllWalls` effects are stored in `PendingWallEffects`.
/// Edge case: breaker `BoundEffects` must remain empty.
#[test]
fn initial_effects_all_walls_target_stored_in_pending_wall_effects() {
    let mut definition = make_scenario(100);
    definition.initial_effects = Some(vec![RootEffect::On {
        target: Target::AllWalls,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
    }]);

    let mut app = bypass_app(definition);

    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();

    app.update();

    let pending = app.world().get_resource::<PendingWallEffects>();
    assert!(
        pending.is_some(),
        "expected PendingWallEffects resource to be inserted for AllWalls target"
    );
    let pending = pending.unwrap();
    assert_eq!(
        pending.0.len(),
        1,
        "expected 1 pending wall effect for AllWalls, got {}",
        pending.0.len()
    );
    assert_eq!(
        pending.0[0],
        (String::new(), EffectNode::Do(EffectKind::DamageBoost(2.0))),
        "expected (\"\", Do(DamageBoost(2.0))), got {:?}",
        pending.0[0]
    );

    // Edge case: breaker BoundEffects must remain empty
    let chains = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(
        chains.0.is_empty(),
        "expected breaker BoundEffects empty when AllWalls target used, got {} entries (misrouted!)",
        chains.0.len()
    );
}

/// `Target::AllWalls` and `Target::Wall` with empty `then` do not insert
/// `PendingWallEffects`.
#[test]
fn initial_effects_wall_targets_empty_then_does_not_insert_pending() {
    // AllWalls with empty then
    let mut definition = make_scenario(100);
    definition.initial_effects = Some(vec![RootEffect::On {
        target: Target::AllWalls,
        then: vec![],
    }]);
    let mut app = bypass_app(definition);
    app.update();

    assert!(
        app.world().get_resource::<PendingWallEffects>().is_none(),
        "expected PendingWallEffects not inserted when AllWalls has empty then"
    );

    // Wall with empty then
    let mut definition2 = make_scenario(100);
    definition2.initial_effects = Some(vec![RootEffect::On {
        target: Target::Wall,
        then: vec![],
    }]);
    let mut app2 = bypass_app(definition2);
    app2.update();

    assert!(
        app2.world().get_resource::<PendingWallEffects>().is_none(),
        "expected PendingWallEffects not inserted when Wall has empty then"
    );
}

/// Multiple `RootEffect` entries for cell targets accumulate in `PendingCellEffects`.
/// Edge case: same for wall targets.
#[test]
fn initial_effects_multiple_same_target_accumulate() {
    // Cell + AllCells accumulate into PendingCellEffects
    let mut definition = make_scenario(100);
    definition.initial_effects = Some(vec![
        RootEffect::On {
            target: Target::Cell,
            then: vec![EffectNode::Do(EffectKind::Piercing(10))],
        },
        RootEffect::On {
            target: Target::AllCells,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
        },
    ]);

    let mut app = bypass_app(definition);
    app.update();

    let pending = app.world().get_resource::<PendingCellEffects>();
    assert!(
        pending.is_some(),
        "expected PendingCellEffects resource to be inserted for multiple cell targets"
    );
    let pending = pending.unwrap();
    assert_eq!(
        pending.0.len(),
        2,
        "expected 2 pending cell effects from Cell + AllCells, got {}",
        pending.0.len()
    );
    assert_eq!(
        pending.0[0],
        (String::new(), EffectNode::Do(EffectKind::Piercing(10))),
        "expected first entry (\"\", Do(Piercing(10))), got {:?}",
        pending.0[0]
    );
    assert_eq!(
        pending.0[1],
        (String::new(), EffectNode::Do(EffectKind::DamageBoost(3.0))),
        "expected second entry (\"\", Do(DamageBoost(3.0))), got {:?}",
        pending.0[1]
    );

    // Edge case: Wall + AllWalls accumulate into PendingWallEffects
    let mut wall_def = make_scenario(100);
    wall_def.initial_effects = Some(vec![
        RootEffect::On {
            target: Target::Wall,
            then: vec![EffectNode::Do(EffectKind::Piercing(20))],
        },
        RootEffect::On {
            target: Target::AllWalls,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(4.0))],
        },
    ]);

    let mut wall_app = bypass_app(wall_def);
    wall_app.update();

    let wall_pending = wall_app.world().get_resource::<PendingWallEffects>();
    assert!(
        wall_pending.is_some(),
        "expected PendingWallEffects resource to be inserted for multiple wall targets"
    );
    let wall_pending = wall_pending.unwrap();
    assert_eq!(
        wall_pending.0.len(),
        2,
        "expected 2 pending wall effects from Wall + AllWalls, got {}",
        wall_pending.0.len()
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
        EffectNode::Do(EffectKind::Piercing(15)),
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
        (String::new(), EffectNode::Do(EffectKind::Piercing(15))),
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
        EffectNode::Do(EffectKind::Piercing(16)),
    )]));

    app.add_systems(Update, apply_pending_bolt_effects);

    // Spawn bolt WITH existing BoundEffects containing a pre-existing entry
    let existing_entries = vec![(
        "existing".to_owned(),
        EffectNode::Do(EffectKind::DamageBoost(5.0)),
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
            EffectNode::Do(EffectKind::DamageBoost(5.0))
        ),
        "expected first entry to be the pre-existing one, got {:?}",
        chains.0[0]
    );
    assert_eq!(
        chains.0[1],
        (String::new(), EffectNode::Do(EffectKind::Piercing(16))),
        "expected second entry to be the pending one, got {:?}",
        chains.0[1]
    );
}

/// Mixed targets route to correct pending resources with no cross-contamination.
#[test]
fn initial_effects_mixed_targets_route_correctly() {
    let mut definition = make_scenario(100);
    definition.initial_effects = Some(vec![
        RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::Do(EffectKind::Piercing(1))],
        },
        RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::Piercing(2))],
        },
        RootEffect::On {
            target: Target::AllCells,
            then: vec![EffectNode::Do(EffectKind::Piercing(3))],
        },
        RootEffect::On {
            target: Target::Wall,
            then: vec![EffectNode::Do(EffectKind::Piercing(4))],
        },
    ]);

    let mut app = bypass_app(definition);
    app.update();

    // PendingBreakerEffects gets Target::Breaker effect (deferred)
    let breaker_pending = app.world().get_resource::<PendingBreakerEffects>().unwrap();
    assert_eq!(
        breaker_pending.0.len(),
        1,
        "expected 1 pending breaker effect, got {}",
        breaker_pending.0.len()
    );
    assert_eq!(
        breaker_pending.0[0],
        (String::new(), EffectNode::Do(EffectKind::Piercing(1))),
        "expected breaker pending (\"\", Do(Piercing(1))), got {:?}",
        breaker_pending.0[0]
    );

    // PendingBoltEffects gets Target::Bolt effect
    let bolt_pending = app.world().get_resource::<PendingBoltEffects>().unwrap();
    assert_eq!(
        bolt_pending.0.len(),
        1,
        "expected 1 pending bolt effect, got {}",
        bolt_pending.0.len()
    );
    assert_eq!(
        bolt_pending.0[0],
        (String::new(), EffectNode::Do(EffectKind::Piercing(2))),
        "expected bolt pending (\"\", Do(Piercing(2))), got {:?}",
        bolt_pending.0[0]
    );

    // PendingCellEffects gets Target::AllCells effect
    let cell_pending = app.world().get_resource::<PendingCellEffects>().unwrap();
    assert_eq!(
        cell_pending.0.len(),
        1,
        "expected 1 pending cell effect, got {}",
        cell_pending.0.len()
    );
    assert_eq!(
        cell_pending.0[0],
        (String::new(), EffectNode::Do(EffectKind::Piercing(3))),
        "expected cell pending (\"\", Do(Piercing(3))), got {:?}",
        cell_pending.0[0]
    );

    // PendingWallEffects gets Target::Wall effect
    let wall_pending = app.world().get_resource::<PendingWallEffects>().unwrap();
    assert_eq!(
        wall_pending.0.len(),
        1,
        "expected 1 pending wall effect, got {}",
        wall_pending.0.len()
    );
    assert_eq!(
        wall_pending.0[0],
        (String::new(), EffectNode::Do(EffectKind::Piercing(4))),
        "expected wall pending (\"\", Do(Piercing(4))), got {:?}",
        wall_pending.0[0]
    );
}

// =========================================================================
// Regression: bypass_menu_to_playing stores breaker-targeted effects
// in PendingBreakerEffects instead of querying breaker entities directly
// (no breaker exists at OnEnter(MainMenu))
// =========================================================================

/// `bypass_menu_to_playing` stores `Target::Breaker` effects in
/// `PendingBreakerEffects` resource for deferred application, because
/// no breaker entity exists when this system runs (OnEnter(MainMenu)).
#[test]
fn bypass_stores_breaker_target_in_pending_breaker_effects() {
    let mut definition = make_scenario(100);
    definition.initial_effects = Some(vec![RootEffect::On {
        target: Target::Breaker,
        then: vec![EffectNode::Do(EffectKind::Piercing(20))],
    }]);

    let mut app = bypass_app(definition);

    // Do NOT spawn a breaker entity — this is the point:
    // bypass runs at OnEnter(MainMenu) when no breaker exists yet.
    app.update();

    let pending = app.world().get_resource::<PendingBreakerEffects>();
    assert!(
        pending.is_some(),
        "expected PendingBreakerEffects resource to be inserted for Breaker target"
    );
    let pending = pending.unwrap();
    assert_eq!(
        pending.0.len(),
        1,
        "expected 1 pending breaker effect, got {}",
        pending.0.len()
    );
    assert_eq!(
        pending.0[0],
        (String::new(), EffectNode::Do(EffectKind::Piercing(20))),
        "expected (\"\", Do(Piercing(20))), got {:?}",
        pending.0[0]
    );
}
