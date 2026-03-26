use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::helpers::*;
use crate::bolt::components::{Bolt, SpawnedByEvolution};

// ── SpawnedByEvolution → DamageCell.source_chip attribution tests ──

#[test]
fn damage_cell_carries_source_chip_from_bolt_spawned_by_evolution() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    let cell_entity = spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let _bolt_entity = app
        .world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, start_y)),
            SpawnedByEvolution("chain_lightning".to_owned()),
        ))
        .id();

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "should emit exactly one DamageCell message on cell hit"
    );
    assert_eq!(
        msgs.0[0].cell, cell_entity,
        "DamageCell.cell should match the hit cell entity"
    );
    assert_eq!(
        msgs.0[0].source_chip,
        Some("chain_lightning".to_owned()),
        "DamageCell.source_chip should carry the bolt's SpawnedByEvolution name"
    );
}

#[test]
fn damage_cell_carries_source_chip_none_when_bolt_has_no_spawned_by_evolution() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    app.world_mut().spawn((
        Bolt,
        bolt_param_bundle(),
        Velocity2D(Vec2::new(0.0, 400.0)),
        Position2D(Vec2::new(0.0, start_y)),
    ));

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "should emit exactly one DamageCell message on cell hit"
    );
    assert_eq!(
        msgs.0[0].source_chip, None,
        "DamageCell.source_chip should be None when bolt has no SpawnedByEvolution"
    );
}

#[test]
fn multiple_bolts_with_different_attributions_produce_correctly_attributed_damage_cells() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_a = spawn_cell(&mut app, -200.0, 100.0);
    let cell_b = spawn_cell(&mut app, 200.0, 100.0);

    let start_y = 100.0 - cc.height / 2.0 - bc.radius - 2.0;

    // Bolt A: attributed to "alpha"
    app.world_mut().spawn((
        Bolt,
        bolt_param_bundle(),
        Velocity2D(Vec2::new(0.0, 400.0)),
        Position2D(Vec2::new(-200.0, start_y)),
        SpawnedByEvolution("alpha".to_owned()),
    ));

    // Bolt B: no attribution
    app.world_mut().spawn((
        Bolt,
        bolt_param_bundle(),
        Velocity2D(Vec2::new(0.0, 400.0)),
        Position2D(Vec2::new(200.0, start_y)),
    ));

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(
        msgs.0.len(),
        2,
        "two bolts hitting two cells should produce two DamageCell messages"
    );

    let msg_a = msgs.0.iter().find(|m| m.cell == cell_a);
    let msg_b = msgs.0.iter().find(|m| m.cell == cell_b);
    assert!(msg_a.is_some(), "DamageCell for cell A should exist");
    assert!(msg_b.is_some(), "DamageCell for cell B should exist");
    assert_eq!(
        msg_a.unwrap().source_chip,
        Some("alpha".to_owned()),
        "DamageCell for cell A should have source_chip Some(\"alpha\") from bolt's SpawnedByEvolution"
    );
    assert_eq!(
        msg_b.unwrap().source_chip,
        None,
        "DamageCell for cell B should have source_chip None (bolt has no SpawnedByEvolution)"
    );
}
