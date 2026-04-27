use bevy::prelude::*;

use super::helpers::{TestT, TestU};
use crate::{
    RantzDmgPlugin,
    messages::{DamageDealt, DespawnEntity, Destroyed, HealDealt, KillYourself},
};

// ── Behavior 45: Adding `RantzDmgPlugin` registers `DespawnEntity` as a
//     Bevy message ──

#[test]
fn plugin_registers_despawn_entity_message() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);

    assert!(
        app.world().contains_resource::<Messages<DespawnEntity>>(),
        "RantzDmgPlugin should register Messages<DespawnEntity>"
    );
}

#[test]
fn baseline_without_plugin_does_not_register_despawn_entity() {
    // Edge case: baseline without `RantzDmgPlugin` must NOT have the
    // resource — proves the plugin is what registers it, not default
    // Bevy behavior.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    assert!(
        !app.world().contains_resource::<Messages<DespawnEntity>>(),
        "baseline App without RantzDmgPlugin must not have Messages<DespawnEntity>"
    );
}

// ── Behavior 46: Adding `RantzDmgPlugin` does NOT register any per-`T`
//     messages ──

#[test]
fn plugin_does_not_register_per_t_messages() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);

    // TestT — none of the four per-T messages are registered.
    assert!(
        !app.world()
            .contains_resource::<Messages<DamageDealt<TestT>>>()
    );
    assert!(
        !app.world()
            .contains_resource::<Messages<HealDealt<TestT>>>()
    );
    assert!(
        !app.world()
            .contains_resource::<Messages<KillYourself<TestT>>>()
    );
    assert!(
        !app.world()
            .contains_resource::<Messages<Destroyed<TestT>>>()
    );

    // Edge case: same for a second dummy type TestU — also all false.
    assert!(
        !app.world()
            .contains_resource::<Messages<DamageDealt<TestU>>>()
    );
    assert!(
        !app.world()
            .contains_resource::<Messages<HealDealt<TestU>>>()
    );
    assert!(
        !app.world()
            .contains_resource::<Messages<KillYourself<TestU>>>()
    );
    assert!(
        !app.world()
            .contains_resource::<Messages<Destroyed<TestU>>>()
    );
}
