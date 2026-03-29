use bevy::prelude::*;

use super::{super::effect::*, *};
use crate::{
    bolt::messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall},
    effect::core::AttractionType,
};

#[test]
fn manage_attraction_cell_impact_deactivates_cell_entry() {
    let mut app = test_app_with_manage();

    let bolt = app
        .world_mut()
        .spawn(ActiveAttractions(vec![
            AttractionEntry {
                attraction_type: AttractionType::Cell,
                force: 500.0,
                max_force: None,
                active: true,
            },
            AttractionEntry {
                attraction_type: AttractionType::Wall,
                force: 300.0,
                max_force: None,
                active: true,
            },
        ]))
        .id();

    app.insert_resource(TestImpactMessages {
        cell: vec![BoltImpactCell {
            bolt,
            cell: Entity::PLACEHOLDER,
        }],
        ..default()
    });

    tick(&mut app);

    let attractions = app.world().get::<ActiveAttractions>(bolt).unwrap();
    // Cell entry should be deactivated
    let cell_entry = attractions
        .0
        .iter()
        .find(|e| e.attraction_type == AttractionType::Cell)
        .expect("Cell entry should still exist");
    assert!(
        !cell_entry.active,
        "Cell attraction should be deactivated after BoltImpactCell"
    );
    // Wall entry should remain active
    let wall_entry = attractions
        .0
        .iter()
        .find(|e| e.attraction_type == AttractionType::Wall)
        .expect("Wall entry should still exist");
    assert!(
        wall_entry.active,
        "Wall attraction should remain active after BoltImpactCell"
    );
}

#[test]
fn manage_attraction_non_attracted_type_impact_reactivates_all() {
    let mut app = test_app_with_manage();

    let bolt = app
        .world_mut()
        .spawn(ActiveAttractions(vec![AttractionEntry {
            attraction_type: AttractionType::Cell,
            force: 500.0,
            max_force: None,
            active: false,
        }]))
        .id();

    // Wall impact -- bolt has no Wall entry, so this is a non-attracted type
    app.insert_resource(TestImpactMessages {
        wall: vec![BoltImpactWall {
            bolt,
            wall: Entity::PLACEHOLDER,
        }],
        ..default()
    });

    tick(&mut app);

    let attractions = app.world().get::<ActiveAttractions>(bolt).unwrap();
    assert!(
        attractions.0[0].active,
        "Cell entry should be reactivated after non-attracted-type impact (wall)"
    );
}

#[test]
fn manage_attraction_wall_impact_deactivates_wall_entry() {
    let mut app = test_app_with_manage();

    let bolt = app
        .world_mut()
        .spawn(ActiveAttractions(vec![AttractionEntry {
            attraction_type: AttractionType::Wall,
            force: 300.0,
            max_force: None,
            active: true,
        }]))
        .id();

    app.insert_resource(TestImpactMessages {
        wall: vec![BoltImpactWall {
            bolt,
            wall: Entity::PLACEHOLDER,
        }],
        ..default()
    });

    tick(&mut app);

    let attractions = app.world().get::<ActiveAttractions>(bolt).unwrap();
    assert!(
        !attractions.0[0].active,
        "Wall attraction should be deactivated after BoltImpactWall"
    );
}

#[test]
fn manage_attraction_breaker_impact_deactivates_breaker_entry() {
    let mut app = test_app_with_manage();

    let bolt = app
        .world_mut()
        .spawn(ActiveAttractions(vec![AttractionEntry {
            attraction_type: AttractionType::Breaker,
            force: 200.0,
            max_force: None,
            active: true,
        }]))
        .id();

    app.insert_resource(TestImpactMessages {
        breaker: vec![BoltImpactBreaker {
            bolt,
            breaker: Entity::PLACEHOLDER,
        }],
        ..default()
    });

    tick(&mut app);

    let attractions = app.world().get::<ActiveAttractions>(bolt).unwrap();
    assert!(
        !attractions.0[0].active,
        "Breaker attraction should be deactivated after BoltImpactBreaker"
    );
}

#[test]
fn manage_attraction_impact_for_different_bolt_is_ignored() {
    let mut app = test_app_with_manage();

    let bolt_a = app
        .world_mut()
        .spawn(ActiveAttractions(vec![AttractionEntry {
            attraction_type: AttractionType::Cell,
            force: 500.0,
            max_force: None,
            active: true,
        }]))
        .id();

    let bolt_b = app.world_mut().spawn_empty().id();

    // Impact message is for bolt_b, not bolt_a
    app.insert_resource(TestImpactMessages {
        cell: vec![BoltImpactCell {
            bolt: bolt_b,
            cell: Entity::PLACEHOLDER,
        }],
        ..default()
    });

    tick(&mut app);

    let attractions = app.world().get::<ActiveAttractions>(bolt_a).unwrap();
    assert!(
        attractions.0[0].active,
        "bolt_a's attractions should be unchanged when impact was for bolt_b"
    );
}

#[test]
fn manage_attraction_attracted_type_already_inactive_no_reactivation() {
    let mut app = test_app_with_manage();

    let bolt = app
        .world_mut()
        .spawn(ActiveAttractions(vec![
            AttractionEntry {
                attraction_type: AttractionType::Cell,
                force: 500.0,
                max_force: None,
                active: false,
            },
            AttractionEntry {
                attraction_type: AttractionType::Wall,
                force: 300.0,
                max_force: None,
                active: false,
            },
        ]))
        .id();

    // Cell impact -- Cell IS an attracted type, so Wall should NOT be reactivated
    app.insert_resource(TestImpactMessages {
        cell: vec![BoltImpactCell {
            bolt,
            cell: Entity::PLACEHOLDER,
        }],
        ..default()
    });

    tick(&mut app);

    let attractions = app.world().get::<ActiveAttractions>(bolt).unwrap();
    let wall_entry = attractions
        .0
        .iter()
        .find(|e| e.attraction_type == AttractionType::Wall)
        .expect("Wall entry should still exist");
    assert!(
        !wall_entry.active,
        "Wall entry should NOT be reactivated when impact is with an attracted type (Cell)"
    );
}

#[test]
fn manage_attraction_multiple_cell_entries_all_deactivated() {
    let mut app = test_app_with_manage();

    let bolt = app
        .world_mut()
        .spawn(ActiveAttractions(vec![
            AttractionEntry {
                attraction_type: AttractionType::Cell,
                force: 500.0,
                max_force: None,
                active: true,
            },
            AttractionEntry {
                attraction_type: AttractionType::Cell,
                force: 300.0,
                max_force: None,
                active: true,
            },
        ]))
        .id();

    app.insert_resource(TestImpactMessages {
        cell: vec![BoltImpactCell {
            bolt,
            cell: Entity::PLACEHOLDER,
        }],
        ..default()
    });

    tick(&mut app);

    let attractions = app.world().get::<ActiveAttractions>(bolt).unwrap();
    for entry in &attractions.0 {
        assert!(
            !entry.active,
            "all Cell entries should be deactivated, but force={} is still active",
            entry.force
        );
    }
}

#[test]
fn manage_attraction_all_already_active_reactivation_is_noop() {
    let mut app = test_app_with_manage();

    let bolt = app
        .world_mut()
        .spawn(ActiveAttractions(vec![AttractionEntry {
            attraction_type: AttractionType::Cell,
            force: 500.0,
            max_force: None,
            active: true,
        }]))
        .id();

    // Wall impact -- bolt has no Wall entry, so this would trigger reactivation
    // but everything is already active, so it's a no-op
    app.insert_resource(TestImpactMessages {
        wall: vec![BoltImpactWall {
            bolt,
            wall: Entity::PLACEHOLDER,
        }],
        ..default()
    });

    tick(&mut app);

    let attractions = app.world().get::<ActiveAttractions>(bolt).unwrap();
    assert!(
        attractions.0[0].active,
        "already active entry should remain active after reactivation no-op"
    );
}
