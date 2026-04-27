use bevy::prelude::*;

use super::helpers::{ExecutionLog, build_app_with_recorders};
use crate::{RantzDmgPlugin, sets::DmgSystems};

// ── Behavior 48: Adding `RantzDmgPlugin` orders the 11 variants in the
//     exact plan/detail chain order within `FixedUpdate` ──

#[test]
fn chain_order_matches_plan_after_one_tick() {
    let mut app = build_app_with_recorders();

    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();

    let expected = vec![
        DmgSystems::EmitDamage,
        DmgSystems::PostEmitDamage,
        DmgSystems::ApplyDamageBoosts,
        DmgSystems::MutateDamage,
        DmgSystems::PostMutateDamage,
        DmgSystems::ApplyVulnerable,
        DmgSystems::ApplyDamage,
        DmgSystems::PostApplyDamage,
        DmgSystems::EmitKill,
        DmgSystems::MutateKill,
        DmgSystems::ApplyKill,
        DmgSystems::PostApplyKill,
        DmgSystems::EmitHeal,
        DmgSystems::MutateHeal,
        DmgSystems::ApplyHeal,
        DmgSystems::PostApplyHeal,
    ];

    assert_eq!(
        app.world().resource::<ExecutionLog>().0,
        expected,
        "DmgSystems chain ran in the wrong order — RantzDmgPlugin's \
         `configure_sets(FixedUpdate, (...).chain())` must list the 16 \
         variants in the canonical Emit/Mutate/Apply plan order"
    );
}

#[test]
fn chain_order_stable_across_two_ticks() {
    // Edge case: a second tick produces another 16 push events in the
    // same order. Clear the log after the first tick, run a second
    // tick, and assert the order is stable.
    let mut app = build_app_with_recorders();

    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();

    // Clear the log.
    app.world_mut().resource_mut::<ExecutionLog>().0.clear();

    // Second tick.
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();

    let expected = vec![
        DmgSystems::EmitDamage,
        DmgSystems::PostEmitDamage,
        DmgSystems::ApplyDamageBoosts,
        DmgSystems::MutateDamage,
        DmgSystems::PostMutateDamage,
        DmgSystems::ApplyVulnerable,
        DmgSystems::ApplyDamage,
        DmgSystems::PostApplyDamage,
        DmgSystems::EmitKill,
        DmgSystems::MutateKill,
        DmgSystems::ApplyKill,
        DmgSystems::PostApplyKill,
        DmgSystems::EmitHeal,
        DmgSystems::MutateHeal,
        DmgSystems::ApplyHeal,
        DmgSystems::PostApplyHeal,
    ];

    assert_eq!(app.world().resource::<ExecutionLog>().0, expected);
}

// ── Behavior 49: Adding `RantzDmgPlugin` places the chain in
//     `FixedUpdate`, not in `Update` ──

#[test]
fn chain_is_in_fixed_update_not_update() {
    let mut app = build_app_with_recorders();

    // app.update() WITHOUT accumulating overstep — FixedUpdate should
    // NOT run, and the recorders should not fire.
    app.update();

    assert!(
        app.world().resource::<ExecutionLog>().0.is_empty(),
        "ExecutionLog should be empty when FixedUpdate has no overstep \
         budget — recorders (and thus the DmgSystems sets) must be bound \
         to FixedUpdate, not Update"
    );

    // Positive control companion assertion: accumulate overstep and tick
    // once — now the 16 recorders should fire.
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();

    assert_eq!(
        app.world().resource::<ExecutionLog>().0.len(),
        16,
        "After one FixedUpdate tick, all 16 recorders should have fired"
    );
}

// `RantzDmgPlugin` is referenced via `super::helpers::build_app_with_recorders`
// — the explicit import keeps the symbol visible to the documentation-style
// `RantzDmgPlugin's …` comment above without resorting to a glob import.
const _: fn() = || {
    let _ = RantzDmgPlugin;
};
