use bevy::prelude::*;

use super::helpers::{PreDespawnWitnessCount, pre_despawn_witness};
use crate::{RantzDmgPlugin, messages::DespawnEntity, systems::process_despawn_requests};

// ── Behavior 50: Adding `RantzDmgPlugin` adds `process_despawn_requests`
//     to `FixedPostUpdate` ──

#[test]
fn process_despawn_requests_scheduled_in_fixed_post_update() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);
    app.init_resource::<PreDespawnWitnessCount>();
    app.add_systems(
        FixedPostUpdate,
        pre_despawn_witness.before(process_despawn_requests),
    );

    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();

    assert_eq!(
        app.world().resource::<PreDespawnWitnessCount>().0,
        1,
        "pre_despawn_witness.before(process_despawn_requests) should have \
         fired once — this proves process_despawn_requests exists in \
         FixedPostUpdate as an ordering anchor"
    );
}

#[test]
fn plugin_tolerates_empty_despawn_queue() {
    // Edge case: plugin does not panic when ticked with zero
    // DespawnEntity messages pending.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);

    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

#[test]
fn plugin_tolerates_single_despawn_message_in_queue() {
    // Edge case (stronger): write one `DespawnEntity` message, tick
    // once, assert no panic. We do NOT assert the message was consumed
    // — that is P6's job.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);

    app.world_mut()
        .resource_mut::<Messages<DespawnEntity>>()
        .write(DespawnEntity {
            entity: Entity::PLACEHOLDER,
        });

    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

// ── Behavior 51: RESERVED NUMBER GAP — no test ──
//
// Behavior 51 (duplicate-plugin add policy) is intentionally out of scope
// for P4 per the test spec: Bevy's built-in duplicate-plugin handling is
// not a P4-level decision. The number is preserved in the global
// behavior ladder to avoid renumbering across phases.

// ── Behavior 52: Adding `RantzDmgPlugin` to a headless app does not
//     panic ──

#[test]
fn adding_plugin_to_headless_app_does_not_panic() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);
    app.update();
}

#[test]
fn adding_plugin_to_headless_app_does_not_panic_with_fixed_tick() {
    // Edge case: tick once with accumulated FixedUpdate overstep.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);

    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}
