use crate::lifecycle::tests::helpers::*;

// =========================================================================
// AllBolts, Cell, AllCells, Wall, AllWalls target routing
// =========================================================================

/// `Target::AllBolts` effects are stored in `PendingBoltEffects`, same as `Target::Bolt`.
/// Edge case: `AllBolts` with empty `then` does not insert `PendingBoltEffects`.
#[test]
fn initial_effects_all_bolts_target_stored_in_pending_bolt_effects() {
    let mut definition = make_scenario(100);
    definition.initial_effects = Some(vec![RootNode::Stamp(
        StampTarget::ActiveBolts,
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 4 })),
    )]);

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
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 4 }))
        ),
        "expected (\"\", Do(Piercing(4))), got {:?}",
        pending.0[0]
    );
}

/// `Target::Cell` effects are stored in `PendingCellEffects`.
/// Edge case: breaker `BoundEffects` must remain empty (not misrouted).
#[test]
fn initial_effects_cell_target_stored_in_pending_cell_effects() {
    let mut definition = make_scenario(100);
    definition.initial_effects = Some(vec![RootNode::Stamp(
        StampTarget::ActiveCells,
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 5 })),
    )]);

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
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 5 }))
        ),
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
    definition.initial_effects = Some(vec![RootNode::Stamp(
        StampTarget::ActiveCells,
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: ordered_float::OrderedFloat(1.5),
        })),
    )]);

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
        (
            String::new(),
            Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: ordered_float::OrderedFloat(1.5),
            }))
        ),
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
/// `Target::Wall` effects are stored in `PendingWallEffects`.
/// Edge case: breaker `BoundEffects` must remain empty.
#[test]
fn initial_effects_wall_target_stored_in_pending_wall_effects() {
    let mut definition = make_scenario(100);
    definition.initial_effects = Some(vec![RootNode::Stamp(
        StampTarget::ActiveWalls,
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 6 })),
    )]);

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
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 6 }))
        ),
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
    definition.initial_effects = Some(vec![RootNode::Stamp(
        StampTarget::ActiveWalls,
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: ordered_float::OrderedFloat(2.0),
        })),
    )]);

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
        (
            String::new(),
            Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: ordered_float::OrderedFloat(2.0),
            }))
        ),
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
/// Multiple `RootNode` entries for cell targets accumulate in `PendingCellEffects`.
/// Edge case: same for wall targets.
#[test]
fn initial_effects_multiple_same_target_accumulate() {
    // Cell + AllCells accumulate into PendingCellEffects
    let mut definition = make_scenario(100);
    definition.initial_effects = Some(vec![
        RootNode::Stamp(
            StampTarget::ActiveCells,
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 10 })),
        ),
        RootNode::Stamp(
            StampTarget::ActiveCells,
            Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: ordered_float::OrderedFloat(3.0),
            })),
        ),
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
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 10 }))
        ),
        "expected first entry (\"\", Do(Piercing(10))), got {:?}",
        pending.0[0]
    );
    assert_eq!(
        pending.0[1],
        (
            String::new(),
            Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: ordered_float::OrderedFloat(3.0),
            }))
        ),
        "expected second entry (\"\", Do(DamageBoost(3.0))), got {:?}",
        pending.0[1]
    );

    // Edge case: Wall + AllWalls accumulate into PendingWallEffects
    let mut wall_def = make_scenario(100);
    wall_def.initial_effects = Some(vec![
        RootNode::Stamp(
            StampTarget::ActiveWalls,
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 20 })),
        ),
        RootNode::Stamp(
            StampTarget::ActiveWalls,
            Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: ordered_float::OrderedFloat(4.0),
            })),
        ),
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
