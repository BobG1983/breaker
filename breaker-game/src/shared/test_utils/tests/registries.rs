use bevy::prelude::*;

use super::{
    super::*,
    helpers::{make_bolt_definition, make_cell_definition},
};
use crate::{
    bolt::registry::BoltRegistry,
    breaker::{definition::BreakerDefinition, registry::BreakerRegistry},
    cells::resources::CellTypeRegistry,
};

// ════════════════════════════════════════════════════════════════════
// Section J: Registry Methods — Bolt
// ════════════════════════════════════════════════════════════════════

// ── Behavior 21: with_bolt_registry() creates empty registry ──

#[test]
fn with_bolt_registry_creates_empty_registry() {
    let app = TestAppBuilder::new().with_bolt_registry().build();
    let registry = app.world().get_resource::<BoltRegistry>();
    assert!(
        registry.is_some(),
        "with_bolt_registry() must register BoltRegistry"
    );
    assert!(
        registry.unwrap().is_empty(),
        "BoltRegistry should start empty"
    );
}

#[test]
fn with_bolt_registry_twice_does_not_panic() {
    let app = TestAppBuilder::new()
        .with_bolt_registry()
        .with_bolt_registry()
        .build();
    assert!(app.world().get_resource::<BoltRegistry>().is_some());
}

// ── Behavior 22: with_bolt_registry_entry() inserts definition ──

#[test]
fn with_bolt_registry_entry_creates_registry_and_inserts() {
    let def = make_bolt_definition("Bolt", 400.0);
    let app = TestAppBuilder::new()
        .with_bolt_registry_entry("Bolt", def)
        .build();
    let registry = app.world().resource::<BoltRegistry>();
    assert!(
        registry.get("Bolt").is_some(),
        "BoltRegistry should contain 'Bolt' after with_bolt_registry_entry"
    );
    assert!(
        (registry.get("Bolt").unwrap().base_speed - 400.0).abs() < f32::EPSILON,
        "Bolt base_speed should be 400.0"
    );
}

#[test]
fn with_bolt_registry_entry_multiple_entries() {
    let def_a = make_bolt_definition("A", 300.0);
    let def_b = make_bolt_definition("B", 500.0);
    let app = TestAppBuilder::new()
        .with_bolt_registry_entry("A", def_a)
        .with_bolt_registry_entry("B", def_b)
        .build();
    let registry = app.world().resource::<BoltRegistry>();
    assert_eq!(
        registry.len(),
        2,
        "Registry should have 2 entries after inserting A and B"
    );
}

// ── Behavior 23: with_bolt_registry_entry() overwrites same name ──

#[test]
fn with_bolt_registry_entry_overwrites_same_name() {
    let def1 = make_bolt_definition("Bolt", 400.0);
    let def2 = make_bolt_definition("Bolt", 600.0);
    let app = TestAppBuilder::new()
        .with_bolt_registry_entry("Bolt", def1)
        .with_bolt_registry_entry("Bolt", def2)
        .build();
    let registry = app.world().resource::<BoltRegistry>();
    assert!(
        (registry.get("Bolt").unwrap().base_speed - 600.0).abs() < f32::EPSILON,
        "Second entry should overwrite first (base_speed should be 600.0)"
    );
    assert_eq!(
        registry.len(),
        1,
        "Registry should have 1 entry (not 2) after overwrite"
    );
}

// ════════════════════════════════════════════════════════════════════
// Section K: Registry Methods — Breaker
// ════════════════════════════════════════════════════════════════════

// ── Behavior 24: with_breaker_registry() creates empty registry ──

#[test]
fn with_breaker_registry_creates_empty_registry() {
    let app = TestAppBuilder::new().with_breaker_registry().build();
    let registry = app.world().get_resource::<BreakerRegistry>();
    assert!(
        registry.is_some(),
        "with_breaker_registry() must register BreakerRegistry"
    );
    assert!(
        registry.unwrap().is_empty(),
        "BreakerRegistry should start empty"
    );
}

// ── Behavior 25: with_breaker_registry_entry() inserts definition ──

#[test]
fn with_breaker_registry_entry_creates_and_inserts() {
    let def = BreakerDefinition {
        name: "Aegis".to_string(),
        life_pool: None,
        effects: vec![],
        ..Default::default()
    };
    let app = TestAppBuilder::new()
        .with_breaker_registry_entry("Aegis", def)
        .build();
    let registry = app.world().resource::<BreakerRegistry>();
    assert!(
        registry.get("Aegis").is_some(),
        "BreakerRegistry should contain 'Aegis' after with_breaker_registry_entry"
    );
}

#[test]
fn with_breaker_registry_entry_without_prior_registry() {
    // Calling entry without with_breaker_registry should auto-create
    let def = BreakerDefinition {
        name: "Vortex".to_string(),
        ..Default::default()
    };
    let app = TestAppBuilder::new()
        .with_breaker_registry_entry("Vortex", def)
        .build();
    assert!(
        app.world()
            .resource::<BreakerRegistry>()
            .get("Vortex")
            .is_some(),
        "with_breaker_registry_entry should auto-create the registry"
    );
}

// ════════════════════════════════════════════════════════════════════
// Section L: Registry Methods — Cell
// ════════════════════════════════════════════════════════════════════

// ── Behavior 26: with_cell_registry() creates empty registry ──

#[test]
fn with_cell_registry_creates_empty_registry() {
    let app = TestAppBuilder::new().with_cell_registry().build();
    let registry = app.world().get_resource::<CellTypeRegistry>();
    assert!(
        registry.is_some(),
        "with_cell_registry() must register CellTypeRegistry"
    );
    assert_eq!(
        registry.unwrap().len(),
        0,
        "CellTypeRegistry should start with 0 entries"
    );
}

// ── Behavior 27: with_cell_registry_entry() inserts definition ──

#[test]
fn with_cell_registry_entry_creates_and_inserts() {
    let def = make_cell_definition("S");
    let app = TestAppBuilder::new()
        .with_cell_registry_entry("S", def)
        .build();
    let registry = app.world().resource::<CellTypeRegistry>();
    assert!(
        registry.get("S").is_some(),
        "CellTypeRegistry should contain 'S' after with_cell_registry_entry"
    );
}

#[test]
fn with_cell_registry_entry_multiple_aliases() {
    let def_s = make_cell_definition("S");
    let def_t = make_cell_definition("T");
    let app = TestAppBuilder::new()
        .with_cell_registry_entry("S", def_s)
        .with_cell_registry_entry("T", def_t)
        .build();
    let registry = app.world().resource::<CellTypeRegistry>();
    assert!(registry.get("S").is_some(), "should contain S");
    assert!(registry.get("T").is_some(), "should contain T");
}
