//! W7 — `tether_emit_partner` against invulnerable partners / sources.
//!
//! Unit-level counterparts to `tether_emit_partner_skips_when_amount_zero`
//! drove the `amount <= 0.0` guard with a hand-written zero-amount message.
//! These tests drive the zeroing END-TO-END through the real
//! `invulnerable_filter::<Cell>` pipeline stage in `DmgSystems::ApplyDamage`.
//!
//! Assertions are W7-specific (message count + message amount). HP-unchanged
//! outcomes for invulnerable cells are already pinned by `rantzsoft_dmg`'s
//! `invulnerable_filter_zeroes_damage_within_apply_damage_set` crate test —
//! duplicating them here would just rehash crate-proven behavior.
//!
//! Behaviors 5 and 6 from `.claude/specs/w7-drop-invulnerable-filter-tests.md`.
//!
//! IMPORTANT — frame delay: partner sibling messages are written in
//! `DmgSystems::PostApplyDamage` on tick N, but `invulnerable_filter::<Cell>`
//! sits in `DmgSystems::ApplyDamage` which already ran on tick N. So the
//! pipeline applies the sibling on tick N+1. Behavior 5 runs TWO ticks.
//! Behavior 6 runs ONE tick (the primary is zeroed on tick N; the
//! `amount <= 0.0` guard in `tether_emit_partner` prevents any sibling from
//! being emitted, so no second tick is needed).

use std::marker::PhantomData;

use bevy::prelude::*;

use super::{emit_partner::build_tether_app, helpers::spawn_linked_pair};
use crate::prelude::*;

fn drain_damage_messages(app: &mut App) -> Vec<DamageDealt<Cell>> {
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect()
}

fn tether_sourced(msgs: &[DamageDealt<Cell>]) -> Vec<DamageDealt<Cell>> {
    msgs.iter()
        .filter(|m| m.source == Some(SourceId::from("hazard:tether")))
        .cloned()
        .collect()
}

// ════════════════════════════════════════════════════════════════════════════
// W7 Behavior 5 — ripple TO invulnerable partner: sibling IS emitted, and its
// amount is zeroed by the pipeline on the NEXT tick.
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn tether_ripple_to_invulnerable_partner_is_zeroed_by_pipeline() {
    // canonical config — damage_percent(1) == 25.0.
    let mut app = build_tether_app(true);
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(30.0, 0.0));
    // Insert Invulnerable on `b` AFTER linking — `spawn_linked_pair` already
    // wired mutual TetherLinks. The establish-side contract is orthogonal:
    // here we only test the ripple-side zeroing.
    app.world_mut().entity_mut(b).insert(Invulnerable);
    let bolt = app.world_mut().spawn_empty().id();

    // Tick 1: write primary targeting vulnerable `a`. The primary applies
    // (a.Hp 100 → 0), `tether_emit_partner` sees the post-apply primary with
    // amount 100 (not zeroed — `a` is vulnerable), and emits a sibling
    // targeting `b` with amount = 100.0 * 25.0 / 100.0 = 25.0.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        a,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app); // N — primary applied, sibling emitted
    tick(&mut app); // N+1 — sibling traverses pipeline, invulnerable_filter zeroes it

    // Drain; filter tether-sourced messages targeting `b`.
    let msgs = drain_damage_messages(&mut app);
    let tether_to_b: Vec<_> = msgs
        .iter()
        .filter(|m| m.source == Some(SourceId::from("hazard:tether")) && m.target == b)
        .collect();
    assert_eq!(
        tether_to_b.len(),
        1,
        "exactly one tether sibling targeting `b` expected, got {}",
        tether_to_b.len()
    );
    assert!(
        tether_to_b[0].amount.abs() < f32::EPSILON,
        "sibling amount must be zeroed by invulnerable_filter::<Cell> post-pipeline, got {}",
        tether_to_b[0].amount
    );
}

// ── W7 Behavior 5 edge case — smaller primary amount. Sibling at 12.5 at emit
//    time, still zeroed to 0.0 post-pipeline.

#[test]
fn tether_ripple_to_invulnerable_partner_zeroing_independent_of_emit_amount() {
    let mut app = build_tether_app(true);
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(30.0, 0.0));
    app.world_mut().entity_mut(b).insert(Invulnerable);
    let bolt = app.world_mut().spawn_empty().id();

    // Primary amount 50.0 → sibling amount 50.0 * 25.0/100.0 = 12.5 at emit.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        a,
            amount:        50.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);
    tick(&mut app);

    let msgs = drain_damage_messages(&mut app);
    let tether_to_b: Vec<_> = msgs
        .iter()
        .filter(|m| m.source == Some(SourceId::from("hazard:tether")) && m.target == b)
        .collect();
    assert_eq!(tether_to_b.len(), 1);
    assert!(
        tether_to_b[0].amount.abs() < f32::EPSILON,
        "sibling amount zero post-pipeline regardless of emit-time amount, got {}",
        tether_to_b[0].amount
    );
}

// ════════════════════════════════════════════════════════════════════════════
// W7 Behavior 6 — ripple FROM invulnerable source: post-zeroed primary makes
// `tether_emit_partner`'s `amount <= 0.0` guard skip; no sibling emitted.
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn tether_ripple_from_invulnerable_source_skips_emission_end_to_end() {
    let mut app = build_tether_app(true);
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(30.0, 0.0));
    // Make `a` invulnerable; `b` stays vulnerable.
    app.world_mut().entity_mut(a).insert(Invulnerable);
    let _ = b;
    let bolt = app.world_mut().spawn_empty().id();

    // Primary at 100.0 targeting invulnerable `a`. `invulnerable_filter::<Cell>`
    // in `DmgSystems::ApplyDamage` zeroes the primary's amount before
    // `apply_damage::<Cell>`. `tether_emit_partner` in `PostApplyDamage` then
    // reads the zeroed primary; its `amount <= 0.0` guard skips emission.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        a,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app); // single tick: primary zeroed, sibling guard fires

    let msgs = drain_damage_messages(&mut app);
    let tether_msgs = tether_sourced(&msgs);
    assert!(
        tether_msgs.is_empty(),
        "no tether-sourced sibling expected from invulnerable source, got {}",
        tether_msgs.len()
    );
}

// ── W7 Behavior 6 edge case — BOTH cells invulnerable. Same outcome: primary
//    zeroed, no ripple, zero tether-sourced messages.

#[test]
fn tether_ripple_skips_when_both_endpoints_invulnerable() {
    let mut app = build_tether_app(true);
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(30.0, 0.0));
    app.world_mut().entity_mut(a).insert(Invulnerable);
    app.world_mut().entity_mut(b).insert(Invulnerable);
    let bolt = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        a,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    let msgs = drain_damage_messages(&mut app);
    let tether_msgs = tether_sourced(&msgs);
    assert!(
        tether_msgs.is_empty(),
        "no tether-sourced sibling when both endpoints are invulnerable"
    );
}
