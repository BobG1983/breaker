use super::helpers::*;
use crate::{
    bolt::components::SpawnedByEvolution,
    cells::resources::CellConfig,
    chips::definition::Rarity,
    prelude::{SourceId, SourceIdExt},
};

/// Builder-format `SourceId` for the canonical "`ChainLightning`" evolution
/// chip used by these attribution tests.
fn chain_lightning_evolution_source() -> SourceId {
    SourceId::chip("ChainLightning")
        .rarity(Rarity::Evolution)
        .build()
}

/// Builder-format `SourceId` for an alternate evolution chip used by the
/// multi-bolt attribution test.
fn alpha_evolution_source() -> SourceId {
    SourceId::chip("Alpha").rarity(Rarity::Evolution).build()
}

// ── SpawnedByEvolution → DamageDealt<Cell>.source attribution tests ──

#[test]
fn damage_cell_carries_source_chip_from_bolt_spawned_by_evolution() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = super::helpers::test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    let cell_entity = spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(SpawnedByEvolution(chain_lightning_evolution_source()));

    tick(&mut app);

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "should emit exactly one DamageDealt<Cell> message on cell hit"
    );
    assert_eq!(
        msgs.0[0].target, cell_entity,
        "DamageDealt<Cell>.target should match the hit cell entity"
    );
    assert_eq!(
        msgs.0[0].source,
        Some(chain_lightning_evolution_source()),
        "DamageDealt<Cell>.source should carry the bolt's SpawnedByEvolution name"
    );
}

#[test]
fn damage_cell_carries_source_chip_none_when_bolt_has_no_spawned_by_evolution() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = super::helpers::test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "should emit exactly one DamageDealt<Cell> message on cell hit"
    );
    assert_eq!(
        msgs.0[0].source, None,
        "DamageDealt<Cell>.source should be None when bolt has no SpawnedByEvolution"
    );
}

#[test]
fn multiple_bolts_with_different_attributions_produce_correctly_attributed_damage_cells() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = super::helpers::test_bolt_definition();
    let cc = CellConfig::default();

    let cell_a = spawn_cell(&mut app, -200.0, 100.0);
    let cell_b = spawn_cell(&mut app, 200.0, 100.0);

    let start_y = 100.0 - cc.height / 2.0 - bc.radius - 2.0;

    // Bolt A: attributed to "alpha"
    let bolt_a = spawn_bolt(&mut app, -200.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_a)
        .insert(SpawnedByEvolution(alpha_evolution_source()));

    // Bolt B: no attribution
    spawn_bolt(&mut app, 200.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(
        msgs.0.len(),
        2,
        "two bolts hitting two cells should produce two DamageDealt<Cell> messages"
    );

    let msg_a = msgs.0.iter().find(|m| m.target == cell_a);
    let msg_b = msgs.0.iter().find(|m| m.target == cell_b);
    assert!(msg_a.is_some(), "DamageDealt<Cell> for cell A should exist");
    assert!(msg_b.is_some(), "DamageDealt<Cell> for cell B should exist");
    assert_eq!(
        msg_a.unwrap().source,
        Some(alpha_evolution_source()),
        "DamageDealt<Cell> for cell A should have source_chip Some(alpha evolution source) from bolt's SpawnedByEvolution"
    );
    assert_eq!(
        msg_b.unwrap().source,
        None,
        "DamageDealt<Cell> for cell B should have source_chip None (bolt has no SpawnedByEvolution)"
    );
}

// ── B48: SpawnedByEvolution(SourceId) is propagated unchanged to DamageDealt ──

#[test]
fn damage_cell_propagates_builder_constructed_evolution_source_id() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = super::helpers::test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    let evolution_source = SourceId::chip("Overclock")
        .rarity(Rarity::Evolution)
        .build();
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(SpawnedByEvolution(evolution_source.clone()));

    tick(&mut app);

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(msgs.0.len(), 1);
    let observed = msgs.0[0].source.clone();
    assert_eq!(
        observed.as_ref().map(|s| s.0.as_ref()),
        Some("chip:Overclock:Evolution"),
        "DamageDealt<Cell>.source should equal the builder-built evolution source"
    );
    assert_eq!(observed, Some(evolution_source));
}
