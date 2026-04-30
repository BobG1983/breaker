use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    effect_v3::{components::EffectSourceChip, effects::tether_beam::components::*},
    shared::test_utils::tick,
};

#[test]
fn damage_amount_equals_tether_beam_damage_value() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let _cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
    ));

    tick(&mut app);

    let msgs = damage_msgs(&app);
    assert_eq!(msgs.len(), 1);
    assert!((msgs[0].amount - 12.5).abs() < 1e-6);
}

#[test]
fn zero_damage_still_emits_message() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let _cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(0.0),
        TetherBeamWidth(10.0),
    ));

    tick(&mut app);

    let msgs = damage_msgs(&app);
    assert_eq!(msgs.len(), 1);
    assert!(msgs[0].amount.abs() < 1e-6);
}

#[test]
fn multiple_beams_each_emit_correctly_attributed_damage() {
    let mut app = tether_test_app();

    // Beam 1 — along y=0
    let beam1_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let beam1_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let _cell1 = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));
    let beam1_entity = app
        .world_mut()
        .spawn((
            TetherBeamSource {
                bolt_a: beam1_a,
                bolt_b: beam1_b,
            },
            TetherBeamDamage(10.0),
            TetherBeamWidth(10.0),
            EffectSourceChip(None),
        ))
        .id();

    // Beam 2 — along y=50
    let beam2_a = spawn_endpoint(&mut app, Vec2::new(0.0, 50.0));
    let beam2_b = spawn_endpoint(&mut app, Vec2::new(100.0, 50.0));
    let _cell2 = spawn_alive_cell(&mut app, Vec2::new(50.0, 50.0));
    let beam2_entity = app
        .world_mut()
        .spawn((
            TetherBeamSource {
                bolt_a: beam2_a,
                bolt_b: beam2_b,
            },
            TetherBeamDamage(20.0),
            TetherBeamWidth(10.0),
            EffectSourceChip(None),
        ))
        .id();

    tick(&mut app);

    let msgs = damage_msgs(&app);
    assert_eq!(msgs.len(), 2, "expected exactly 2 messages, one per beam");

    let beam1_msg = msgs
        .iter()
        .find(|m| m.dealer == Some(beam1_entity))
        .expect("beam1 damage message missing");
    assert!((beam1_msg.amount - 10.0).abs() < 1e-6);

    let beam2_msg = msgs
        .iter()
        .find(|m| m.dealer == Some(beam2_entity))
        .expect("beam2 damage message missing");
    assert!((beam2_msg.amount - 20.0).abs() < 1e-6);
}
