//! W5 — whitelist regression pin for `diffusion_emit_rings`.
//!
//! `diffusion_emit_rings` is a ripple emitter that fires in
//! `DmgSystems::PostApplyDamage` after a primary `DamageDealt<Cell>` applies.
//! It must NEVER be moved into `DmgSystems::EmitDamage`. These assertions
//! catch two failure modes:
//!
//! 1. Accidental removal from `PostApplyDamage` (e.g., misregistration).
//! 2. Accidental move into `EmitDamage` (which would let Diffusion emit
//!    rings BEFORE any primary damage applies — semantically wrong).

use std::marker::PhantomData;

use bevy::prelude::*;

use super::{
    super::system::{DiffusionInstances, PendingDiffusionEmissions, diffusion_emit_rings, wire},
    helpers::canonical_config,
};
use crate::{
    mutators::{
        hazards::{definition::HazardKind, resources::ActiveHazards},
        plugin::wire_damage_chain,
        protocols::resources::ActiveProtocols,
    },
    prelude::*,
};

fn diffusion_scheduling_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_resource::<ActiveProtocols>()
        .with_effects_pipeline()
        .build();
    wire(&mut app);
    wire_damage_chain(&mut app);
    app
}

#[test]
fn diffusion_emit_rings_is_pinned_to_post_apply_damage() {
    let mut app = diffusion_scheduling_app();

    assert!(
        system_in_set(
            &mut app,
            FixedUpdate,
            diffusion_emit_rings,
            DmgSystems::PostApplyDamage,
        ),
        "diffusion_emit_rings must remain in DmgSystems::PostApplyDamage. If \
         it is removed entirely, the ripple semantics break (diffusion rings \
         never fire). If it is moved into EmitDamage, rings fire BEFORE any \
         primary damage applies — semantically wrong."
    );
}

#[test]
fn diffusion_emit_rings_is_not_in_emit_damage() {
    let mut app = diffusion_scheduling_app();

    assert!(
        !system_in_set(
            &mut app,
            FixedUpdate,
            diffusion_emit_rings,
            DmgSystems::EmitDamage,
        ),
        "diffusion_emit_rings must NOT be a member of DmgSystems::EmitDamage. \
         That would let Diffusion emit ripple rings in the emit stage — BEFORE \
         any primary damage applies — which inverts the intended post-apply \
         semantics."
    );
}

// ════════════════════════════════════════════════════════════════════
// W2 Behavior 53 — `wire(app)` + `wire_damage_chain(app)` together schedule
// `reduce_primary` in `MutateDamage` and `emit_rings` in `PostApplyDamage`.
//
// W2 Behavior 56 (cross-mechanic PostApplyDamage emitter ordering) lives in
// `mutators/plugin/tests/damage_chain.rs`, which exercises
// `MutatorsPlugin::wire_damage_chain` directly.
// ════════════════════════════════════════════════════════════════════

#[test]
fn register_wires_systems_into_dmg_sets() {
    // After wire(app) + wire_damage_chain(app) + 1 tick with Diffusion
    // active + primary msg, msg.amount must be reduced (proves
    // reduce_primary ran in MutateDamage).
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_resource::<ActiveHazards>()
        .with_resource::<ActiveProtocols>()
        .with_resource::<PendingDiffusionEmissions>()
        .with_resource::<DiffusionInstances>()
        .build();
    app.world_mut().insert_resource(canonical_config());
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Diffusion);
    wire(&mut app);
    wire_damage_chain(&mut app);

    let c0 = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::ZERO),
            Hp::new(100.0),
            KilledBy { killer: None },
        ))
        .id();
    // A second adjacent cell so the diffusion BFS has somewhere to splash
    // ripple damage; the test only asserts on the primary's HP reduction.
    app.world_mut().spawn((
        Cell,
        Position2D(Vec2::new(30.0, 0.0)),
        Hp::new(100.0),
        KilledBy { killer: None },
    ));

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: None,
            target:        c0,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    let c0_hp = app.world().get::<Hp>(c0).expect("Hp").current;
    assert!(
        c0_hp < 100.0,
        "C0 HP must be reduced: wire must schedule reduce_primary in MutateDamage"
    );
}
