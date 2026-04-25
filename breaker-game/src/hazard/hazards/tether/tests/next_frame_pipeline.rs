//! W2 Behavior 34 (supplemental): the partner sibling message emitted by
//! `tether_emit_partner` at Frame N traverses the FULL damage pipeline on
//! Frame N+1 — it is NOT a shortcut. This pins the partner-side observable
//! so future refactors can't accidentally bypass `apply_vulnerable` or
//! `apply_damage_boosts` for tether-emitted messages.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::{
    super::system::tether_emit_partner,
    helpers::{
        add_tether_stacks, canonical_tether_config, install_tether_config, spawn_linked_pair,
    },
};
use crate::{
    chips::definition::Rarity,
    hazard::{
        definition::HazardKind,
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
};

/// Builder-format `SourceId` used as the canonical opaque tag for the
/// `VulnerableStack` augmentation in this test.
fn test_vuln_source() -> SourceId {
    SourceId::chip("TestVuln").rarity(Rarity::Common).build()
}

#[track_caller]
fn assert_f32_eq(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-3,
        "expected {expected}, got {actual}"
    );
}

/// Build a Tether test app wired for the `PostApplyDamage` emission system.
fn build_tether_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_resource::<ActiveHazards>()
        .build();
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);
    app.add_systems(
        FixedUpdate,
        tether_emit_partner
            .in_set(DmgSystems::PostApplyDamage)
            .run_if(hazard_active(HazardKind::Tether))
            .run_if(in_state(NodeState::Playing)),
    );
    app
}

// ── W2 Behavior 34 (supplemental): partner message runs apply_vulnerable
// on Frame N+1 ──

#[test]
fn tether_partner_sibling_runs_through_vulnerable_on_next_frame() {
    // Given:
    //   - Tether active, canonical config (damage_percent(1) = 25.0).
    //   - Cells A and B linked via TetherLink.
    //   - B has a VulnerableStack with one persistent 1.5× multiplier.
    //   - B starts at 100 HP.
    //   - Primary damage of 100.0 hits A at Frame N (dealer = bolt).
    //
    // When:
    //   - Frame N: primary applies to A (A loses 100 HP → A.Hp.current = 0.0),
    //     tether_emit_partner writes a sibling DamageDealt<Cell> to B in
    //     PostApplyDamage with amount = 25.0 (25% of primary amount).
    //   - Frame N+1: the sibling message is read by apply_damage_boosts
    //     (dealer = None on sibling → no boost), apply_vulnerable (B has
    //     1.5× multiplier → 25.0 × 1.5 = 37.5), apply_damage (B.Hp.current
    //     decrements by 37.5).
    //
    // Then:
    //   - After 2 ticks, B.Hp.current == 100.0 - 37.5 == 62.5.
    //
    // This confirms the sibling goes through the full pipeline, NOT a
    // shortcut that would bypass apply_vulnerable and deal only 25 to B.
    let mut app = build_tether_app();
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(30.0, 0.0));

    // B has a 1.5× vulnerability multiplier.
    {
        let mut b_ent = app.world_mut().entity_mut(b);
        let mut stack = VulnerableStack::default();
        stack.add(test_vuln_source(), 1.5);
        b_ent.insert(stack);
    }

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

    tick(&mut app); // Frame N — primary on A, tether emits sibling to B.
    tick(&mut app); // Frame N+1 — sibling goes through full pipeline on B.

    let b_hp = app.world().get::<Hp>(b).expect("B has Hp").current;
    assert_f32_eq(b_hp, 62.5);
}
