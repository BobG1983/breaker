use super::helpers::*;

// =========================================================================
// initial_effects Dispatch
// =========================================================================

/// Breaker-targeted effects are pushed to breaker `BoundEffects`.
#[test]
fn initial_effects_breaker_target_pushed_to_effect_chains() {
    let mut definition = make_scenario(100);
    definition.initial_effects = Some(vec![RootEffect::On {
        target: Target::Breaker,
        then: vec![EffectNode::Do(EffectKind::Piercing(1))],
    }]);

    let mut app = bypass_app(definition);

    // Spawn a breaker entity with BoundEffects
    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();

    app.update();

    let chains = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "expected breaker BoundEffects to have 1 entry from initial_effects, got {}",
        chains.0.len()
    );
    // The On wrapper is unwrapped — only inner `then` children are pushed
    assert_eq!(
        chains.0[0],
        (String::new(), EffectNode::Do(EffectKind::Piercing(1))),
        "expected (\"\", Do(Piercing(1))), got {:?}",
        chains.0[0]
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
/// `PendingBoltEffects` is not inserted.
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
