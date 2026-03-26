use super::helpers::*;

// =========================================================================
// Godmode Breaker Sentinel
// =========================================================================

/// When `config.breaker = "godmode"`, `bypass_menu_to_playing` inserts a
/// synthetic `BreakerDefinition` into the `BreakerRegistry`.
#[test]
fn godmode_sentinel_inserts_synthetic_breaker_definition() {
    let mut definition = make_scenario(100);
    definition.breaker = "godmode".to_owned();

    let mut app = bypass_app(definition);
    app.update();

    let registry = app.world().resource::<BreakerRegistry>();
    assert!(
        registry.get("Godmode").is_some(),
        "expected BreakerRegistry to contain 'Godmode'"
    );

    let selected = app.world().resource::<breaker::breaker::SelectedBreaker>();
    assert_eq!(
        selected.0, "Godmode",
        "expected SelectedBreaker == 'Godmode', got '{}'",
        selected.0
    );

    let def = registry.get("Godmode").unwrap();
    assert_eq!(def.name, "Godmode");
    assert!(def.life_pool.is_none(), "godmode should have no life_pool");
    assert!(def.effects.is_empty(), "godmode should have empty effects");
}

/// When `config.breaker = "Aegis"`, `BreakerRegistry` does NOT contain "Godmode".
#[test]
fn non_godmode_proceeds_normally() {
    let mut definition = make_scenario(100);
    definition.breaker = "Aegis".to_owned();

    let mut app = bypass_app(definition);
    app.update();

    let registry = app.world().resource::<BreakerRegistry>();
    assert!(
        registry.get("Godmode").is_none(),
        "expected BreakerRegistry to NOT contain 'Godmode'"
    );

    let selected = app.world().resource::<breaker::breaker::SelectedBreaker>();
    assert_eq!(selected.0, "Aegis");
}

// =========================================================================
// Quick-Clear Layout Sentinel
// =========================================================================

/// When `config.layout = "quick_clear"`, `bypass_menu_to_playing` inserts a
/// synthetic `NodeLayout` into the `NodeLayoutRegistry`.
#[test]
fn quick_clear_sentinel_inserts_synthetic_node_layout() {
    let mut definition = make_scenario(100);
    definition.layout = "quick_clear".to_owned();

    let mut app = bypass_app(definition);
    app.update();

    let registry = app.world().resource::<NodeLayoutRegistry>();
    let layout = registry.get_by_name("quick_clear");
    assert!(
        layout.is_some(),
        "expected NodeLayoutRegistry to contain 'quick_clear'"
    );

    let layout = layout.unwrap();
    assert_eq!(layout.name, "quick_clear");
    assert!((layout.timer_secs - 999.0).abs() < f32::EPSILON);
    assert_eq!(layout.cols, 1);
    assert_eq!(layout.rows, 1);
    assert!((layout.grid_top_offset - 50.0).abs() < f32::EPSILON);
    assert_eq!(layout.grid, vec![vec!['S']]);
    assert!((layout.entity_scale - 1.0).abs() < f32::EPSILON);

    let override_res = app
        .world()
        .resource::<breaker::run::node::ScenarioLayoutOverride>();
    assert_eq!(
        override_res.0.as_deref(),
        Some("quick_clear"),
        "expected ScenarioLayoutOverride == Some('quick_clear')"
    );
}

/// When `config.layout = "corridor"`, `NodeLayoutRegistry` does NOT contain `quick_clear`.
#[test]
fn non_quick_clear_proceeds_normally() {
    let mut definition = make_scenario(100);
    definition.layout = "corridor".to_owned();

    let mut app = bypass_app(definition);
    app.update();

    let registry = app.world().resource::<NodeLayoutRegistry>();
    assert!(
        registry.get_by_name("quick_clear").is_none(),
        "expected NodeLayoutRegistry to NOT contain 'quick_clear'"
    );

    let override_res = app
        .world()
        .resource::<breaker::run::node::ScenarioLayoutOverride>();
    assert_eq!(
        override_res.0.as_deref(),
        Some("corridor"),
        "expected ScenarioLayoutOverride == Some('corridor')"
    );
}
