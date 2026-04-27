use bevy::prelude::*;

use super::types::*;
use crate::mutators::hazards::definition::{HazardDefinition, HazardKind, HazardTuning};

fn decay_def() -> HazardDefinition {
    HazardDefinition {
        name:        "Decay".into(),
        description: String::new(),
        unlock_tier: 0,
        tuning:      HazardTuning::Decay {
            base_percent:      0.05,
            per_level_percent: 0.03,
        },
    }
}

// ── Behavior 18: HazardRegistry default + insert/get roundtrip ────────

#[test]
fn hazard_registry_default_is_empty() {
    let registry = HazardRegistry::default();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

#[test]
fn hazard_registry_insert_then_get_returns_definition() {
    let mut registry = HazardRegistry::default();
    registry.insert(decay_def());

    let fetched = registry.get(HazardKind::Decay);
    assert!(fetched.is_some(), "expected Some after inserting Decay");
    assert_eq!(fetched.unwrap().name, "Decay");
}

#[test]
fn hazard_registry_get_missing_kind_returns_none() {
    let mut registry = HazardRegistry::default();
    registry.insert(decay_def());
    assert!(
        registry.get(HazardKind::Drift).is_none(),
        "unseen kind should return None"
    );
}

// ── Behavior 19: ActiveHazards::default() is empty ────────────────────

#[test]
fn active_hazards_default_is_empty() {
    let active = ActiveHazards::default();
    assert!(active.is_empty());
    assert_eq!(active.len(), 0);
    assert_eq!(active.stacks(HazardKind::Decay), 0);
    assert!(!active.is_active(HazardKind::Decay));
}

// ── Behavior 20: add_stack starts at 1 and increments ─────────────────

#[test]
fn active_hazards_add_stack_first_call_returns_one() {
    let mut active = ActiveHazards::default();
    let first = active.add_stack(HazardKind::Decay);
    assert_eq!(first, 1, "first add_stack(Decay) should return 1");
    assert_eq!(active.stacks(HazardKind::Decay), 1);
}

#[test]
fn active_hazards_add_stack_three_times_reaches_three() {
    let mut active = ActiveHazards::default();
    active.add_stack(HazardKind::Decay);
    active.add_stack(HazardKind::Decay);
    active.add_stack(HazardKind::Decay);

    assert_eq!(active.stacks(HazardKind::Decay), 3);
    assert!(active.is_active(HazardKind::Decay));
    assert_eq!(active.len(), 1);
}

#[test]
fn active_hazards_add_stack_second_kind_does_not_affect_first() {
    // Edge case: stacking a second kind leaves the first intact.
    let mut active = ActiveHazards::default();
    active.add_stack(HazardKind::Decay);
    active.add_stack(HazardKind::Decay);
    active.add_stack(HazardKind::Decay);
    active.add_stack(HazardKind::Haste);

    assert_eq!(active.stacks(HazardKind::Haste), 1);
    assert_eq!(active.stacks(HazardKind::Decay), 3);
    assert_eq!(active.len(), 2);
}

// ── Behavior 21: stacks() returns 0 for absent kinds without inserting ─

#[test]
fn active_hazards_stacks_absent_kind_returns_zero_without_inserting() {
    let mut active = ActiveHazards::default();
    active.add_stack(HazardKind::Decay);

    assert_eq!(active.stacks(HazardKind::Drift), 0);
    assert_eq!(active.stacks(HazardKind::Decay), 1);
    assert_eq!(
        active.len(),
        1,
        "querying an absent kind must not create a phantom entry"
    );
}

// ── Behavior 22: iter() visits every stored entry ─────────────────────

#[test]
fn active_hazards_iter_visits_every_stored_entry() {
    let mut active = ActiveHazards::default();
    active.add_stack(HazardKind::Decay);
    active.add_stack(HazardKind::Decay);
    active.add_stack(HazardKind::Haste);

    let entries: Vec<(HazardKind, u32)> = active.iter().collect();

    assert_eq!(entries.len(), 2, "iter should visit both Decay and Haste");
    assert!(
        entries.contains(&(HazardKind::Decay, 2)),
        "iter must visit Decay with its 2 stacks; got {entries:?}"
    );
    assert!(
        entries.contains(&(HazardKind::Haste, 1)),
        "iter must visit Haste with its 1 stack; got {entries:?}"
    );
    assert_eq!(active.len(), 2);
    assert!(!active.is_empty());
}

// ── Behavior 23: clear() empties stacks ───────────────────────────────

#[test]
fn active_hazards_clear_empties_stacks() {
    let mut active = ActiveHazards::default();
    active.add_stack(HazardKind::Decay);
    active.add_stack(HazardKind::Decay);
    active.add_stack(HazardKind::Haste);

    active.clear();

    assert!(active.is_empty());
    assert_eq!(active.stacks(HazardKind::Decay), 0);
    assert_eq!(active.stacks(HazardKind::Haste), 0);
}

// ── Behavior 24: HazardOffers default + push + inspect ────────────────

#[test]
fn hazard_offers_default_is_empty() {
    let offers = HazardOffers::default();
    assert!(offers.0.is_empty());
}

#[test]
fn hazard_offers_accept_pushed_definitions() {
    // Proves inner type is Vec<HazardDefinition> (not Vec<HazardKind>).
    let mut offers = HazardOffers::default();
    offers.0.push(decay_def());

    assert_eq!(offers.0.len(), 1);
    assert_eq!(offers.0[0].kind(), HazardKind::Decay);
}

// ── Behavior 25: hazard_active() run condition ────────────────────────

/// Counter resource that systems increment to prove they ran.
#[derive(Resource, Default)]
struct RanCounter(u32);

fn increment_counter(mut counter: ResMut<RanCounter>) {
    counter.0 += 1;
}

#[test]
fn hazard_active_gates_system_on_active_hazards() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<ActiveHazards>()
        .init_resource::<RanCounter>()
        .add_systems(
            Update,
            increment_counter.run_if(hazard_active(HazardKind::Decay)),
        );

    // First update: Decay is not active, system should NOT run.
    app.update();
    assert_eq!(
        app.world().resource::<RanCounter>().0,
        0,
        "hazard_active(Decay) should gate off when ActiveHazards is empty"
    );

    // Add a Decay stack → system should run next frame.
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Decay);

    app.update();
    assert_eq!(
        app.world().resource::<RanCounter>().0,
        1,
        "hazard_active(Decay) should pass once Decay has ≥ 1 stack"
    );
}

#[test]
fn hazard_active_does_not_gate_on_other_kinds() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<ActiveHazards>()
        .init_resource::<RanCounter>()
        .add_systems(
            Update,
            increment_counter.run_if(hazard_active(HazardKind::Drift)),
        );

    // Activate a different kind — Decay — then ensure Drift-gated system stays off.
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Decay);

    app.update();
    assert_eq!(
        app.world().resource::<RanCounter>().0,
        0,
        "hazard_active(Drift) must stay gated when only Decay is active"
    );
}

// ── Behavior 18 extension: HazardRegistry replace-on-same-kind ────────

#[test]
fn hazard_registry_insert_replaces_existing_definition() {
    let mut registry = HazardRegistry::default();
    let mut def_old = decay_def();
    def_old.name = "old".into();
    registry.insert(def_old);

    let mut def_new = decay_def();
    def_new.name = "new".into();
    registry.insert(def_new);

    assert_eq!(registry.len(), 1, "same-kind insert must replace in place");
    assert_eq!(registry.get(HazardKind::Decay).unwrap().name, "new");
}
