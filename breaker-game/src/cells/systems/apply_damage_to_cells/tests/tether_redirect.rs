//! Section E — Tether redirect inside `apply_damage_to_cells` (Behaviours 46–61).
//!
//! Pins the redirect branch that `apply_damage_to_cells` pushes onto
//! `TetherRedirectBuffer`, which is drained into `DamageDealt<Cell>` by
//! `emit_tether_redirects` later in the same `ApplyDamage` set.
//!
//! Test helpers (`PendingCellDamage`, `enqueue_cell_damage`) are local copies
//! from the sibling `diffusion` tests file for independence.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::super::system::apply_damage_to_cells;
use crate::{
    hazard::{
        definition::HazardKind,
        hazards::{
            diffusion::DiffusionConfig,
            tether::{TETHER_SENTINEL, TetherConfig, TetherLink, TetherRedirectBuffer},
        },
        resources::ActiveHazards,
    },
    prelude::*,
    shared::death_pipeline::sets::DeathPipelineSystems,
};

// ── Local test helpers ──────────────────────────────────────────────────────

/// Pending `DamageDealt<Cell>` messages enqueued before each tick.
#[derive(Resource, Default)]
struct PendingCellDamage(Vec<DamageDealt<Cell>>);

fn enqueue_cell_damage(
    mut pending: ResMut<PendingCellDamage>,
    mut writer: MessageWriter<DamageDealt<Cell>>,
) {
    // Drain — each message is emitted exactly once, not re-enqueued every
    // tick. Tests that span multiple ticks (for redirect round-trip) must not
    // double-apply the primary damage.
    for msg in pending.0.drain(..) {
        writer.write(msg);
    }
}

fn damage_msg_with_source(
    target: Entity,
    amount: f32,
    dealer: Option<Entity>,
    source_chip: Option<String>,
) -> DamageDealt<Cell> {
    DamageDealt {
        dealer,
        target,
        amount,
        source_chip,
        _marker: PhantomData,
    }
}

fn damage_msg(target: Entity, amount: f32, dealer: Option<Entity>) -> DamageDealt<Cell> {
    damage_msg_with_source(target, amount, dealer, None)
}

/// Persistent collector that accumulates across ticks — complements the
/// `MessageCollector` which clears every `First`. Tests that run multiple
/// ticks (for redirect round-trip) inspect this instead.
#[derive(Resource, Default)]
struct PersistentMessageLog(Vec<DamageDealt<Cell>>);

fn log_cell_damage_messages(
    mut reader: MessageReader<DamageDealt<Cell>>,
    mut log: ResMut<PersistentMessageLog>,
) {
    for msg in reader.read() {
        log.0.push(msg.clone());
    }
}

/// Canonical builder: state hierarchy at `NodeState::Playing`,
/// `ActiveHazards`, captured `DamageDealt<Cell>`, `TetherRedirectBuffer`,
/// `PendingCellDamage` + enqueue system wired before `ApplyDamage`, and
/// `apply_damage_to_cells` in that set. Tether's `emit_tether_redirects` is
/// ALSO wired (after `apply_damage_to_cells`) so buffered redirects reach the
/// message queue. A `PersistentMessageLog` accumulates across ticks so
/// multi-tick tests can assert total message counts.
fn build_apply_damage_to_cells_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message_capture::<DamageDealt<Cell>>()
        .with_resource::<PendingCellDamage>()
        .with_resource::<TetherRedirectBuffer>()
        .with_resource::<PersistentMessageLog>()
        .build();

    app.add_systems(
        FixedUpdate,
        enqueue_cell_damage.before(DeathPipelineSystems::ApplyDamage),
    );
    app.add_systems(
        FixedUpdate,
        apply_damage_to_cells.in_set(DeathPipelineSystems::ApplyDamage),
    );
    app.add_systems(
        FixedUpdate,
        drain_tether_redirect_buffer
            .in_set(DeathPipelineSystems::ApplyDamage)
            .after(apply_damage_to_cells),
    );
    app.add_systems(Last, log_cell_damage_messages);
    app
}

/// Test-local stand-in for the production `emit_tether_redirects`. Drains
/// `TetherRedirectBuffer` into `MessageWriter<DamageDealt<Cell>>`. This mirrors
/// the production system so redirect messages show up in the collector and
/// apply HP changes the same tick / next tick they would under `register`.
fn drain_tether_redirect_buffer(
    mut buffer: ResMut<TetherRedirectBuffer>,
    mut writer: MessageWriter<DamageDealt<Cell>>,
) {
    for msg in buffer.0.drain(..) {
        writer.write(msg);
    }
}

fn spawn_cell_at(app: &mut App, pos: Vec2, hp: f32) -> Entity {
    app.world_mut()
        .spawn((Cell, Position2D(pos), Hp::new(hp), KilledBy::default()))
        .id()
}

fn spawn_cell_at_dead(app: &mut App, pos: Vec2, hp: f32) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            Hp::new(hp),
            KilledBy::default(),
            Dead,
        ))
        .id()
}

fn spawn_cell_at_invulnerable(app: &mut App, pos: Vec2, hp: f32) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            Hp::new(hp),
            KilledBy::default(),
            Invulnerable,
        ))
        .id()
}

fn spawn_linked_pair(app: &mut App, pos_a: Vec2, pos_b: Vec2, hp: f32) -> (Entity, Entity) {
    let a = spawn_cell_at(app, pos_a, hp);
    let b = spawn_cell_at(app, pos_b, hp);
    app.world_mut()
        .entity_mut(a)
        .insert(TetherLink { partner: b });
    app.world_mut()
        .entity_mut(b)
        .insert(TetherLink { partner: a });
    (a, b)
}

fn install_tether_config(app: &mut App, cfg: TetherConfig) {
    app.world_mut().insert_resource(cfg);
}

fn install_diffusion_config(app: &mut App, cfg: DiffusionConfig) {
    app.world_mut().insert_resource(cfg);
}

fn add_tether_stacks(app: &mut App, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(HazardKind::Tether);
    }
}

fn add_diffusion_stacks(app: &mut App, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(HazardKind::Diffusion);
    }
}

const fn canonical_tether_config() -> TetherConfig {
    TetherConfig {
        base_damage:        25.0,
        damage_per_level:   10.0,
        base_coverage:      40.0,
        coverage_per_level: 10.0,
    }
}

fn hp_of(app: &App, entity: Entity) -> f32 {
    app.world().get::<Hp>(entity).unwrap().current
}

fn push_damage(app: &mut App, msg: DamageDealt<Cell>) {
    app.world_mut()
        .resource_mut::<PendingCellDamage>()
        .0
        .push(msg);
}

fn collected_messages(app: &App) -> Vec<DamageDealt<Cell>> {
    app.world().resource::<PersistentMessageLog>().0.clone()
}

/// Runs enough ticks to let any redirect messages roundtrip through the
/// message queue and be applied the following tick.
fn tick_n(app: &mut App, n: usize) {
    for _ in 0..n {
        tick(app);
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 46 — Stack 1: damage to A redirects 25% to B (primary takes FULL)
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn stack_one_damage_to_a_redirects_twenty_five_percent_to_b() {
    let mut app = build_apply_damage_to_cells_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(50.0, 0.0), 100.0);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);

    push_damage(
        &mut app,
        damage_msg_with_source(a, 100.0, None, Some("test_chip".to_string())),
    );
    tick_n(&mut app, 2);

    assert!(
        (hp_of(&app, a) - 0.0).abs() < f32::EPSILON,
        "A must take FULL 100 damage, got HP {}",
        hp_of(&app, a)
    );
    assert!(
        (hp_of(&app, b) - 75.0).abs() < f32::EPSILON,
        "B must take 25% redirect (25.0) → HP 75.0, got {}",
        hp_of(&app, b)
    );
}

#[test]
fn zero_amount_message_produces_no_redirect() {
    let mut app = build_apply_damage_to_cells_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(50.0, 0.0), 100.0);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(a, 0.0, None));
    tick_n(&mut app, 2);

    assert!((hp_of(&app, a) - 100.0).abs() < f32::EPSILON);
    assert!((hp_of(&app, b) - 100.0).abs() < f32::EPSILON);
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 47 — Redirect message fields match contract
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn redirect_message_fields_match_contract() {
    let mut app = build_apply_damage_to_cells_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(50.0, 0.0), 100.0);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);
    let dealer = app.world_mut().spawn_empty().id();

    push_damage(
        &mut app,
        damage_msg_with_source(a, 100.0, Some(dealer), Some("test_chip".to_string())),
    );
    tick_n(&mut app, 2);

    let collected = collected_messages(&app);

    // One original message + one redirect (+ applied to nothing else).
    let originals: Vec<_> = collected
        .iter()
        .filter(|m| m.target == a && m.source_chip.as_deref() == Some("test_chip"))
        .collect();
    let redirects: Vec<_> = collected
        .iter()
        .filter(|m| m.target == b && m.source_chip.as_deref() == Some(TETHER_SENTINEL))
        .collect();

    assert_eq!(
        originals.len(),
        1,
        "expected exactly one original DamageDealt<Cell> to A, got {}",
        originals.len()
    );
    assert!(
        (originals[0].amount - 100.0).abs() < f32::EPSILON,
        "original amount expected 100.0, got {}",
        originals[0].amount
    );

    assert_eq!(
        redirects.len(),
        1,
        "expected exactly one redirect DamageDealt<Cell> to B with sentinel \
         source_chip, got {}",
        redirects.len()
    );
    assert!(
        (redirects[0].amount - 25.0).abs() < f32::EPSILON,
        "redirect amount expected 25.0, got {}",
        redirects[0].amount
    );
    assert_eq!(
        redirects[0].dealer,
        Some(dealer),
        "redirect dealer must be the original dealer (pass-through)"
    );
    assert_eq!(
        redirects[0].source_chip.as_deref(),
        Some(TETHER_SENTINEL),
        "sentinel overwrites original source_chip"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 48 — Stack 3: 45% damage
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn stack_three_redirects_forty_five_percent() {
    let mut app = build_apply_damage_to_cells_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(50.0, 0.0), 100.0);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 3);

    push_damage(&mut app, damage_msg(a, 80.0, None));
    tick_n(&mut app, 2);

    assert!(
        (hp_of(&app, a) - 20.0).abs() < f32::EPSILON,
        "A HP expected 20.0 (took 80 full), got {}",
        hp_of(&app, a)
    );
    // 45% of 80 = 36 → B HP = 64.
    assert!(
        (hp_of(&app, b) - 64.0).abs() < f32::EPSILON,
        "B HP expected 64.0 (took 36 redirect), got {}",
        hp_of(&app, b)
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 49 — Stack 5: 65% damage
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn stack_five_redirects_sixty_five_percent() {
    let mut app = build_apply_damage_to_cells_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(50.0, 0.0), 200.0);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 5);

    push_damage(&mut app, damage_msg(a, 100.0, None));
    tick_n(&mut app, 2);

    assert!(
        (hp_of(&app, a) - 100.0).abs() < f32::EPSILON,
        "A HP expected 100.0, got {}",
        hp_of(&app, a)
    );
    // 65% of 100 = 65 → B HP = 200 - 65 = 135.
    assert!(
        (hp_of(&app, b) - 135.0).abs() < f32::EPSILON,
        "B HP expected 135.0, got {}",
        hp_of(&app, b)
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 50 — Stack 9: 105% (uncapped)
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn stack_nine_redirects_one_hundred_five_percent_uncapped() {
    let mut app = build_apply_damage_to_cells_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(50.0, 0.0), 200.0);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 9);

    push_damage(&mut app, damage_msg(a, 100.0, None));
    tick_n(&mut app, 2);

    assert!(
        (hp_of(&app, a) - 100.0).abs() < f32::EPSILON,
        "A HP expected 100.0, got {}",
        hp_of(&app, a)
    );
    // 105% of 100 = 105 → B HP = 200 - 105 = 95.
    assert!(
        (hp_of(&app, b) - 95.0).abs() < f32::EPSILON,
        "B HP expected 95.0 (105% uncapped), got {}",
        hp_of(&app, b)
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 51 — No redirect when target has no TetherLink
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn no_redirect_when_target_has_no_tether_link() {
    let mut app = build_apply_damage_to_cells_app();
    let a = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(a, 50.0, None));
    tick_n(&mut app, 2);

    assert!((hp_of(&app, a) - 50.0).abs() < f32::EPSILON);

    let collected = collected_messages(&app);
    assert_eq!(
        collected.len(),
        1,
        "only the original message should be in the collector, got {}",
        collected.len()
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 52 — Redirect does NOT re-emit on sentinel-tagged incoming message
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn sentinel_tagged_message_does_not_re_emit_redirect() {
    let mut app = build_apply_damage_to_cells_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(50.0, 0.0), 100.0);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);

    // Pushed message carries the sentinel — must be treated as already-redirected.
    push_damage(
        &mut app,
        damage_msg_with_source(b, 25.0, None, Some(TETHER_SENTINEL.to_string())),
    );
    tick_n(&mut app, 2);

    assert!(
        (hp_of(&app, b) - 75.0).abs() < f32::EPSILON,
        "B takes the 25 primary damage, got HP {}",
        hp_of(&app, b)
    );
    assert!(
        (hp_of(&app, a) - 100.0).abs() < f32::EPSILON,
        "A must NOT be damaged — sentinel on incoming skips redirect. Got HP {}",
        hp_of(&app, a)
    );

    let collected = collected_messages(&app);
    let redirects_to_a: Vec<_> = collected
        .iter()
        .filter(|m| m.target == a && m.source_chip.as_deref() == Some(TETHER_SENTINEL))
        .collect();
    assert_eq!(
        redirects_to_a.len(),
        0,
        "no new redirect from the sentinel-tagged message, got {}",
        redirects_to_a.len()
    );
}

#[test]
fn other_source_chip_still_produces_redirect() {
    let mut app = build_apply_damage_to_cells_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(50.0, 0.0), 100.0);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);

    push_damage(
        &mut app,
        damage_msg_with_source(b, 25.0, None, Some("other_chip".to_string())),
    );
    tick_n(&mut app, 2);

    // 25% of 25 = 6.25 → A loses 6.25 HP.
    assert!(
        (hp_of(&app, a) - 93.75).abs() < f32::EPSILON,
        "A should be damaged by 6.25 (non-sentinel source_chip), got HP {}",
        hp_of(&app, a)
    );
}

#[test]
fn none_source_chip_still_produces_redirect() {
    let mut app = build_apply_damage_to_cells_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(50.0, 0.0), 100.0);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(b, 25.0, None));
    tick_n(&mut app, 2);

    assert!(
        (hp_of(&app, a) - 93.75).abs() < f32::EPSILON,
        "A should take 25% of 25 = 6.25 damage on None source_chip, got HP {}",
        hp_of(&app, a)
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 53 — Bidirectional: damage to B redirects to A
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn damage_to_b_redirects_to_a() {
    let mut app = build_apply_damage_to_cells_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(50.0, 0.0), 100.0);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(b, 100.0, None));
    tick_n(&mut app, 2);

    assert!(
        (hp_of(&app, b) - 0.0).abs() < f32::EPSILON,
        "B HP expected 0.0, got {}",
        hp_of(&app, b)
    );
    assert!(
        (hp_of(&app, a) - 75.0).abs() < f32::EPSILON,
        "A HP expected 75.0 (25% redirect), got {}",
        hp_of(&app, a)
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 54 — Same-tick damage to both linked cells → no infinite loop
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn same_tick_damage_to_both_partners_bounded_message_count() {
    let mut app = build_apply_damage_to_cells_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(50.0, 0.0), 100.0);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(a, 100.0, None));
    push_damage(&mut app, damage_msg(b, 40.0, None));
    tick_n(&mut app, 2);

    // A gets 100 damage (→ dead, HP <= 0).
    assert!(
        hp_of(&app, a) <= 0.0,
        "A HP must be ≤ 0.0 after taking 100 damage, got {}",
        hp_of(&app, a)
    );
    // B takes 40 primary + 25 redirect from A (25% of 100) = 65 → HP 35.
    assert!(
        (hp_of(&app, b) - 35.0).abs() < f32::EPSILON,
        "B HP expected 35.0 (40 primary + 25 redirect from A), got {}",
        hp_of(&app, b)
    );

    let collected = collected_messages(&app);
    assert_eq!(
        collected.len(),
        4,
        "message count must be exactly 4 (2 primary + 2 redirects, no chain); got {}",
        collected.len()
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 55 — Harness-safe when TetherConfig absent
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn apply_damage_does_not_panic_when_tether_config_absent() {
    let mut app = build_apply_damage_to_cells_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(50.0, 0.0), 100.0);
    // NO TetherConfig installed.
    add_tether_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(a, 50.0, None));
    tick_n(&mut app, 2);

    assert!(
        (hp_of(&app, a) - 50.0).abs() < f32::EPSILON,
        "A must take FULL 50 damage, got {}",
        hp_of(&app, a)
    );
    assert!(
        (hp_of(&app, b) - 100.0).abs() < f32::EPSILON,
        "B must be untouched (no TetherConfig → no redirect), got {}",
        hp_of(&app, b)
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 56 — Zero Tether stacks + config present → short-circuit
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn zero_stacks_with_config_short_circuits_redirect() {
    let mut app = build_apply_damage_to_cells_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(50.0, 0.0), 100.0);
    install_tether_config(&mut app, canonical_tether_config());
    // No Tether stacks added.

    push_damage(&mut app, damage_msg(a, 50.0, None));
    tick_n(&mut app, 2);

    assert!(
        (hp_of(&app, a) - 50.0).abs() < f32::EPSILON,
        "A HP expected 50.0, got {}",
        hp_of(&app, a)
    );
    assert!(
        (hp_of(&app, b) - 100.0).abs() < f32::EPSILON,
        "B HP expected 100.0 (stacks == 0 short-circuit), got {}",
        hp_of(&app, b)
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 57 — Missing partner → redirect silently skipped
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn missing_partner_redirect_silently_skipped() {
    let mut app = build_apply_damage_to_cells_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(50.0, 0.0), 100.0);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);

    // Despawn B before the tick — A's TetherLink now points at a missing
    // entity, and cleanup has not run yet.
    app.world_mut().despawn(b);

    push_damage(&mut app, damage_msg(a, 100.0, None));
    tick_n(&mut app, 2);

    assert!(
        (hp_of(&app, a) - 0.0).abs() < f32::EPSILON,
        "A must take FULL 100 damage, got {}",
        hp_of(&app, a)
    );
    // No panic above AND no redirect message.
    let collected = collected_messages(&app);
    let redirects: Vec<_> = collected
        .iter()
        .filter(|m| m.source_chip.as_deref() == Some(TETHER_SENTINEL))
        .collect();
    assert_eq!(
        redirects.len(),
        0,
        "missing partner → no redirect message emitted; got {} redirects",
        redirects.len()
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 58 — Tether + Diffusion ordering: Diffusion first, Tether reads reduced
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn tether_redirect_uses_diffusion_reduced_primary_damage() {
    let mut app = build_apply_damage_to_cells_app();
    // Tether pair A-B. A also has a diffusion neighbour C (no TetherLink).
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(50.0, 0.0), 100.0);
    let c = spawn_cell_at(&mut app, Vec2::new(0.0, 50.0), 100.0);

    install_tether_config(&mut app, canonical_tether_config());
    install_diffusion_config(
        &mut app,
        DiffusionConfig {
            base_share_percent:      20.0,
            share_per_level_percent: 10.0,
            depth_increase_interval: 5,
        },
    );
    add_tether_stacks(&mut app, 1);
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(a, 50.0, None));
    tick_n(&mut app, 2);

    // Tick N (primary msg to A):
    //   Diffusion reduces A's primary to 50 * 0.80 = 40 → A HP 60.
    //   Ring-1 of A = {B, C}: each takes 50 * 0.20 / 2 = 5 → B HP 95, C HP 95.
    //   Tether reads Diffusion-REDUCED primary damage 40:
    //     redirect msg queued = DamageDealt(B, 40 * 0.25 = 10, sentinel).
    // Tick N+1 (redirect msg to B with sentinel):
    //   Sentinel skips Tether re-emission, but the damage still goes through
    //   Diffusion BFS (normal message semantics).
    //   Ring-1 of B in this topology: only A is within ADJACENCY_RADIUS_SQ=4900
    //   (C is at dist_sq=5000, outside). So ring1_len=1.
    //   B primary damage = 10 * 0.80 = 8 → B HP 87.
    //   A ring damage = 10 * 0.20 / 1 = 2 → A HP 58.
    // Final: A=58, B=87, C=95.
    assert!(
        (hp_of(&app, a) - 58.0).abs() < f32::EPSILON,
        "A HP expected 58.0 (60 - 2 ring damage from tether redirect's diffusion), got {}",
        hp_of(&app, a)
    );
    assert!(
        (hp_of(&app, b) - 87.0).abs() < f32::EPSILON,
        "B HP expected 87.0 (95 - 8 primary damage from tether redirect's diffusion), got {}",
        hp_of(&app, b)
    );
    assert!(
        (hp_of(&app, c) - 95.0).abs() < f32::EPSILON,
        "C HP expected 95.0 (5 diffusion ring only, no tether, no ring from redirect), got {}",
        hp_of(&app, c)
    );
}

#[test]
fn tether_only_variant_uses_original_amount() {
    // No diffusion → Tether reads msg.amount unchanged.
    let mut app = build_apply_damage_to_cells_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(50.0, 0.0), 100.0);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(a, 50.0, None));
    tick_n(&mut app, 2);

    assert!(
        (hp_of(&app, a) - 50.0).abs() < f32::EPSILON,
        "A takes full 50 damage (no diffusion)"
    );
    // 25% of 50 = 12.5 → B HP 87.5.
    assert!(
        (hp_of(&app, b) - 87.5).abs() < f32::EPSILON,
        "B HP expected 87.5 (25% of 50), got {}",
        hp_of(&app, b)
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 59 — Diffusion ring damage does NOT trigger Tether redirect
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn diffusion_ring_damage_does_not_trigger_tether_redirect() {
    let mut app = build_apply_damage_to_cells_app();
    // Two tether pairs: A-B adjacent to C-D vertically.
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(50.0, 0.0), 100.0);
    let (c, d) = spawn_linked_pair(&mut app, Vec2::new(0.0, 50.0), Vec2::new(0.0, 100.0), 100.0);

    install_tether_config(&mut app, canonical_tether_config());
    install_diffusion_config(
        &mut app,
        DiffusionConfig {
            base_share_percent:      20.0,
            share_per_level_percent: 10.0,
            depth_increase_interval: 5,
        },
    );
    add_tether_stacks(&mut app, 1);
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(a, 50.0, None));
    tick_n(&mut app, 2);

    // A's Tether redirect to B fires (uses reduced primary 40 → 10 damage).
    // A's diffusion ring hits C in-memory — it does NOT emit a DamageDealt<Cell>.
    // → C's redirect to D does NOT fire.
    assert!(
        (hp_of(&app, d) - 100.0).abs() < f32::EPSILON,
        "D HP must be 100.0 — diffusion ring on C must NOT trigger a redirect to D, got {}",
        hp_of(&app, d)
    );

    // sanity on upstream values
    let _ = (a, b, c);
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 60 — Invulnerable primary → no redirect
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn invulnerable_primary_produces_no_redirect() {
    let mut app = build_apply_damage_to_cells_app();
    let a = spawn_cell_at_invulnerable(&mut app, Vec2::ZERO, 100.0);
    let b = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    // Mutual TetherLink, A is Invulnerable.
    app.world_mut()
        .entity_mut(a)
        .insert(TetherLink { partner: b });
    app.world_mut()
        .entity_mut(b)
        .insert(TetherLink { partner: a });
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(a, 50.0, None));
    tick_n(&mut app, 2);

    // A's HP unchanged (invulnerable filtered out of primary query).
    assert!(
        (hp_of(&app, a) - 100.0).abs() < f32::EPSILON,
        "A HP expected 100.0 (Invulnerable), got {}",
        hp_of(&app, a)
    );
    assert!(
        (hp_of(&app, b) - 100.0).abs() < f32::EPSILON,
        "B HP expected 100.0 (no redirect from invulnerable primary), got {}",
        hp_of(&app, b)
    );

    let collected = collected_messages(&app);
    assert_eq!(
        collected
            .iter()
            .filter(|m| m.source_chip.as_deref() == Some(TETHER_SENTINEL))
            .count(),
        0,
        "invulnerable primary must not produce redirect"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 61 — Dead primary → no redirect
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn dead_primary_produces_no_redirect() {
    let mut app = build_apply_damage_to_cells_app();
    let a = spawn_cell_at_dead(&mut app, Vec2::ZERO, 100.0);
    let b = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    app.world_mut()
        .entity_mut(a)
        .insert(TetherLink { partner: b });
    app.world_mut()
        .entity_mut(b)
        .insert(TetherLink { partner: a });
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(a, 50.0, None));
    tick_n(&mut app, 2);

    assert!(
        (hp_of(&app, a) - 100.0).abs() < f32::EPSILON,
        "A HP expected 100.0 (Dead primary filtered), got {}",
        hp_of(&app, a)
    );
    assert!(
        (hp_of(&app, b) - 100.0).abs() < f32::EPSILON,
        "B HP expected 100.0 (no redirect from Dead primary), got {}",
        hp_of(&app, b)
    );

    let collected = collected_messages(&app);
    assert_eq!(
        collected
            .iter()
            .filter(|m| m.source_chip.as_deref() == Some(TETHER_SENTINEL))
            .count(),
        0,
        "Dead primary must not produce redirect"
    );
}
