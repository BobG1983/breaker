//! Tests for the cascade hazard retrofit to the unified heal pipeline.
//!
//! Most runtime tests observe emitted `HealDealt<Cell>` messages via
//! `MessageCollector<HealDealt<Cell>>`; Group G runs the full
//! `cascade_heal_on_death` → `apply_heal::<Cell>` pipeline and observes
//! post-tick `Hp.current`.

use std::marker::PhantomData;

use bevy::{ecs::message::Messages, prelude::*};
use rantzsoft_spatial2d::components::Position2D;

use super::system::*;
use crate::{
    cells::components::Cell,
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::ActiveHazards,
    },
    prelude::*,
    shared::death_pipeline::{
        Destroyed, HealCap, Hp, heal_dealt::HealDealt, sets::DeathPipelineSystems,
        systems::apply_heal,
    },
};

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Default builder: state hierarchy at `NodeState::Playing`, `ActiveHazards`,
/// `Destroyed<Cell>` registered, `HealDealt<Cell>` captured.
fn test_app_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message::<Destroyed<Cell>>()
        .with_message_capture::<HealDealt<Cell>>()
        .build()
}

fn spawn_cell_at(app: &mut App, pos: Vec2, current: f32, starting: f32) -> Entity {
    spawn_cell_at_with_max(app, pos, current, starting, None)
}

fn spawn_cell_at_with_max(
    app: &mut App,
    pos: Vec2,
    current: f32,
    starting: f32,
    max: Option<f32>,
) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            Hp {
                current,
                starting,
                max,
            },
        ))
        .id()
}

fn send_cell_destroyed(app: &mut App, victim: Entity, victim_pos: Vec2) {
    app.world_mut()
        .resource_mut::<Messages<Destroyed<Cell>>>()
        .write(Destroyed::<Cell> {
            victim,
            killer: None,
            victim_pos,
            killer_pos: None,
            _marker: PhantomData,
        });
}

fn run_fixed_update(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

fn heals_for_cell(app: &App, target: Entity) -> Vec<HealDealt<Cell>> {
    app.world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0
        .iter()
        .filter(|m| m.target == target)
        .cloned()
        .collect()
}

fn cascade_heal_collector_len(app: &App) -> usize {
    app.world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0
        .len()
}

fn add_cascade_stacks(app: &mut App, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(HazardKind::Cascade);
    }
}

fn install_cascade_config(app: &mut App, cfg: CascadeConfig) {
    app.world_mut().insert_resource(cfg);
}

// ════════════════════════════════════════════════════════════════════════════
// Group A — heal_per_neighbour formula (unchanged pure unit tests)
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn heal_zero_stacks_is_zero() {
    let cfg = CascadeConfig {
        base_heal:      1.0,
        per_level_heal: 0.5,
    };
    assert!((cfg.heal_per_neighbour(0) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn heal_zero_stacks_is_zero_even_with_large_config() {
    let cfg = CascadeConfig {
        base_heal:      999.0,
        per_level_heal: 999.0,
    };
    assert!((cfg.heal_per_neighbour(0) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn heal_stack_one_equals_base() {
    let cfg = CascadeConfig {
        base_heal:      10.0,
        per_level_heal: 5.0,
    };
    assert!((cfg.heal_per_neighbour(1) - 10.0).abs() < f32::EPSILON);
}

#[test]
fn heal_stack_one_with_zero_base_is_zero() {
    let cfg = CascadeConfig {
        base_heal:      0.0,
        per_level_heal: 5.0,
    };
    assert!((cfg.heal_per_neighbour(1) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn heal_stack_three_adds_two_levels() {
    let cfg = CascadeConfig {
        base_heal:      10.0,
        per_level_heal: 5.0,
    };
    // base 10.0 + 5.0 * 2 = 20.0.
    assert!((cfg.heal_per_neighbour(3) - 20.0).abs() < f32::EPSILON);
}

#[test]
fn heal_stack_five_matches_design_doc_value() {
    let cfg = CascadeConfig {
        base_heal:      10.0,
        per_level_heal: 5.0,
    };
    // base 10.0 + 5.0 * 4 = 30.0 (pinned by the design doc table).
    assert!((cfg.heal_per_neighbour(5) - 30.0).abs() < f32::EPSILON);
}

// ════════════════════════════════════════════════════════════════════════════
// Group B — activate preserved scaffold tests (unchanged)
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn activate_with_matching_tuning_inserts_config() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Cascade {
                base_heal:      1.0,
                per_level_heal: 0.5,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<CascadeConfig>();
    assert!((cfg.base_heal - 1.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_heal - 0.5).abs() < f32::EPSILON);
}

#[test]
fn activate_with_mismatched_tuning_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Decay {
                base_percent:      0.05,
                per_level_percent: 0.03,
            },
            &mut commands,
        );
    });
    app.update();
    assert!(app.world().get_resource::<CascadeConfig>().is_none());
}

// Behavior 4 edge case: second activate overwrites prior config (last-write-wins
// semantics from `commands.insert_resource`).
#[test]
fn second_activate_overwrites_prior_cascade_config() {
    let mut app = TestAppBuilder::new().build();
    app.world_mut().insert_resource(CascadeConfig {
        base_heal:      1.0,
        per_level_heal: 0.5,
    });
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Cascade {
                base_heal:      99.0,
                per_level_heal: 9.9,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<CascadeConfig>();
    assert!(
        (cfg.base_heal - 99.0).abs() < f32::EPSILON,
        "second activate must overwrite base_heal, got {}",
        cfg.base_heal
    );
    assert!(
        (cfg.per_level_heal - 9.9).abs() < f32::EPSILON,
        "second activate must overwrite per_level_heal, got {}",
        cfg.per_level_heal
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Group C — No-op guards
// ════════════════════════════════════════════════════════════════════════════

// Behavior 6: no message emitted when CascadeConfig resource is absent.
#[test]
fn no_message_when_config_absent() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 5.0, 10.0);
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(
        cascade_heal_collector_len(&app),
        0,
        "no HealDealt<Cell> should be emitted when CascadeConfig is absent"
    );
    // Neighbour's Hp must not have been mutated either — cascade_heal_on_death
    // has no business touching Hp, ever.
    let hp = app.world().get::<Hp>(neighbour).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "neighbour Hp must remain 5.0 when no config present, got {}",
        hp.current
    );
}

// Behavior 7: no message emitted when ActiveHazards.stacks(Cascade) == 0.
#[test]
fn no_message_when_zero_cascade_stacks() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 5.0,
        },
    );
    // No stacks added — heal_per_neighbour(0) == 0.0 triggers the early return.

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let _neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(
        cascade_heal_collector_len(&app),
        0,
        "no HealDealt<Cell> should be emitted when Cascade has zero stacks"
    );
}

// Behavior 7 edge: zero stacks + zero config → still no messages.
#[test]
fn no_message_when_zero_stacks_and_zero_config() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      0.0,
            per_level_heal: 0.0,
        },
    );

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let _neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 0);
}

// Behavior 8: no message emitted when there are no Destroyed<Cell> messages.
#[test]
fn no_message_when_no_destroyed_messages() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 5.0,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let _a = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 5.0, 10.0);
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    // No send_cell_destroyed call.

    run_fixed_update(&mut app);

    assert_eq!(
        cascade_heal_collector_len(&app),
        0,
        "no HealDealt<Cell> without any Destroyed<Cell> this tick"
    );
    let hp = app.world().get::<Hp>(neighbour).unwrap();
    assert!((hp.current - 5.0).abs() < f32::EPSILON);
}

// Behavior 9: run-condition gate — system skipped when Cascade inactive.
#[test]
fn system_skipped_when_cascade_run_condition_false() {
    let mut app = test_app_playing();
    // Use register(app) so the hazard_active + in_state run-conditions apply.
    register(&mut app);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 5.0,
        },
    );
    // Stack a different hazard (Volatility) — Cascade remains inactive.
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Volatility);
    assert_eq!(
        app.world()
            .resource::<ActiveHazards>()
            .stacks(HazardKind::Cascade),
        0
    );

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let _neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(
        cascade_heal_collector_len(&app),
        0,
        "hazard_active(Cascade) run-condition must gate off the system"
    );
}

// Behavior 10: run-condition gate — system skipped when NOT in NodeState::Playing.
#[test]
fn system_skipped_when_not_in_node_playing() {
    // Build app with state hierarchy but DO NOT drive into NodeState::Playing.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveHazards>()
        .with_message::<Destroyed<Cell>>()
        .with_message_capture::<HealDealt<Cell>>()
        .build();
    register(&mut app);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 5.0,
        },
    );
    add_cascade_stacks(&mut app, 3);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let _neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(
        cascade_heal_collector_len(&app),
        0,
        "in_state(NodeState::Playing) run-condition must gate off the system"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Group D — Single-death / single-neighbour message shape
// ════════════════════════════════════════════════════════════════════════════

// Behavior 11: single death + single neighbour → one HealDealt<Cell> with
// correct fields.
#[test]
fn single_death_single_neighbour_emits_one_message_with_correct_fields() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 1);
    let msgs = heals_for_cell(&app, neighbour);
    assert_eq!(msgs.len(), 1);
    let msg = &msgs[0];
    assert_eq!(msg.target, neighbour);
    assert!(
        (msg.amount - 1.0).abs() < f32::EPSILON,
        "amount should equal heal_per_neighbour(1) = 1.0, got {}",
        msg.amount
    );
    assert!(matches!(msg.cap, HealCap::Starting));
    assert_eq!(msg.healer, None);
    assert_eq!(msg.source, Some("hazard:cascade".to_string()));
}

// Behavior 11 edge: neighbour at exactly the radius boundary (distance² == 4900)
// is included.
#[test]
fn neighbour_at_exact_radius_boundary_is_included() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    // At distance exactly 70.0 — distance² == ADJACENCY_RADIUS_SQ (4900).
    let neighbour = spawn_cell_at(&mut app, Vec2::new(70.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 1);
    let msgs = heals_for_cell(&app, neighbour);
    assert_eq!(msgs.len(), 1);
    assert!((msgs[0].amount - 1.0).abs() < f32::EPSILON);
    assert!(matches!(msgs[0].cap, HealCap::Starting));
}

// Behavior 12: stack-3 → amount is base + 2 * per_level.
#[test]
fn stack_three_message_amount_is_base_plus_two_per_level() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 5.0,
        },
    );
    add_cascade_stacks(&mut app, 3);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 100.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 1);
    let msgs = heals_for_cell(&app, neighbour);
    assert_eq!(msgs.len(), 1);
    assert!(
        (msgs[0].amount - 20.0).abs() < f32::EPSILON,
        "amount should be 10.0 + 5.0*2 = 20.0, got {}",
        msgs[0].amount
    );
    assert!(matches!(msgs[0].cap, HealCap::Starting));
    assert_eq!(msgs[0].source, Some("hazard:cascade".to_string()));
}

// Behavior 12 edge: stack 5 produces amount == 30.0 (design-doc pinned).
#[test]
fn stack_five_message_amount_is_thirty() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 5.0,
        },
    );
    add_cascade_stacks(&mut app, 5);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 100.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let msgs = heals_for_cell(&app, neighbour);
    assert_eq!(msgs.len(), 1);
    assert!(
        (msgs[0].amount - 30.0).abs() < f32::EPSILON,
        "amount should be 30.0 at stack 5, got {}",
        msgs[0].amount
    );
}

// Behavior 13: neighbour outside radius → zero messages.
#[test]
fn neighbour_outside_radius_is_not_emitted() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    // distance² = 40000 > 4900
    let _far = spawn_cell_at(&mut app, Vec2::new(200.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 0);
}

// Behavior 13 edge: just-outside radius — (71.0, 0.0), distance² = 5041 > 4900.
#[test]
fn neighbour_just_outside_radius_is_not_emitted() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    // distance² = 71*71 = 5041 > 4900
    let _just_out = spawn_cell_at(&mut app, Vec2::new(71.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 0);
}

// Behavior 14: the Destroyed<Cell>.victim entity is never a heal target.
// Corrected per minor-spec-fix 2: set victim's Hp.current = 5.0 (>0.0) so
// this test pins the `*victim != entity` guard, not the `hp.current <= 0.0`
// guard.
#[test]
fn destroyed_victim_entity_is_never_self_healed() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    // Victim Hp.current = 5.0 (> 0.0) so the only guard that can exclude
    // this entity is the `*victim != entity` self-exclusion check.
    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(
        cascade_heal_collector_len(&app),
        0,
        "victim entity must never receive a self-heal message"
    );
    let self_msgs = heals_for_cell(&app, victim);
    assert!(self_msgs.is_empty());
}

// Behavior 14 edge: victim self-excluded, but adjacent neighbour still healed.
#[test]
fn self_exclusion_does_not_block_adjacent_neighbour_heal() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 1);
    let msgs = heals_for_cell(&app, neighbour);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].target, neighbour);
    // victim got none:
    assert!(heals_for_cell(&app, victim).is_empty());
}

// Behavior 15: a separately-dead cell (Hp.current <= 0.0) is not a heal
// target.
#[test]
fn separately_dead_cell_is_not_a_heal_target() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    let corpse = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 0.0, 10.0);
    // Separate victim entity at origin (its position is what triggers
    // adjacency); its Hp doesn't matter because the victim is referenced by
    // entity id in the message.
    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(
        cascade_heal_collector_len(&app),
        1,
        "exactly one message — for the living neighbour, none for the corpse"
    );
    let neighbour_msgs = heals_for_cell(&app, neighbour);
    assert_eq!(neighbour_msgs.len(), 1);
    assert_eq!(neighbour_msgs[0].target, neighbour);
    let corpse_msgs = heals_for_cell(&app, corpse);
    assert!(
        corpse_msgs.is_empty(),
        "corpse with Hp.current = 0.0 must not receive a message"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Group E — Multiple deaths / multiple neighbours
// ════════════════════════════════════════════════════════════════════════════

// Behavior 16: multiple neighbours within radius → one message per neighbour.
#[test]
fn multiple_neighbours_each_receive_one_message() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let n1 = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    let n2 = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 5.0, 10.0);
    let n3 = spawn_cell_at(&mut app, Vec2::new(0.0, 50.0), 5.0, 10.0);
    let n4 = spawn_cell_at(&mut app, Vec2::new(0.0, -50.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 4);
    for n in [n1, n2, n3, n4] {
        let msgs = heals_for_cell(&app, n);
        assert_eq!(msgs.len(), 1, "each neighbour gets exactly one message");
        assert!((msgs[0].amount - 1.0).abs() < f32::EPSILON);
        assert!(matches!(msgs[0].cap, HealCap::Starting));
        assert_eq!(msgs[0].source, Some("hazard:cascade".to_string()));
        assert_eq!(msgs[0].healer, None);
    }
}

// Behavior 16 edge: adding a fifth outside-radius cell — still exactly 4 messages.
#[test]
fn multiple_neighbours_with_one_outside_radius_still_four_messages() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let _n1 = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    let _n2 = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 5.0, 10.0);
    let _n3 = spawn_cell_at(&mut app, Vec2::new(0.0, 50.0), 5.0, 10.0);
    let _n4 = spawn_cell_at(&mut app, Vec2::new(0.0, -50.0), 5.0, 10.0);
    let far = spawn_cell_at(&mut app, Vec2::new(200.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 4);
    assert!(heals_for_cell(&app, far).is_empty());
}

// Behavior 17: multiple deaths, shared neighbour → N separate messages.
#[test]
fn multiple_deaths_shared_neighbour_emit_two_messages() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let n = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 5.0, 20.0);
    let victim_a = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 0.0, 10.0);
    let victim_b = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 0.0, 10.0);
    send_cell_destroyed(&mut app, victim_a, Vec2::new(50.0, 0.0));
    send_cell_destroyed(&mut app, victim_b, Vec2::new(-50.0, 0.0));

    run_fixed_update(&mut app);

    let msgs = heals_for_cell(&app, n);
    assert_eq!(
        msgs.len(),
        2,
        "two deaths adjacent to one neighbour → two separate HealDealt<Cell> messages"
    );
    for msg in &msgs {
        assert!((msg.amount - 1.0).abs() < f32::EPSILON);
        assert!(matches!(msg.cap, HealCap::Starting));
        assert_eq!(msg.source, Some("hazard:cascade".to_string()));
    }
}

// Behavior 17 edge: three shared-neighbour deaths → three messages.
#[test]
fn three_deaths_shared_neighbour_emit_three_messages() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let n = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 5.0, 20.0);
    let va = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 0.0, 10.0);
    let vb = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 0.0, 10.0);
    let vc = spawn_cell_at(&mut app, Vec2::new(0.0, 50.0), 0.0, 10.0);
    send_cell_destroyed(&mut app, va, Vec2::new(50.0, 0.0));
    send_cell_destroyed(&mut app, vb, Vec2::new(-50.0, 0.0));
    send_cell_destroyed(&mut app, vc, Vec2::new(0.0, 50.0));

    run_fixed_update(&mut app);

    let msgs = heals_for_cell(&app, n);
    assert_eq!(msgs.len(), 3);
}

// Behavior 18: multiple deaths with disjoint neighbours → union of per-death
// messages.
#[test]
fn disjoint_deaths_produce_one_message_each() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let a = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    let b = spawn_cell_at(&mut app, Vec2::new(-200.0, 0.0), 5.0, 10.0);
    let va = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0); // adjacent to a
    let vb = spawn_cell_at(&mut app, Vec2::new(-250.0, 0.0), 0.0, 10.0); // adjacent to b
    send_cell_destroyed(&mut app, va, Vec2::new(0.0, 0.0));
    send_cell_destroyed(&mut app, vb, Vec2::new(-250.0, 0.0));

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 2);
    assert_eq!(heals_for_cell(&app, a).len(), 1);
    assert_eq!(heals_for_cell(&app, b).len(), 1);
}

// Behavior 19: mixed case — one death, one neighbour in + one out.
#[test]
fn single_death_mixed_in_out_radius_emits_one_message() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let in_r = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    let out_r = spawn_cell_at(&mut app, Vec2::new(200.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 1);
    assert_eq!(heals_for_cell(&app, in_r).len(), 1);
    assert!(heals_for_cell(&app, out_r).is_empty());
}

// ════════════════════════════════════════════════════════════════════════════
// Group F — Message-field invariants
// ════════════════════════════════════════════════════════════════════════════

// Behavior 20: every emitted message has cap == HealCap::Starting.
#[test]
fn every_message_cap_is_starting() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let _n1 = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    let _n2 = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 5.0, 10.0);
    let _n3 = spawn_cell_at(&mut app, Vec2::new(0.0, 50.0), 5.0, 10.0);
    let _n4 = spawn_cell_at(&mut app, Vec2::new(0.0, -50.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let all = &app
        .world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0;
    assert_eq!(all.len(), 4);
    assert!(all.iter().all(|m| matches!(m.cap, HealCap::Starting)));
}

// Behavior 20 edge: cap is HealCap::Starting even at stack 5 with elevated
// amounts.
#[test]
fn cap_is_starting_even_at_stack_five() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 5.0,
        },
    );
    add_cascade_stacks(&mut app, 5);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let n = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 100.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let msgs = heals_for_cell(&app, n);
    assert_eq!(msgs.len(), 1);
    assert!((msgs[0].amount - 30.0).abs() < f32::EPSILON);
    assert!(matches!(msgs[0].cap, HealCap::Starting));
}

// Behavior 21: every emitted message has source == Some("hazard:cascade").
#[test]
fn every_message_source_is_hazard_cascade() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let n = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 5.0, 20.0);
    let va = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 0.0, 10.0);
    let vb = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 0.0, 10.0);
    send_cell_destroyed(&mut app, va, Vec2::new(50.0, 0.0));
    send_cell_destroyed(&mut app, vb, Vec2::new(-50.0, 0.0));

    run_fixed_update(&mut app);

    let all = &app
        .world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0;
    assert_eq!(all.len(), 2);
    assert!(
        all.iter()
            .all(|m| m.source.as_deref() == Some("hazard:cascade"))
    );
    // Bind n so compiler doesn't warn on unused:
    let _ = n;
}

// Behavior 22: every emitted message has healer == None.
#[test]
fn every_message_healer_is_none() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let _n1 = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    let _n2 = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 5.0, 10.0);
    let _n3 = spawn_cell_at(&mut app, Vec2::new(0.0, 50.0), 5.0, 10.0);
    let _n4 = spawn_cell_at(&mut app, Vec2::new(0.0, -50.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let all = &app
        .world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0;
    assert_eq!(all.len(), 4);
    assert!(all.iter().all(|m| m.healer.is_none()));
}

// Behavior 23: every message's target is a living cell — never victim, never
// corpse.
#[test]
fn every_target_is_a_live_cell_never_victim_or_corpse() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    let corpse = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 0.0, 10.0);
    let v = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    send_cell_destroyed(&mut app, v, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 1);
    let msgs = heals_for_cell(&app, neighbour);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].target, neighbour);
    assert!(heals_for_cell(&app, v).is_empty());
    assert!(heals_for_cell(&app, corpse).is_empty());
}

// ════════════════════════════════════════════════════════════════════════════
// Group G — Full-pipeline integration with apply_heal::<Cell>
// ════════════════════════════════════════════════════════════════════════════

/// Builder for Group G: wires `cascade_heal_on_death` before
/// `apply_heal::<Cell>` in `DeathPipelineSystems::ApplyHeal`.
fn test_app_pipeline() -> App {
    let mut app = test_app_playing();
    app.add_systems(
        FixedUpdate,
        cascade_heal_on_death.before(DeathPipelineSystems::ApplyHeal),
    );
    app.add_systems(
        FixedUpdate,
        apply_heal::<Cell>.in_set(DeathPipelineSystems::ApplyHeal),
    );
    app
}

// Behavior 24: neighbour within radius gets healed end-to-end.
#[test]
fn pipeline_neighbour_within_radius_gets_healed() {
    let mut app = test_app_pipeline();
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(neighbour).unwrap();
    assert!(
        (hp.current - 6.0).abs() < f32::EPSILON,
        "neighbour should heal 5.0 → 6.0 end-to-end, got {}",
        hp.current
    );
}

// Behavior 24 edge: at boundary distance 70.0 still heals.
#[test]
fn pipeline_neighbour_at_boundary_heals() {
    let mut app = test_app_pipeline();
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let neighbour = spawn_cell_at(&mut app, Vec2::new(70.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(neighbour).unwrap();
    assert!(
        (hp.current - 6.0).abs() < f32::EPSILON,
        "boundary neighbour (dist 70) heals, got {}",
        hp.current
    );
}

// Behavior 25: heal clamps at starting via HealCap::Starting.
#[test]
fn pipeline_heal_clamps_to_starting() {
    let mut app = test_app_pipeline();
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 0.0,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 8.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(neighbour).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "heal must clamp to starting (10.0), got {}",
        hp.current
    );
}

// Behavior 25 edge: at starting already — heal is a no-op.
#[test]
fn pipeline_heal_at_starting_is_noop() {
    let mut app = test_app_pipeline();
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 0.0,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 10.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(neighbour).unwrap();
    assert!((hp.current - 10.0).abs() < f32::EPSILON);
}

// Behavior 26: HealCap::Starting honoured even when hp.max > starting
// (Cascade + Volatility synergy).
#[test]
fn pipeline_heal_cap_is_starting_not_max_with_elevated_max() {
    let mut app = test_app_pipeline();
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 0.0,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    // Volatility has lifted hp.max to 20.0 on this neighbour. HealCap::Starting
    // still clamps at hp.starting = 10.0, NOT 20.0.
    let neighbour = spawn_cell_at_with_max(&mut app, Vec2::new(50.0, 0.0), 8.0, 10.0, Some(20.0));
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(neighbour).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "HealCap::Starting must clamp at starting (10.0), not max (20.0); got {}",
        hp.current
    );
}

// Behavior 27: multiple deaths, shared neighbour → Hp sums (no clamp needed).
#[test]
fn pipeline_multiple_deaths_shared_neighbour_sums_hp() {
    let mut app = test_app_pipeline();
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let n = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 5.0, 20.0);
    let va = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 0.0, 10.0);
    let vb = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 0.0, 10.0);
    send_cell_destroyed(&mut app, va, Vec2::new(50.0, 0.0));
    send_cell_destroyed(&mut app, vb, Vec2::new(-50.0, 0.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(n).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "two heals of 1.0 each → 5.0 + 2.0 = 7.0, got {}",
        hp.current
    );
}

// Behavior 27 edge: shared neighbour with starting = 6.0 clamps at 6.0.
#[test]
fn pipeline_multiple_deaths_clamp_at_starting() {
    let mut app = test_app_pipeline();
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let n = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 5.0, 6.0);
    let va = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 0.0, 10.0);
    let vb = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 0.0, 10.0);
    send_cell_destroyed(&mut app, va, Vec2::new(50.0, 0.0));
    send_cell_destroyed(&mut app, vb, Vec2::new(-50.0, 0.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(n).unwrap();
    assert!(
        (hp.current - 6.0).abs() < f32::EPSILON,
        "shared-neighbour heals clamp at starting = 6.0, got {}",
        hp.current
    );
}

// Behavior 28: cell outside radius — Hp unchanged end-to-end.
#[test]
fn pipeline_cell_outside_radius_unchanged() {
    let mut app = test_app_pipeline();
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let far = spawn_cell_at(&mut app, Vec2::new(200.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(far).unwrap();
    assert!((hp.current - 5.0).abs() < f32::EPSILON);
}

// Behavior 29: dead cell is not self-healed end-to-end.
#[test]
fn pipeline_dead_cell_is_not_self_healed() {
    let mut app = test_app_pipeline();
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let dead_cell = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    send_cell_destroyed(&mut app, dead_cell, Vec2::ZERO);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(dead_cell).unwrap();
    assert!(
        hp.current.abs() < f32::EPSILON,
        "dead cell must not self-heal end-to-end, got {}",
        hp.current
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Group H — Retrofit-specific regression guards
// ════════════════════════════════════════════════════════════════════════════

// Behavior 30: regression — cascade_heal_on_death does NOT mutate Hp directly.
#[test]
fn regression_cascade_does_not_mutate_hp_directly() {
    let mut app = test_app_playing();
    // ONLY cascade_heal_on_death wired — apply_heal::<Cell> is deliberately
    // absent. Any Hp change observed here is a direct mutation.
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    // 1. Message WAS emitted.
    assert_eq!(
        cascade_heal_collector_len(&app),
        1,
        "cascade must emit exactly one HealDealt<Cell> message"
    );
    // 2. Hp was NOT directly mutated — apply_heal isn't wired here.
    let hp = app.world().get::<Hp>(neighbour).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "neighbour Hp must remain 5.0 (apply_heal isn't wired); got {}. \
         A change here proves cascade_heal_on_death is directly mutating Hp.",
        hp.current
    );
}

// Behavior 32: regression — no pre-clamp in cascade. amount is raw
// heal_per_neighbour(stacks), not min'd against hp.max or hp.starting.
#[test]
fn regression_cascade_emits_raw_amount_no_pre_clamp() {
    let mut app = test_app_playing();
    // ONLY cascade wired.
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      100.0,
            per_level_heal: 0.0,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    // Neighbour has starting = 10.0, max = Some(20.0). A buggy implementation
    // that pre-clamps at min(max, starting) or similar would emit amount = 15.0
    // or 5.0. We expect the raw 100.0 — the cap is declarative.
    let neighbour = spawn_cell_at_with_max(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0, Some(20.0));
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let msgs = heals_for_cell(&app, neighbour);
    assert_eq!(msgs.len(), 1);
    assert!(
        (msgs[0].amount - 100.0).abs() < f32::EPSILON,
        "amount must be the raw heal (100.0), not pre-clamped against Hp \
         ceilings; got {}. Pre-clamp is apply_heal's job via HealCap.",
        msgs[0].amount
    );
    assert!(matches!(msgs[0].cap, HealCap::Starting));
}
