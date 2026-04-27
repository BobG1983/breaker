use bevy::prelude::*;

use crate::{RantzDmgPlugin, sets::DmgSystems};

// ── Behavior 47: Adding `RantzDmgPlugin` configures all 16 `DmgSystems`
//     variants as sets in `FixedUpdate` usable as `.before(...)` /
//     `.after(...)` anchors ──

#[test]
fn all_sixteen_sets_are_usable_as_before_and_after_anchors() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);

    let variants = [
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

    for v in variants {
        app.add_systems(FixedUpdate, (|| {}).after(v));
    }
    // Edge case: also add .before for each of the 16 — 32 no-op systems
    // total.
    for v in variants {
        app.add_systems(FixedUpdate, (|| {}).before(v));
    }

    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}
