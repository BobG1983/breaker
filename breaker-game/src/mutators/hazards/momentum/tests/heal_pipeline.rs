//! Group E — Full heal pipeline integration (Behaviors 28–31).
//!
//! End-to-end: `attach_momentum_ceiling` → `apply_damage::<Cell>` →
//! `momentum_heal_on_nonlethal` → `apply_heal::<Cell>`. Pins that the ceiling
//! is lifted before heals land so heals can push `hp.current` past
//! `hp.starting`. Behavior 30 is the negative control proving
//! `attach_momentum_ceiling` is load-bearing.

use bevy::prelude::*;
use rantzsoft_dmg::{RantzDmgAppExt, RantzDmgPlugin};

use super::{
    super::system::{MomentumConfig, attach_momentum_ceiling, momentum_heal_on_nonlethal},
    helpers::{
        add_momentum_stacks, canonical_momentum_config, heal_collector_len,
        install_momentum_config, run_fixed_update, spawn_cell_at, test_app_playing,
        write_cell_damage,
    },
};
use crate::{cells::components::Cell, prelude::*};

/// Wires the full pipeline in the correct order:
/// `attach_momentum_ceiling` → `apply_damage` → `momentum_heal_on_nonlethal` → `apply_heal`.
///
/// `register_dmgable::<Cell>` wires `apply_damage::<Cell>`, `apply_heal::<Cell>`,
/// `detect_deaths::<Cell>`, `handle_kill::<Cell>` automatically.
fn test_app_full_pipeline() -> App {
    let mut app = test_app_playing();
    app.add_plugins(RantzDmgPlugin);
    let _ = app.register_dmgable::<Cell>();
    app.add_systems(
        FixedUpdate,
        (
            attach_momentum_ceiling.before(DmgSystems::ApplyDamage),
            momentum_heal_on_nonlethal.in_set(DmgSystems::PostApplyDamage),
        ),
    );
    app
}

// ── Behavior 28 — non-lethal hit lands 10 HP at stack 1 end-to-end ──────────

#[test]
fn nonlethal_stack_one_end_to_end_lands_fifteen_hp() {
    let mut app = test_app_full_pipeline();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    // Fresh cell — hp.max = None. attach_momentum_ceiling should lift it.
    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);
    write_cell_damage(&mut app, cell, 5.0);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max,
        Some(20.0),
        "attach_momentum_ceiling must lift hp.max to Some(20.0); got {:?}",
        hp.max
    );
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "end-to-end tick must leave hp.current at 15.0 (=5 after damage +10 heal); got {}",
        hp.current
    );
}

// ── Behavior 29 — stack 3 pushes current from 40.0 → 70.0 end-to-end ────────

#[test]
fn nonlethal_stack_three_end_to_end_lands_seventy_hp() {
    let mut app = test_app_full_pipeline();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 3);

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 50.0, 50.0);
    write_cell_damage(&mut app, cell, 10.0);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max,
        Some(100.0),
        "attach must lift hp.max to Some(100.0) (=50 * 2); got {:?}",
        hp.max
    );
    assert!(
        (hp.current - 70.0).abs() < f32::EPSILON,
        "end-to-end stack-3 must leave hp.current at 70.0; got {}",
        hp.current
    );
}

// ── Behavior 30 — without attach_momentum_ceiling, heal clamps at starting ──

#[test]
fn without_attach_ceiling_heal_clamps_at_starting() {
    // Deliberately build a pipeline WITHOUT attach_momentum_ceiling — to prove
    // it is load-bearing for the "heal past pristine" semantic.
    let mut app = test_app_playing();
    app.add_plugins(RantzDmgPlugin);
    let _ = app.register_dmgable::<Cell>();
    app.add_systems(
        FixedUpdate,
        momentum_heal_on_nonlethal.in_set(DmgSystems::PostApplyDamage),
    );
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);
    write_cell_damage(&mut app, cell, 5.0);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "without attach_momentum_ceiling the heal is clamped at hp.starting \
         (10.0); got {}. Without the ceiling system, Momentum's 'heal past \
         pristine' semantic is neutralized.",
        hp.current
    );
}

// ── Behavior 31 — lethal hit: no revival via Momentum ───────────────────────
//
// Game-specific invariant: when a hit is lethal (`hp.current <= 0.0` post-
// damage), `momentum_heal_on_nonlethal` MUST NOT emit a heal. The crate-owned
// death chain (`detect_deaths` → `handle_kill` → `DespawnEntity` →
// `process_despawn_requests`) is covered by the crate's own tests; this test
// only pins the Momentum-specific "no revival" contract.
#[test]
fn lethal_hit_emits_no_momentum_heal() {
    let mut app = test_app_full_pipeline();

    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);
    write_cell_damage(&mut app, cell, 15.0); // lethal

    run_fixed_update(&mut app);

    // The positive signal: zero Momentum heals were emitted this tick. The
    // crate pipeline despawns the dead cell in FixedPostUpdate; querying the
    // entity post-tick would return None for every component (entity gone).
    assert_eq!(
        heal_collector_len(&app),
        0,
        "lethal hit must NOT trigger a Momentum heal — no revival"
    );
    let _ = cell;
    let _ = HealCap::Max;
    let _ = MomentumConfig {
        base_hp_per_hit:      0.0,
        per_level_hp_per_hit: 0.0,
    };
}
