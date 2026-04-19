//! Section D — `cleanup_broken_tether_links` system (Behaviours 36–43).
//!
//! Runs in `FixedUpdate` after `DeathPipelineSystems::HandleKill`, gated by
//! `hazard_active(Tether)` AND `in_state(NodeState::Playing)`. Removes
//! `TetherLink` from any surviving cell whose partner has been despawned or
//! marked `Dead`.

use bevy::prelude::*;

use super::{
    super::system::{TetherLink, register},
    helpers::{
        add_hazard_stacks, add_tether_stacks, build_cleanup_tether_app,
        build_cleanup_tether_app_not_playing, canonical_tether_config, install_tether_config,
        run_fixed_update, spawn_linked_pair,
    },
};
use crate::{
    hazard::{definition::HazardKind, resources::ActiveHazards},
    prelude::*,
};

// ── Behavior 36 — Partner despawned → surviving cell loses TetherLink ────────

#[test]
fn partner_despawned_removes_surviving_tether_link() {
    let mut app = build_cleanup_tether_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::new(0.0, 0.0), Vec2::new(50.0, 0.0));

    app.world_mut().despawn(b);
    run_fixed_update(&mut app);

    assert!(
        app.world().get::<TetherLink>(a).is_none(),
        "surviving cell A must lose TetherLink when partner B despawned"
    );
    assert!(
        app.world().get_entity(b).is_err(),
        "partner B should not exist after despawn"
    );
}

// ── Behavior 37 — Partner marked Dead → surviving cell loses TetherLink ──────

#[test]
fn partner_marked_dead_removes_surviving_tether_link() {
    let mut app = build_cleanup_tether_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::new(0.0, 0.0), Vec2::new(50.0, 0.0));

    app.world_mut().entity_mut(b).insert(Dead);
    run_fixed_update(&mut app);

    assert!(
        app.world().get::<TetherLink>(a).is_none(),
        "cell A must lose TetherLink when partner B is Dead-marked"
    );
}

#[test]
fn both_partners_dead_marked_both_lose_tether_link() {
    let mut app = build_cleanup_tether_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::new(0.0, 0.0), Vec2::new(50.0, 0.0));

    app.world_mut().entity_mut(a).insert(Dead);
    app.world_mut().entity_mut(b).insert(Dead);
    run_fixed_update(&mut app);

    assert!(
        app.world().get::<TetherLink>(a).is_none(),
        "both cells Dead → both lose TetherLink"
    );
    assert!(app.world().get::<TetherLink>(b).is_none());
}

// ── Behavior 38 — Both partners alive → no component removed ─────────────────

#[test]
fn both_partners_alive_leaves_tether_links_intact() {
    let mut app = build_cleanup_tether_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::new(0.0, 0.0), Vec2::new(50.0, 0.0));

    run_fixed_update(&mut app);

    let link_a = app.world().get::<TetherLink>(a).expect("A still has link");
    assert_eq!(link_a.partner, b, "A.partner must still be B");
    let link_b = app.world().get::<TetherLink>(b).expect("B still has link");
    assert_eq!(link_b.partner, a, "B.partner must still be A");
}

// ── Behavior 39 — Multiple broken links in one tick ──────────────────────────

#[test]
fn multiple_broken_links_in_one_tick_each_unlinks_independently() {
    let mut app = build_cleanup_tether_app();
    let (p1_left, p1_right) =
        spawn_linked_pair(&mut app, Vec2::new(0.0, 0.0), Vec2::new(50.0, 0.0));
    let (p2_left, p2_right) =
        spawn_linked_pair(&mut app, Vec2::new(200.0, 0.0), Vec2::new(250.0, 0.0));
    let (p3_left, p3_right) =
        spawn_linked_pair(&mut app, Vec2::new(400.0, 0.0), Vec2::new(450.0, 0.0));

    app.world_mut().despawn(p1_right);
    app.world_mut().entity_mut(p2_right).insert(Dead);
    run_fixed_update(&mut app);

    assert!(
        app.world().get::<TetherLink>(p1_left).is_none(),
        "pair 1 left must lose link (partner despawned)"
    );
    assert!(
        app.world().get::<TetherLink>(p2_left).is_none(),
        "pair 2 left must lose link (partner Dead)"
    );
    let link_left = app
        .world()
        .get::<TetherLink>(p3_left)
        .expect("pair 3 left should still have link");
    assert_eq!(link_left.partner, p3_right);
    let link_right = app
        .world()
        .get::<TetherLink>(p3_right)
        .expect("pair 3 right should still have link");
    assert_eq!(link_right.partner, p3_left);
}

// ── Behavior 40 — Cleanup runs after DeathPipelineSystems::HandleKill ────────

#[test]
fn cleanup_runs_after_dead_marker_inserted() {
    let mut app = build_cleanup_tether_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::new(0.0, 0.0), Vec2::new(50.0, 0.0));

    app.world_mut().entity_mut(b).insert(Dead);
    run_fixed_update(&mut app);

    assert!(
        app.world().get::<TetherLink>(a).is_none(),
        "A's TetherLink must be removed in the same tick Dead is present on B"
    );
}

// ── Behavior 41 — System does NOT run when Tether inactive ───────────────────

#[test]
fn cleanup_does_not_run_when_tether_inactive() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message::<DamageDealt<Cell>>()
        .build();
    install_tether_config(&mut app, canonical_tether_config());
    add_hazard_stacks(&mut app, HazardKind::Volatility, 1);
    register(&mut app);

    let (a, b) = spawn_linked_pair(&mut app, Vec2::new(0.0, 0.0), Vec2::new(50.0, 0.0));
    app.world_mut().despawn(b);
    run_fixed_update(&mut app);

    let link_a = app
        .world()
        .get::<TetherLink>(a)
        .expect("A should still hold TetherLink (cleanup gated off)");
    assert_eq!(link_a.partner, b);
}

// ── Behavior 42 — System does NOT run when not in NodeState::Playing ─────────

#[test]
fn cleanup_does_not_run_when_not_in_playing_state() {
    let mut app = build_cleanup_tether_app_not_playing();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::new(0.0, 0.0), Vec2::new(50.0, 0.0));

    app.world_mut().despawn(b);
    run_fixed_update(&mut app);

    let link_a = app
        .world()
        .get::<TetherLink>(a)
        .expect("A should still hold TetherLink (state-gate off)");
    assert_eq!(link_a.partner, b);
}

// ── Behavior 43 — System does NOT panic when TetherConfig absent ─────────────

#[test]
fn cleanup_does_not_panic_without_tether_config() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message::<DamageDealt<Cell>>()
        .build();
    add_tether_stacks(&mut app, 1);
    register(&mut app);

    let (a, b) = spawn_linked_pair(&mut app, Vec2::new(0.0, 0.0), Vec2::new(50.0, 0.0));
    app.world_mut().despawn(b);
    run_fixed_update(&mut app);

    assert!(
        app.world().get::<TetherLink>(a).is_none(),
        "cleanup must still remove A's link without reading TetherConfig"
    );
}
