use super::super::helpers::*;

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
