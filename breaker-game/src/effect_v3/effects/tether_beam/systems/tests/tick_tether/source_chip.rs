use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    chips::definition::Rarity,
    effect_v3::{components::EffectSourceChip, effects::tether_beam::components::*},
    prelude::{SourceId, SourceIdExt},
    shared::test_utils::tick,
};

/// Builder-format `SourceId` for the canonical "`StormCoil`" chip used by
/// these tether-beam tick tests.
fn storm_coil_source() -> SourceId {
    SourceId::chip("StormCoil").rarity(Rarity::Common).build()
}

// ── Group B — tick_tether_beam source_chip propagation ─────────────────

#[test]
fn tether_beam_propagates_source_chip_some_in_damage_dealt() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let _cell_mid = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));
    let _cell_far = spawn_alive_cell(&mut app, Vec2::new(75.0, 0.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
        EffectSourceChip(Some(storm_coil_source())),
    ));

    tick(&mut app);

    let msgs = damage_msgs(&app);
    assert_eq!(msgs.len(), 2, "expected 2 DamageDealt<Cell> messages");
    for msg in &msgs {
        assert_eq!(
            msg.source,
            Some(storm_coil_source()),
            "all messages must carry Some(storm coil source) source_chip, got {:?}",
            msg.source,
        );
    }
}

#[test]
fn tether_beam_propagates_source_chip_none_in_damage_dealt() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let _cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
        EffectSourceChip(None),
    ));

    tick(&mut app);

    let msgs = damage_msgs(&app);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].source, None);
}

#[test]
fn tether_beam_missing_source_chip_component_propagates_none() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let _cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));

    // Beam entity is spawned WITHOUT EffectSourceChip at all.
    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
    ));

    tick(&mut app);

    let msgs = damage_msgs(&app);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].source, None);
}
