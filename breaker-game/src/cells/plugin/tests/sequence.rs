use bevy::prelude::*;

use super::helpers::{
    PluginTestPendingCellDamage, plugin_damage_msg, sequence_plugin_advance_to_playing,
    sequence_plugin_app_loading, spawn_plugin_sequence_cell,
};
use crate::{cells::components::SequenceActive, prelude::*};

/// Behavior 30: `CellsPlugin` registers `init_sequence_groups` in
/// `OnEnter(NodeState::Playing)`.
#[test]
fn cells_plugin_registers_init_sequence_groups_on_enter_playing() {
    let mut app = sequence_plugin_app_loading();

    let e0 = spawn_plugin_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_plugin_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    let e2 = spawn_plugin_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 2, 20.0);

    sequence_plugin_advance_to_playing(&mut app);

    assert!(
        app.world().get::<SequenceActive>(e0).is_some(),
        "CellsPlugin should register init_sequence_groups on OnEnter(NodeState::Playing)"
    );
    assert!(app.world().get::<SequenceActive>(e1).is_none());
    assert!(app.world().get::<SequenceActive>(e2).is_none());
}

/// Behavior 31: `CellsPlugin` registers `reset_inactive_sequence_hp`
/// between `ApplyDamage` and `DetectDeaths`.
#[test]
fn cells_plugin_registers_reset_inactive_sequence_hp_between_apply_and_detect() {
    let mut app = sequence_plugin_app_loading();

    let _e0 = spawn_plugin_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_plugin_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);

    sequence_plugin_advance_to_playing(&mut app);

    app.world_mut()
        .resource_mut::<PluginTestPendingCellDamage>()
        .0
        .push(plugin_damage_msg(e1, 25.0));
    tick(&mut app);

    let hp = app.world().get::<Hp>(e1).expect("e1 should still have Hp");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "CellsPlugin should register reset_inactive_sequence_hp between ApplyDamage and DetectDeaths, got {}",
        hp.current
    );
    assert!(app.world().get::<Dead>(e1).is_none());
}

/// Behavior 32: `CellsPlugin` registers `advance_sequence` after
/// `EffectV3Systems::Death`.
#[test]
fn cells_plugin_registers_advance_sequence_after_effect_v3_death() {
    let mut app = sequence_plugin_app_loading();

    let e0 = spawn_plugin_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_plugin_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);

    sequence_plugin_advance_to_playing(&mut app);

    app.world_mut()
        .resource_mut::<PluginTestPendingCellDamage>()
        .0
        .push(plugin_damage_msg(e0, 25.0));

    for _ in 0..2 {
        tick(&mut app);
    }

    assert!(
        app.world().get_entity(e0).is_err() || app.world().get::<Dead>(e0).is_some(),
        "e0 should be dead after lethal damage"
    );
    assert!(
        app.world().get::<SequenceActive>(e1).is_some(),
        "CellsPlugin should register advance_sequence after EffectV3Systems::Death"
    );
}
