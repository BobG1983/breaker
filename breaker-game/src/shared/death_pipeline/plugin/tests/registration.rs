use bevy::prelude::*;

use super::super::system::*;
use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    cells::{behaviors::survival::salvo::components::Salvo, components::Cell},
    shared::death_pipeline::heal_dealt::HealDealt,
    walls::components::Wall,
};

// ── Group K / Behavior 32: HealDealt<T> registered for all 5 types ───

#[test]
fn plugin_registers_heal_dealt_for_cell() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(DeathPipelinePlugin);
    assert!(
        app.world()
            .get_resource::<Messages<HealDealt<Cell>>>()
            .is_some(),
        "DeathPipelinePlugin should register Messages<HealDealt<Cell>>"
    );
}

#[test]
fn plugin_registers_heal_dealt_for_bolt() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(DeathPipelinePlugin);
    assert!(
        app.world()
            .get_resource::<Messages<HealDealt<Bolt>>>()
            .is_some(),
        "DeathPipelinePlugin should register Messages<HealDealt<Bolt>>"
    );
}

#[test]
fn plugin_registers_heal_dealt_for_wall() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(DeathPipelinePlugin);
    assert!(
        app.world()
            .get_resource::<Messages<HealDealt<Wall>>>()
            .is_some(),
        "DeathPipelinePlugin should register Messages<HealDealt<Wall>>"
    );
}

#[test]
fn plugin_registers_heal_dealt_for_breaker() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(DeathPipelinePlugin);
    assert!(
        app.world()
            .get_resource::<Messages<HealDealt<Breaker>>>()
            .is_some(),
        "DeathPipelinePlugin should register Messages<HealDealt<Breaker>>"
    );
}

#[test]
fn plugin_registers_heal_dealt_for_salvo() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(DeathPipelinePlugin);
    assert!(
        app.world()
            .get_resource::<Messages<HealDealt<Salvo>>>()
            .is_some(),
        "DeathPipelinePlugin should register Messages<HealDealt<Salvo>>"
    );
}
