//! Scheduling tests for `iron_curtain_on_bolt_lost`.
//!
//! Production guarantee: `iron_curtain_on_bolt_lost` is tagged
//! `.after(BoltSystems::BoltLost).before(EffectV3Systems::Bridge)`, and
//! `EffectV3Plugin` configures the transitive chain
//! `Bridge → Tick → DmgSystems::EmitDamage`. These tests pin the functional
//! consequence: a single `tick(...)` applies a target cell's
//! `VulnerableStack` multiplier to the Iron Curtain wave damage in the
//! same tick the wave fires. Iron Curtain emits with `dealer: None`, so
//! `DamageBoostStack` on the bolt is NOT applied — this exercises the
//! TARGET-side multiplier pathway same-tick.
//!
//! Iron Curtain fires on `BoltLost`, not on cell collision, so these tests
//! do not pair with a `bolt_cell_collision` emission — no pre-W6
//! double-apply contribution is involved here.

use bevy::{ecs::message::Messages, prelude::*};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

use super::{
    super::system::{IronCurtainConfig, register},
    helpers::{amount_for_target, collected_iron_curtain_damage},
};
use crate::{
    bolt::components::BoltBaseDamage,
    chips::definition::Rarity,
    mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
        resources::ActiveProtocols,
    },
    prelude::*,
    shared::GameDrawLayer,
};

/// Builder-format `SourceId` used as the canonical opaque tag for the
/// `VulnerableStack` augmentation in scheduling tests.
fn test_source() -> SourceId {
    SourceId::chip("Test").rarity(Rarity::Common).build()
}

fn iron_curtain_scheduling_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_protocol_scaffolding()
        .insert_resource(IronCurtainConfig {
            damage_fraction: 0.5,
            falloff_start:   1000.0,
        })
        .build();
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(ProtocolDefinition {
            name:        "IronCurtain".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::IronCurtain {
                damage_fraction: 0.5,
                falloff_start:   1000.0,
            },
        });
    register(&mut app);
    app
}

fn spawn_breaker(app: &mut App, x: f32, y: f32) -> Entity {
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((Breaker, Position2D(pos), GlobalPosition2D(pos), Spatial2D))
        .id()
}

fn spawn_bolt_entity(app: &mut App, x: f32, y: f32, base_damage: f32) -> Entity {
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Bolt,
            BoltBaseDamage(base_damage),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
        ))
        .id()
}

fn spawn_cell_with_hp(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Cell,
            Hp::new(hp),
            KilledBy { killer: None },
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            GameDrawLayer::Cell,
        ))
        .id()
}

fn spawn_cell_with_hp_and_vuln(app: &mut App, x: f32, y: f32, hp: f32, vuln: f32) -> Entity {
    let entity = spawn_cell_with_hp(app, x, y, hp);
    let mut stack = VulnerableStack::default();
    stack.add(test_source(), vuln);
    app.world_mut().entity_mut(entity).insert(stack);
    entity
}

fn write_bolt_lost(app: &mut App, bolt: Entity, breaker: Entity) {
    app.world_mut()
        .resource_mut::<Messages<BoltLost>>()
        .write(BoltLost { bolt, breaker });
}

#[test]
fn iron_curtain_on_bolt_lost_applies_vulnerable_stack_in_same_tick() {
    let mut app = iron_curtain_scheduling_app();
    let breaker = spawn_breaker(&mut app, 0.0, -100.0);
    let bolt = spawn_bolt_entity(&mut app, 0.0, -500.0, 10.0);
    let cell = spawn_cell_with_hp_and_vuln(&mut app, 0.0, 0.0, 100.0, 2.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let hp = app
        .world()
        .get::<Hp>(cell)
        .expect("cell should still have Hp")
        .current;
    // Wave origin damage: base × damage_fraction = 10.0 × 0.5 = 5.0.
    // Vulnerability 2.0 multiplies: 5.0 × 2.0 = 10.0.
    // final_hp = 100.0 − 10.0 == 90.0.
    assert!(
        (hp - 90.0).abs() < 1e-5,
        "final_hp = 100.0 − (10.0 × 0.5 × 2.0) == 90.0, got {hp}"
    );
}

#[test]
fn iron_curtain_on_bolt_lost_without_vuln_uses_identity() {
    let mut app = iron_curtain_scheduling_app();
    let breaker = spawn_breaker(&mut app, 0.0, -100.0);
    let bolt = spawn_bolt_entity(&mut app, 0.0, -500.0, 10.0);
    let cell = spawn_cell_with_hp(&mut app, 0.0, 0.0, 100.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let hp = app
        .world()
        .get::<Hp>(cell)
        .expect("cell should still have Hp")
        .current;
    // No vulnerability: final_hp = 100.0 − (10.0 × 0.5) == 95.0.
    assert!(
        (hp - 95.0).abs() < 1e-5,
        "final_hp = 100.0 − 5.0 == 95.0 (no VulnerableStack), got {hp}"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// W7 — drop `Without<Invulnerable>` from `LiveCellQuery`
//
// Pins the end-to-end contract: Iron Curtain now emits a DamageDealt<Cell>
// for invulnerable cells, and `invulnerable_filter::<Cell>` (a MessageMutator
// in DmgSystems::ApplyDamage from `rantzsoft_dmg`) zeroes the `amount` field
// before `apply_damage::<Cell>` runs. Invulnerable cells' HP is unchanged;
// messages post-pipeline carry `amount == 0.0`. See
// `.claude/specs/w7-drop-invulnerable-filter-tests.md` for the spec.
//
// MessageCollector<DamageDealt<Cell>> captures in Bevy's `Last` schedule
// (after FixedUpdate), so it observes post-pipeline amounts.
// ════════════════════════════════════════════════════════════════════════════

fn spawn_cell_invulnerable(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
    let entity = spawn_cell_with_hp(app, x, y, hp);
    app.world_mut().entity_mut(entity).insert(Invulnerable);
    entity
}

// ── W7 Behavior 1 — invulnerable cell receives a message with post-pipeline
//    amount == 0.0 and HP is unchanged.

#[test]
fn invulnerable_cell_receives_iron_curtain_message_with_post_pipeline_zero_amount() {
    let mut app = iron_curtain_scheduling_app();
    attach_message_capture::<DamageDealt<Cell>>(&mut app);

    let breaker = spawn_breaker(&mut app, 0.0, -100.0);
    let bolt = spawn_bolt_entity(&mut app, 0.0, -500.0, 10.0);
    let cell = spawn_cell_invulnerable(&mut app, 0.0, 0.0, 100.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    // HP on invulnerable cell must be unchanged — pipeline zeroes amount
    // before `apply_damage::<Cell>` writes HP.
    let hp = app
        .world()
        .get::<Hp>(cell)
        .expect("cell should still have Hp")
        .current;
    assert!(
        (hp - 100.0).abs() < 1e-5,
        "invulnerable cell Hp must stay at 100.0, got {hp}"
    );

    // One Iron-Curtain-sourced message observed, targeting the invulnerable
    // cell, with post-pipeline amount == 0.0 and dealer == None.
    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(
        msgs.len(),
        1,
        "exactly one Iron Curtain message expected for the invulnerable cell, got {}",
        msgs.len()
    );
    let msg = &msgs[0];
    assert_eq!(
        msg.target, cell,
        "message target must be the invulnerable cell"
    );
    assert_eq!(
        msg.source.as_ref(),
        Some(&SourceId::protocol(ProtocolKind::IronCurtain).build())
    );
    assert!(
        msg.amount.abs() < f32::EPSILON,
        "invulnerable_filter::<Cell> zeroes amount to 0.0, got {}",
        msg.amount
    );
    assert!(
        msg.dealer.is_none(),
        "iron curtain sets dealer: None; pipeline must not synthesize one, got {:?}",
        msg.dealer
    );
}

// ── W7 Behavior 1 edge case — cell with BOTH Dead and Invulnerable is excluded
//    by the remaining `Without<Dead>` filter. No Iron-Curtain message.

#[test]
fn dead_and_invulnerable_cell_still_excluded_by_dead_filter() {
    let mut app = iron_curtain_scheduling_app();
    attach_message_capture::<DamageDealt<Cell>>(&mut app);

    let breaker = spawn_breaker(&mut app, 0.0, -100.0);
    let bolt = spawn_bolt_entity(&mut app, 0.0, -500.0, 10.0);
    let cell = spawn_cell_with_hp(&mut app, 0.0, 0.0, 100.0);
    app.world_mut().entity_mut(cell).insert(Dead);
    app.world_mut().entity_mut(cell).insert(Invulnerable);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "Dead cell must be excluded by the remaining Without<Dead> filter"
    );
}

// ── W7 Behavior 2 — fully vulnerable cell still takes non-zero damage.

#[test]
fn vulnerable_cell_still_takes_non_zero_damage_post_pipeline() {
    let mut app = iron_curtain_scheduling_app();
    attach_message_capture::<DamageDealt<Cell>>(&mut app);

    let breaker = spawn_breaker(&mut app, 0.0, -100.0);
    let bolt = spawn_bolt_entity(&mut app, 0.0, -500.0, 10.0);
    let cell = spawn_cell_with_hp(&mut app, 0.0, 0.0, 100.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    // base_damage × damage_fraction = 10.0 × 0.5 = 5.0; distance 100.0 ≤
    // falloff_start 1000.0 → full-damage zone. 100.0 − 5.0 == 95.0.
    let hp = app
        .world()
        .get::<Hp>(cell)
        .expect("cell should still have Hp")
        .current;
    assert!(
        (hp - 95.0).abs() < 1e-5,
        "vulnerable cell Hp expected 95.0, got {hp}"
    );

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(
        msgs.len(),
        1,
        "exactly one Iron Curtain message expected for the vulnerable cell"
    );
    let msg = &msgs[0];
    assert_eq!(msg.target, cell);
    assert!(
        (msg.amount - 5.0).abs() < 1e-4,
        "vulnerable cell post-pipeline amount expected 5.0 (pass-through), got {}",
        msg.amount
    );
    assert!(
        msg.dealer.is_none(),
        "iron curtain dealer field is None; pipeline does not synthesize a dealer"
    );
}

// ── W7 Behavior 3 — mixed wave: one invulnerable + one vulnerable cell in a
//    single tick. Both messages emitted; pipeline zeroes only the invulnerable.

#[test]
fn mixed_wave_invulnerable_and_vulnerable_amounts_differ_post_pipeline() {
    let mut app = iron_curtain_scheduling_app();
    attach_message_capture::<DamageDealt<Cell>>(&mut app);

    let breaker = spawn_breaker(&mut app, 0.0, -100.0);
    let bolt = spawn_bolt_entity(&mut app, 0.0, -500.0, 10.0);
    let cell_invuln = spawn_cell_invulnerable(&mut app, 0.0, 0.0, 100.0);
    let cell_vuln = spawn_cell_with_hp(&mut app, 20.0, 0.0, 100.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(
        msgs.len(),
        2,
        "two Iron Curtain messages expected (one per cell), got {}",
        msgs.len()
    );

    assert_eq!(
        amount_for_target(&msgs, cell_invuln),
        Some(0.0),
        "invulnerable cell's post-pipeline amount must be 0.0"
    );

    let vuln_amount =
        amount_for_target(&msgs, cell_vuln).expect("vulnerable cell must have a message");
    assert!(
        (vuln_amount - 5.0).abs() < 1e-4,
        "vulnerable cell's post-pipeline amount expected 5.0 (pass-through), got {vuln_amount}"
    );

    let hp_invuln = app
        .world()
        .get::<Hp>(cell_invuln)
        .expect("cell should still have Hp")
        .current;
    assert!(
        (hp_invuln - 100.0).abs() < 1e-5,
        "invulnerable cell Hp unchanged at 100.0, got {hp_invuln}"
    );
    let hp_vuln = app
        .world()
        .get::<Hp>(cell_vuln)
        .expect("cell should still have Hp")
        .current;
    assert!(
        (hp_vuln - 95.0).abs() < 1e-5,
        "vulnerable cell Hp expected 95.0, got {hp_vuln}"
    );
}

// ── W7 Behavior 3 edge case — reverse spawn order. Pipeline filter must be
//    order-independent at the end-to-end level.

#[test]
fn mixed_wave_is_order_independent_under_reversed_spawn() {
    let mut app = iron_curtain_scheduling_app();
    attach_message_capture::<DamageDealt<Cell>>(&mut app);

    let breaker = spawn_breaker(&mut app, 0.0, -100.0);
    let bolt = spawn_bolt_entity(&mut app, 0.0, -500.0, 10.0);
    // Reversed: vulnerable spawned FIRST, invulnerable second.
    let cell_vuln = spawn_cell_with_hp(&mut app, 20.0, 0.0, 100.0);
    let cell_invuln = spawn_cell_invulnerable(&mut app, 0.0, 0.0, 100.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(
        msgs.len(),
        2,
        "two Iron Curtain messages expected regardless of spawn order, got {}",
        msgs.len()
    );
    assert_eq!(
        amount_for_target(&msgs, cell_invuln),
        Some(0.0),
        "invulnerable cell's post-pipeline amount must be 0.0 regardless of spawn order"
    );
    let vuln_amount =
        amount_for_target(&msgs, cell_vuln).expect("vulnerable cell must have a message");
    assert!(
        (vuln_amount - 5.0).abs() < 1e-4,
        "vulnerable cell's post-pipeline amount expected 5.0, got {vuln_amount}"
    );
}
