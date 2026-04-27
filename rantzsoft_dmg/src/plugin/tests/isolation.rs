use bevy::prelude::*;

use super::helpers::{
    ExecutionLog, FixedTickCount, PostCounters, TestT, bump_post_apply, bump_post_emit,
    bump_post_heal, bump_post_kill, bump_post_mutate, fixed_tick_witness,
};
use crate::{RantzDmgAppExt, RantzDmgPlugin, sets::DmgSystems};

// ── Behavior 53: `RantzDmgPlugin` does not inject any systems into
//     `FixedUpdate` beyond what consumers add ──

#[test]
fn plugin_injects_no_systems_into_fixed_update_beyond_consumer_additions() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);
    app.init_resource::<ExecutionLog>();
    app.init_resource::<FixedTickCount>();
    app.add_systems(FixedUpdate, fixed_tick_witness);

    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();

    assert_eq!(
        app.world().resource::<FixedTickCount>().0,
        1,
        "FixedUpdate must run exactly once (positive control)"
    );
    assert!(
        app.world().resource::<ExecutionLog>().0.is_empty(),
        "RantzDmgPlugin must not inject any recorder system that writes \
         to ExecutionLog — the 16 DmgSystems sets are empty until \
         consumers attach systems"
    );
}

#[test]
fn plugin_injects_no_systems_across_two_fixed_update_ticks() {
    // Edge case: two ticks — FixedTickCount should be 2 and
    // ExecutionLog should still be empty.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);
    app.init_resource::<ExecutionLog>();
    app.init_resource::<FixedTickCount>();
    app.add_systems(FixedUpdate, fixed_tick_witness);

    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    // First tick.
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
    // Second tick — two separate accumulate+update pairs (Duration * f32
    // does not compile in Bevy 0.18).
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();

    assert_eq!(app.world().resource::<FixedTickCount>().0, 2);
    assert!(app.world().resource::<ExecutionLog>().0.is_empty());
}

// ── W2 Behavior 4: Post* sets ship empty — no crate-owned system is
//     scheduled into any of them ──
//
// Each Post* set gets a single sentinel system that bumps a counter. After
// one FixedUpdate tick, the counter must read exactly 1 (the sentinel
// only). Any additional increment means the crate wired a system into the
// Post* set, which violates "ships empty".

#[test]
fn post_sets_ship_empty_sentinel_is_only_system() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);
    app.init_resource::<PostCounters>();
    app.add_systems(
        FixedUpdate,
        (
            bump_post_emit.in_set(DmgSystems::PostEmitDamage),
            bump_post_mutate.in_set(DmgSystems::PostMutateDamage),
            bump_post_apply.in_set(DmgSystems::PostApplyDamage),
            bump_post_kill.in_set(DmgSystems::PostApplyKill),
            bump_post_heal.in_set(DmgSystems::PostApplyHeal),
        ),
    );

    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();

    let counters = app.world().resource::<PostCounters>();
    assert_eq!(
        counters.emit, 1,
        "RantzDmgPlugin must ship DmgSystems::PostEmitDamage empty — sentinel fired once, \
         but count reads {} (extra consumer/crate system?)",
        counters.emit
    );
    assert_eq!(counters.mutate, 1);
    assert_eq!(counters.apply, 1);
    assert_eq!(counters.kill, 1);
    assert_eq!(counters.heal, 1);
}

#[test]
fn post_sets_ship_empty_with_register_dmgable() {
    // Edge case: calling register_dmgable::<T>() for a dummy T does NOT
    // populate any Post* set. Confirms register_dmgable does not schedule
    // per-T systems into Post* sets.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);
    let _ = app.register_dmgable::<TestT>();
    app.init_resource::<PostCounters>();
    app.add_systems(
        FixedUpdate,
        (
            bump_post_emit.in_set(DmgSystems::PostEmitDamage),
            bump_post_mutate.in_set(DmgSystems::PostMutateDamage),
            bump_post_apply.in_set(DmgSystems::PostApplyDamage),
            bump_post_kill.in_set(DmgSystems::PostApplyKill),
            bump_post_heal.in_set(DmgSystems::PostApplyHeal),
        ),
    );

    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();

    let counters = app.world().resource::<PostCounters>();
    assert_eq!(counters.emit, 1);
    assert_eq!(counters.mutate, 1);
    assert_eq!(counters.apply, 1);
    assert_eq!(counters.kill, 1);
    assert_eq!(counters.heal, 1);
}
