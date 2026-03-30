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

// -------------------------------------------------------------------------
// tag_game_entities — tags Cell entities with ScenarioTagCell
// -------------------------------------------------------------------------

/// `tag_game_entities` must find all [`Cell`] entities that lack
/// [`ScenarioTagCell`] and insert the marker. After two updates (system
/// runs + commands flush), the entity must have [`ScenarioTagCell`].
/// Edge case: an entity that already has `ScenarioTagCell` is not re-tagged.
#[test]
fn tag_game_entities_tags_cell_entity_with_scenario_tag_cell() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, tag_game_entities);

    let entity = app.world_mut().spawn(Cell).id();

    // First update: system runs and enqueues insert(ScenarioTagCell)
    app.update();
    // Second update: commands are flushed
    app.update();

    assert!(
        app.world()
            .entity(entity)
            .get::<ScenarioTagCell>()
            .is_some(),
        "expected ScenarioTagCell to be added to Cell entity"
    );

    // Edge case: already-tagged entity is not re-tagged.
    // Spawn a new cell and verify only the new one gets tagged.
    let entity2 = app.world_mut().spawn(Cell).id();
    app.update();
    app.update();

    assert!(
        app.world()
            .entity(entity2)
            .get::<ScenarioTagCell>()
            .is_some(),
        "expected second Cell entity to receive ScenarioTagCell"
    );
    // Original entity still has the tag
    assert!(
        app.world()
            .entity(entity)
            .get::<ScenarioTagCell>()
            .is_some(),
        "expected original Cell entity to still have ScenarioTagCell"
    );
}

// -------------------------------------------------------------------------
// tag_game_entities — tags Wall entities with ScenarioTagWall
// -------------------------------------------------------------------------

/// `tag_game_entities` must find all [`Wall`] entities that lack
/// [`ScenarioTagWall`] and insert the marker. After two updates the
/// entity must have [`ScenarioTagWall`].
/// Edge case: an entity that already has `ScenarioTagWall` is not re-tagged.
#[test]
fn tag_game_entities_tags_wall_entity_with_scenario_tag_wall() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, tag_game_entities);

    let entity = app.world_mut().spawn(Wall).id();

    app.update();
    app.update();

    assert!(
        app.world()
            .entity(entity)
            .get::<ScenarioTagWall>()
            .is_some(),
        "expected ScenarioTagWall to be added to Wall entity"
    );

    // Edge case: already-tagged entity is not re-tagged
    let entity2 = app.world_mut().spawn(Wall).id();
    app.update();
    app.update();

    assert!(
        app.world()
            .entity(entity2)
            .get::<ScenarioTagWall>()
            .is_some(),
        "expected second Wall entity to receive ScenarioTagWall"
    );
    assert!(
        app.world()
            .entity(entity)
            .get::<ScenarioTagWall>()
            .is_some(),
        "expected original Wall entity to still have ScenarioTagWall"
    );
}

// -------------------------------------------------------------------------
// tag_game_entities — tags multiple entity types in one pass
// -------------------------------------------------------------------------

/// `tag_game_entities` must tag bolt, breaker, cell, and wall entities
/// all in a single pass.
#[test]
fn tag_game_entities_tags_multiple_entity_types_in_one_pass() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, tag_game_entities);

    let bolt = app
        .world_mut()
        .spawn((Bolt, Position2D(Vec2::new(50.0, 50.0))))
        .id();
    let breaker = app
        .world_mut()
        .spawn((Breaker, Position2D(Vec2::new(0.0, -250.0))))
        .id();
    let cell_a = app.world_mut().spawn(Cell).id();
    let cell_b = app.world_mut().spawn(Cell).id();
    let wall_a = app.world_mut().spawn(Wall).id();
    let wall_b = app.world_mut().spawn(Wall).id();
    let wall_c = app.world_mut().spawn(Wall).id();

    app.update();
    app.update();

    assert!(
        app.world().entity(bolt).get::<ScenarioTagBolt>().is_some(),
        "expected Bolt entity to have ScenarioTagBolt"
    );
    assert!(
        app.world()
            .entity(breaker)
            .get::<ScenarioTagBreaker>()
            .is_some(),
        "expected Breaker entity to have ScenarioTagBreaker"
    );
    assert!(
        app.world()
            .entity(cell_a)
            .get::<ScenarioTagCell>()
            .is_some(),
        "expected Cell A to have ScenarioTagCell"
    );
    assert!(
        app.world()
            .entity(cell_b)
            .get::<ScenarioTagCell>()
            .is_some(),
        "expected Cell B to have ScenarioTagCell"
    );
    assert!(
        app.world()
            .entity(wall_a)
            .get::<ScenarioTagWall>()
            .is_some(),
        "expected Wall A to have ScenarioTagWall"
    );
    assert!(
        app.world()
            .entity(wall_b)
            .get::<ScenarioTagWall>()
            .is_some(),
        "expected Wall B to have ScenarioTagWall"
    );
    assert!(
        app.world()
            .entity(wall_c)
            .get::<ScenarioTagWall>()
            .is_some(),
        "expected Wall C to have ScenarioTagWall"
    );

    // Edge case: a second update pair does NOT modify already-tagged entities
    app.update();
    app.update();

    // Still tagged — no panics, no changes
    assert!(
        app.world().entity(bolt).get::<ScenarioTagBolt>().is_some(),
        "expected Bolt entity to still have ScenarioTagBolt after second pass"
    );
    assert!(
        app.world()
            .entity(cell_a)
            .get::<ScenarioTagCell>()
            .is_some(),
        "expected Cell A to still have ScenarioTagCell after second pass"
    );
}

// -------------------------------------------------------------------------
// tag_game_entities — updates ScenarioStats cell/wall counts
// -------------------------------------------------------------------------

/// `tag_game_entities` must update `ScenarioStats.cells_tagged` and
/// `ScenarioStats.walls_tagged` when tagging cell and wall entities.
#[test]
fn tag_game_entities_updates_scenario_stats_cell_wall_counts() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<ScenarioStats>()
        .add_systems(Update, tag_game_entities);

    // Spawn 2 cells and 1 wall
    app.world_mut().spawn(Cell);
    app.world_mut().spawn(Cell);
    app.world_mut().spawn(Wall);

    app.update();
    app.update();

    let stats = app.world().resource::<ScenarioStats>();
    assert_eq!(
        stats.cells_tagged, 2,
        "expected cells_tagged == 2, got {}",
        stats.cells_tagged
    );
    assert_eq!(
        stats.walls_tagged, 1,
        "expected walls_tagged == 1, got {}",
        stats.walls_tagged
    );
}
