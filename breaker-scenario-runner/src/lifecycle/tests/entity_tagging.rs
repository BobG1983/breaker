use super::helpers::*;

// -------------------------------------------------------------------------
// tag_game_entities — tags Bolt entities with ScenarioTagBolt
// -------------------------------------------------------------------------

/// `tag_game_entities` must find all [`Bolt`] entities that lack
/// [`ScenarioTagBolt`] and insert the marker. After two updates (system
/// runs + commands flush), the entity must have [`ScenarioTagBolt`] and its
/// position must be unchanged.
#[test]
fn tag_game_entities_tags_bolt_entity_with_scenario_tag_bolt() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, tag_game_entities);

    let entity = app
        .world_mut()
        .spawn((Bolt, Position2D(Vec2::new(50.0, 50.0))))
        .id();

    // First update: system runs and enqueues insert(ScenarioTagBolt)
    app.update();
    // Second update: commands are flushed
    app.update();

    assert!(
        app.world()
            .entity(entity)
            .get::<ScenarioTagBolt>()
            .is_some(),
        "expected ScenarioTagBolt to be added to Bolt entity"
    );

    // Position must be unchanged — tagging should not move the entity.
    let position = app
        .world()
        .entity(entity)
        .get::<Position2D>()
        .expect("entity must still have Position2D");
    assert_eq!(
        position.0,
        Vec2::new(50.0, 50.0),
        "expected position unchanged after tagging, got {:?}",
        position.0
    );
}

// -------------------------------------------------------------------------
// tag_game_entities — tags Breaker entities with ScenarioTagBreaker
// -------------------------------------------------------------------------

/// `tag_game_entities` must find all [`Breaker`] entities that lack
/// [`ScenarioTagBreaker`] and insert the marker. After two updates the
/// entity must have [`ScenarioTagBreaker`].
#[test]
fn tag_game_entities_tags_breaker_entity_with_scenario_tag_breaker() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, tag_game_entities);

    let entity = app
        .world_mut()
        .spawn((Breaker, Position2D(Vec2::new(0.0, -250.0))))
        .id();

    app.update();
    app.update();

    assert!(
        app.world()
            .entity(entity)
            .get::<ScenarioTagBreaker>()
            .is_some(),
        "expected ScenarioTagBreaker to be added to Breaker entity"
    );
}
