use super::helpers::*;

// =========================================================================
// apply_pending_breaker_effects
// =========================================================================

/// `apply_pending_breaker_effects` applies pending entries to tagged breaker
/// entities' `BoundEffects`.
/// Regression: breaker effects must be deferred like cell/wall effects because
/// no breaker entity exists when `bypass_menu_to_playing` runs.
#[test]
fn pending_breaker_effects_applied_to_tagged_breaker_entities() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.insert_resource(PendingBreakerEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 21 })),
    )]));

    app.add_systems(Update, apply_pending_breaker_effects);

    // Spawn breaker entity WITHOUT BoundEffects (matching real spawn conditions)
    let breaker = app.world_mut().spawn(ScenarioTagBreaker).id();

    // Two updates: first runs system + queues commands, second flushes
    app.update();
    app.update();

    // BoundEffects should have been inserted with the pending entry
    let chains = app.world().get::<BoundEffects>(breaker);
    assert!(
        chains.is_some(),
        "expected BoundEffects to be inserted on breaker entity that lacked it"
    );
    let chains = chains.unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "expected breaker BoundEffects to have 1 entry, got {}",
        chains.0.len()
    );
    assert_eq!(
        chains.0[0],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 21 }))
        ),
        "expected (\"\", Do(Piercing(21))), got {:?}",
        chains.0[0]
    );

    // PendingBreakerEffects should be cleared after application
    let pending = app.world().resource::<PendingBreakerEffects>();
    assert!(
        pending.0.is_empty(),
        "expected PendingBreakerEffects cleared after application, got {} entries",
        pending.0.len()
    );
}

/// `apply_pending_breaker_effects` extends existing `BoundEffects` on breaker
/// entities rather than overwriting.
#[test]
fn pending_breaker_effects_extends_existing_bound_effects() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.insert_resource(PendingBreakerEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 22 })),
    )]));

    app.add_systems(Update, apply_pending_breaker_effects);

    // Spawn breaker WITH existing BoundEffects
    let existing_entries = vec![(
        "existing".to_owned(),
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: ordered_float::OrderedFloat(5.0),
        })),
    )];
    let breaker = app
        .world_mut()
        .spawn((
            ScenarioTagBreaker,
            BoundEffects(existing_entries),
            StagedEffects::default(),
        ))
        .id();

    // Two updates to ensure deferred commands flush
    app.update();
    app.update();

    let chains = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        chains.0.len(),
        2,
        "expected breaker BoundEffects to have 2 entries (1 existing + 1 pending), got {}",
        chains.0.len()
    );
    assert_eq!(
        chains.0[0],
        (
            "existing".to_owned(),
            Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: ordered_float::OrderedFloat(5.0),
            }))
        ),
        "expected first entry to be the pre-existing one, got {:?}",
        chains.0[0]
    );
    assert_eq!(
        chains.0[1],
        (
            String::new(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 22 }))
        ),
        "expected second entry to be the pending one, got {:?}",
        chains.0[1]
    );
}

/// When `PendingBreakerEffects` resource is absent, `apply_pending_breaker_effects`
/// must not panic and breaker `BoundEffects` must remain empty.
#[test]
fn pending_breaker_effects_noop_when_resource_absent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Do NOT insert PendingBreakerEffects resource
    app.add_systems(Update, apply_pending_breaker_effects);

    let breaker = app
        .world_mut()
        .spawn((ScenarioTagBreaker, BoundEffects::default()))
        .id();

    // Must not panic
    app.update();

    let chains = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(
        chains.0.is_empty(),
        "expected breaker BoundEffects to remain empty when PendingBreakerEffects absent, got {} entries",
        chains.0.len()
    );
}

/// `Local<bool>` guard prevents re-application on second update.
#[test]
fn pending_breaker_effects_local_guard_prevents_reapplication() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.insert_resource(PendingBreakerEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 23 })),
    )]));

    app.add_systems(Update, apply_pending_breaker_effects);

    let breaker = app.world_mut().spawn(ScenarioTagBreaker).id();

    // First two updates: apply + flush
    app.update();
    app.update();

    let chains = app.world().get::<BoundEffects>(breaker);
    assert!(
        chains.is_some(),
        "expected BoundEffects to be inserted after first apply"
    );
    assert_eq!(
        chains.unwrap().0.len(),
        1,
        "expected 1 entry after first apply"
    );

    // Insert new pending effects and update again
    app.insert_resource(PendingBreakerEffects(vec![(
        String::new(),
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 99 })),
    )]));

    app.update();
    app.update();

    // Should still have only 1 entry (guard prevents re-application)
    let chains = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "expected Local<bool> guard to prevent re-application; breaker should still have 1 entry, got {}",
        chains.0.len()
    );
}
