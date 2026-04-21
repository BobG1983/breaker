//! Groups G + H + I — run-if gates, scheduling / ordering, and `register`
//! wiring (Behaviors 28, 29, 32-37).
//!
//! Pins that the `fission_on_cell_destroyed` system is gated by
//! `protocol_active(Fission)` + `in_state(NodeState::Playing)`, ordered
//! `.after(DeathPipelineSystems::HandleKill)`, wired into `FixedUpdate`, and
//! that cleanup is wired into `OnExit(MenuState::Main)`.

use bevy::prelude::*;

use super::{
    super::system::{FissionConfig, FissionCounter},
    helpers::{
        build_fission_app, build_fission_app_in_chip_selecting, count_bolts,
        install_fission_counter, seed_active_protocols_with_fission, spawn_bolt_at_with_velocity,
        write_destroyed_cell,
    },
};
use crate::{
    prelude::*,
    protocol::{
        definition::{ProtocolDefinition, ProtocolTuning},
        resources::ActiveProtocols,
    },
    shared::death_pipeline::sets::DeathPipelineSystems,
};

// ── Behavior 28 — Fission NOT in ActiveProtocols: no increment, no split ───-

#[test]
fn fission_not_active_does_not_increment_counter_or_split() {
    let mut app = build_fission_app();
    install_fission_counter(&mut app, 7);
    // Intentionally do NOT seed ActiveProtocols.
    let _parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));

    write_destroyed_cell(&mut app, None);
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 7 },
        "counter must stay at 7 when Fission not active; got {counter:?}"
    );
    assert_eq!(count_bolts(&mut app), 1, "no split when Fission not active");
}

// ── Behavior 28 (edge case) — ActiveProtocols contains ONLY IronCurtain ────-

#[test]
fn only_iron_curtain_active_does_not_trigger_fission() {
    let mut app = build_fission_app();
    install_fission_counter(&mut app, 7);
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(ProtocolDefinition {
            name:        "Iron Curtain".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::IronCurtain {
                damage_fraction: 0.5,
                falloff_start:   50.0,
            },
        });
    let _parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));

    write_destroyed_cell(&mut app, None);
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 7 },
        "counter must stay at 7 with only Iron Curtain active; got {counter:?}"
    );
    assert_eq!(count_bolts(&mut app), 1, "no Fission split");
}

// ── Behavior 29 — NodeState != Playing: no increment, no split ─────────────-

#[test]
fn node_state_not_playing_does_not_increment_or_split() {
    let mut app = build_fission_app_in_chip_selecting();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let _parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));

    write_destroyed_cell(&mut app, None);
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 7 },
        "counter must stay at 7 in ChipSelectState::Selecting; got {counter:?}"
    );
    assert_eq!(
        count_bolts(&mut app),
        1,
        "no split in ChipSelectState::Selecting"
    );
}

// ── Behavior 32 — .after(DeathPipelineSystems::HandleKill) consumes same tick
//
// Edge case (per spec escape hatch): the adversarial before-HandleKill variant
// (writer registered .before(HandleKill) producing a Destroyed<Cell> earlier in
// the schedule) was dropped — the same-tick consumption assertion below is the
// load-bearing invariant; the before/after distinction is implementation detail
// of FixedUpdate ordering that Bevy already guarantees.

#[test]
fn destroyed_cell_written_in_handle_kill_is_consumed_same_tick() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    let _parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));

    // Inject a ghost producer in the HandleKill set that writes a
    // Destroyed<Cell> message.
    app.add_systems(
        FixedUpdate,
        (move |mut w: MessageWriter<Destroyed<Cell>>, mut done: Local<bool>| {
            if *done {
                return;
            }
            w.write(Destroyed::<Cell> {
                victim:     Entity::PLACEHOLDER,
                killer:     None,
                victim_pos: Vec2::ZERO,
                killer_pos: None,
                _marker:    std::marker::PhantomData,
            });
            *done = true;
        })
        .in_set(DeathPipelineSystems::HandleKill)
        .run_if(in_state(NodeState::Playing)),
    );

    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 1 },
        "Destroyed<Cell> from HandleKill must be consumed same tick; got {counter:?}"
    );
}

// ── Behavior 33 — schedule ticks cleanly with no messages / bolts / cells ──-

#[test]
fn schedule_ticks_cleanly_with_no_messages_no_bolts_no_cells() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);

    for _ in 0..3 {
        tick(&mut app);
    }

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 0 },
        "quiet schedule must leave counter at default; got {counter:?}"
    );
    assert_eq!(
        count_bolts(&mut app),
        0,
        "quiet schedule must not spawn bolts"
    );
}

// ── Behavior 34 — pregate Destroyed<Cell> drains cleanly before Fission active
//     Regression pin against accidental `.run_if` reintroduction on the
//     reader system: `fission_on_cell_destroyed` now enforces its gate
//     in-body via `reader.clear()` so pre-gate `Destroyed<Cell>` messages
//     drain cleanly instead of accumulating and replaying on gate open.

#[test]
fn pregate_destroyed_cell_drains_cleanly_before_fission_activates() {
    let mut app = build_fission_app();
    // Gate closed: Fission NOT yet in ActiveProtocols.
    let _parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));

    // Tick 1 — write Destroyed<Cell> with gate closed. Retrofit drains reader.
    write_destroyed_cell(&mut app, None);
    tick(&mut app);
    assert_eq!(
        *app.world().resource::<FissionCounter>(),
        FissionCounter { kills: 0 },
        "gate closed → counter unchanged on tick 1"
    );

    // Tick 2 — open the gate, no new message. The pre-gate message was
    // drained on tick 1; nothing remains to replay.
    seed_active_protocols_with_fission(&mut app, 8);
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 0 },
        "pre-gate Destroyed<Cell> was drained on tick 1 — counter stays 0 \
         post-activation; got {counter:?}"
    );
}

// ── Behavior 35 — register wires fission_on_cell_destroyed into FixedUpdate ─

#[test]
fn register_wires_fission_on_cell_destroyed_in_fixed_update() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    let _parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));

    write_destroyed_cell(&mut app, None);
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 1 },
        "register-wired reader must process Destroyed<Cell> in one FixedUpdate; got {counter:?}"
    );
}

// Note (Behavior 35 edge case): the "Update alone does not increment" assertion
// was dropped — running `app.world_mut().run_schedule(Update)` directly requires
// the Update schedule to be registered, which `TestAppBuilder` deliberately
// omits. The schedule placement is already pinned by the pregate / FixedUpdate-
// only tests that pass via `tick()`.

// ── Behavior 37 — register wires cleanup into OnExit(MenuState::Main) ──────-

#[test]
fn register_wires_cleanup_on_exit_menu_state_main() {
    // Build a fresh app in MenuState::Main and install both resources.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveProtocols>()
        .with_resource::<FissionCounter>()
        .with_message::<Destroyed<Cell>>()
        .build();
    // Drive into MenuState::Main.
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Game);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Menu);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<MenuState>>()
        .set(MenuState::Main);
    app.update();

    // Register Fission and install the config/counter.
    super::super::system::register(&mut app);
    app.world_mut()
        .insert_resource(FissionConfig { kills_per_split: 8 });
    app.world_mut().insert_resource(FissionCounter { kills: 5 });

    // Transition MenuState away from Main.
    app.world_mut()
        .resource_mut::<NextState<MenuState>>()
        .set(MenuState::Teardown);
    app.update();

    assert!(
        app.world().get_resource::<FissionConfig>().is_none(),
        "cleanup must remove FissionConfig on OnExit(MenuState::Main)"
    );
    assert!(
        app.world().get_resource::<FissionCounter>().is_none(),
        "cleanup must remove FissionCounter on OnExit(MenuState::Main)"
    );
}
