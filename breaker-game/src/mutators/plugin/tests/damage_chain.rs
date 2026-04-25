//! Cross-mechanic ordering pins for [`super::super::system::wire_damage_chain`].
//!
//! `wire_damage_chain` is the single source of truth for game-side
//! `DmgSystems::MutateDamage` and `DmgSystems::PostApplyDamage` ordering.
//! The `PostApplyDamage` ripple chain runs in the documented order:
//!
//! ```text
//! diffusion_emit_rings → tether_emit_partner → echo_strike_emit_siblings
//! ```
//!
//! These tests pin the ordering by attaching `record_*` recorders with
//! `.after(target_system)` and asserting the recorded execution order
//! after a tick. Recorders carry no `run_if` of their own, so they run
//! and capture order even when the chain participants are gated off —
//! the ordering edge is what matters.
//!
//! The pairwise tests isolate one pair at a time so a regression that
//! breaks only one edge (e.g. tether before diffusion) still surfaces.

use bevy::prelude::*;

use super::super::system::wire_damage_chain;
use crate::{
    mutators::{
        hazards::{
            diffusion::system::{
                PendingDiffusionEmissions, diffusion_emit_rings, diffusion_reduce_primary,
            },
            resources::ActiveHazards,
            tether::system::tether_emit_partner,
        },
        protocols::{echo_strike::system::echo_strike_emit_siblings, resources::ActiveProtocols},
    },
    prelude::*,
};

#[derive(Resource, Default)]
struct PostApplyOrder(Vec<&'static str>);

fn record_diffusion(mut log: ResMut<PostApplyOrder>) {
    log.0.push("diffusion_emit_rings");
}
fn record_tether(mut log: ResMut<PostApplyOrder>) {
    log.0.push("tether_emit_partner");
}
fn record_echo(mut log: ResMut<PostApplyOrder>) {
    log.0.push("echo_strike_emit_siblings");
}

/// Builds an app with `wire_damage_chain` installed and a fresh
/// `PostApplyOrder` recorder. Resources required by the chain
/// participants' parameters (`Option<Res<ActiveHazards>>`,
/// `Option<Res<ActiveProtocols>>`, `PendingDiffusionEmissions`) are
/// inserted so the schedule resolves cleanly even though no participant
/// runs (their `run_if` gates evaluate false against an empty active set,
/// but `.after()` ordering is still applied).
fn build_chain_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_resource::<ActiveHazards>()
        .with_resource::<ActiveProtocols>()
        .with_resource::<PendingDiffusionEmissions>()
        .build();
    app.init_resource::<crate::mutators::hazards::diffusion::system::DiffusionInstances>();
    app.init_resource::<PostApplyOrder>();
    wire_damage_chain(&mut app);
    app
}

/// `diffusion_reduce_primary` is the sole `MutateDamage` participant.
/// Pin its set membership centrally so a regression that drops the
/// `.in_set(MutateDamage)` tag in `wire_damage_chain` is caught here
/// even if no per-mechanic test exercises the central wiring.
#[test]
fn diffusion_reduce_primary_is_pinned_to_mutate_damage() {
    let mut app = build_chain_app();

    assert!(
        system_in_set(
            &mut app,
            FixedUpdate,
            diffusion_reduce_primary,
            DmgSystems::MutateDamage,
        ),
        "wire_damage_chain must register diffusion_reduce_primary in DmgSystems::MutateDamage"
    );
}

/// Negative: `diffusion_reduce_primary` must NOT live in any later
/// `Dmg` set (`PostApplyDamage`, `ApplyDamage`, etc.) — running it
/// after `MutateDamage` would let `apply_damage::<Cell>` see the
/// pre-reduction primary.
#[test]
fn diffusion_reduce_primary_is_not_in_post_apply_damage() {
    let mut app = build_chain_app();

    assert!(
        !system_in_set(
            &mut app,
            FixedUpdate,
            diffusion_reduce_primary,
            DmgSystems::PostApplyDamage,
        ),
        "diffusion_reduce_primary must NOT be in DmgSystems::PostApplyDamage \
         — that would let apply_damage::<Cell> see the unreduced primary"
    );
}

#[test]
fn post_apply_emitters_run_in_order_diffusion_tether_echo() {
    let mut app = build_chain_app();

    app.add_systems(
        FixedUpdate,
        (
            record_diffusion
                .in_set(DmgSystems::PostApplyDamage)
                .after(diffusion_emit_rings),
            record_tether
                .in_set(DmgSystems::PostApplyDamage)
                .after(tether_emit_partner),
            record_echo
                .in_set(DmgSystems::PostApplyDamage)
                .after(echo_strike_emit_siblings),
        ),
    );

    tick(&mut app);

    let order = &app.world().resource::<PostApplyOrder>().0;
    assert_eq!(
        order,
        &vec![
            "diffusion_emit_rings",
            "tether_emit_partner",
            "echo_strike_emit_siblings",
        ],
        "wire_damage_chain must order PostApplyDamage emitters: \
         diffusion → tether → echo_strike"
    );
}

#[test]
fn post_apply_order_diffusion_before_tether() {
    let mut app = build_chain_app();

    app.add_systems(
        FixedUpdate,
        (
            record_diffusion
                .in_set(DmgSystems::PostApplyDamage)
                .after(diffusion_emit_rings),
            record_tether
                .in_set(DmgSystems::PostApplyDamage)
                .after(tether_emit_partner),
        ),
    );

    tick(&mut app);

    let order = &app.world().resource::<PostApplyOrder>().0;
    assert_eq!(
        order,
        &vec!["diffusion_emit_rings", "tether_emit_partner"],
        "diffusion_emit_rings must run before tether_emit_partner in PostApplyDamage"
    );
}

#[test]
fn post_apply_order_tether_before_echo() {
    let mut app = build_chain_app();

    app.add_systems(
        FixedUpdate,
        (
            record_tether
                .in_set(DmgSystems::PostApplyDamage)
                .after(tether_emit_partner),
            record_echo
                .in_set(DmgSystems::PostApplyDamage)
                .after(echo_strike_emit_siblings),
        ),
    );

    tick(&mut app);

    let order = &app.world().resource::<PostApplyOrder>().0;
    assert_eq!(
        order,
        &vec!["tether_emit_partner", "echo_strike_emit_siblings"],
        "tether_emit_partner must run before echo_strike_emit_siblings in PostApplyDamage"
    );
}

#[test]
fn post_apply_order_diffusion_before_echo() {
    let mut app = build_chain_app();

    app.add_systems(
        FixedUpdate,
        (
            record_diffusion
                .in_set(DmgSystems::PostApplyDamage)
                .after(diffusion_emit_rings),
            record_echo
                .in_set(DmgSystems::PostApplyDamage)
                .after(echo_strike_emit_siblings),
        ),
    );

    tick(&mut app);

    let order = &app.world().resource::<PostApplyOrder>().0;
    assert_eq!(
        order,
        &vec!["diffusion_emit_rings", "echo_strike_emit_siblings"],
        "diffusion_emit_rings must run before echo_strike_emit_siblings in PostApplyDamage"
    );
}
