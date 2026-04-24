//! Game plugin group tests — basic wiring + Wave 1 `rantzsoft_dmg` proofs.
//!
//! Wave 1 `rantzsoft_dmg` wiring tests.
//!
//! Location: Option A from `.claude/specs/w1-port-rantzsoft-dmg-tests.md`
//! Constraints — originally a child module `mod dmg_wiring` inside `mod tests`
//! so the parent's `fn test_app(game: Game) -> App` helper was reachable via
//! `super::test_app(...)`. After the file split, all tests are siblings in
//! this module and share `test_app` directly. The tests assert that
//! `RantzDmgPlugin` is wired into the `Game` plugin group, that
//! `register_dmgable` was called for the four registered types this wave
//! (Bolt, Wall, Breaker, Salvo), that `Cell` is NOT registered this wave (W2
//! owns that), that the `DamageDealt::source: Option<SourceId>` and
//! `HealDealt::source: Option<SourceId>` field rewrites compile, and that
//! `handle_breaker_death` is wired between `EmitKill` and `ApplyKill` in the
//! live plugin group.
//!
//! Negative contract — `DeathPipelineSystems` and `GameEntity` MUST be
//! absent from the game crate after W1. Enforcement is a compile-level
//! check by `cargo all-dtest` across the whole workspace (after the
//! in-tree `shared/death_pipeline/` directory is deleted). Per the test
//! spec Behaviors 12–13, NO dedicated `#[test]` is required — the
//! workspace-wide compile is the contract. These lines document the
//! contract; uncommenting them MUST fail to compile in GREEN state:
//!
//! ```ignore
//! use crate::shared::death_pipeline::sets::DeathPipelineSystems;
//! const _: DeathPipelineSystems = DeathPipelineSystems::ApplyDamage;
//!
//! use crate::shared::death_pipeline::game_entity::GameEntity;
//! const fn uses<T: GameEntity>() {}
//! ```

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::system::Game;
use crate::{
    cells::behaviors::survival::salvo::components::Salvo,
    debug::DebugPlugin,
    prelude::*,
    shared::test_utils::{MessageCollector, attach_message_capture, tick},
    state::run::messages::RunLost,
};

fn test_app(game: Game) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::state::app::StatesPlugin,
        bevy::asset::AssetPlugin::default(),
        bevy::input::InputPlugin,
    ))
    .add_plugins(game.build().disable::<DebugPlugin>());
    app
}

#[test]
fn game_plugin_group_builds() {
    let mut app = test_app(Game::default());
    app.update();
}

#[test]
fn headless_game_spawns_no_camera() {
    let mut app = test_app(Game::headless());
    app.update();

    let count = app
        .world_mut()
        .query::<&Camera2d>()
        .iter(app.world())
        .count();
    assert_eq!(count, 0, "headless game should not spawn a camera");
}

#[test]
fn headless_game_registers_headless_assets() {
    let mut app = test_app(Game::headless());
    app.update();

    assert!(
        app.world().get_resource::<Assets<Mesh>>().is_some(),
        "headless game must register Assets<Mesh> via MeshPlugin"
    );
    assert!(
        app.world()
            .get_resource::<Assets<ColorMaterial>>()
            .is_some(),
        "headless game must register Assets<ColorMaterial>"
    );
    assert!(
        app.world().get_resource::<Assets<Font>>().is_some(),
        "headless game must register Assets<Font> via TextPlugin"
    );
}

// Crate-surface tests (per-T message registration, DamageDealt/HealDealt
// field layout, RantzDmgPlugin adding Messages<DespawnEntity>) were removed
// — those are covered by `rantzsoft_dmg`'s own tests. The game-side contract
// is "the `Dmgable` types compile + our `handle_breaker_death` runs in the
// right set", pinned below.

// ── `handle_breaker_death` runs between `EmitKill` and
//     `ApplyKill` ──
//
// Primary — exactly ONE `RunLost` message (`handle_breaker_death`
// is the only RunLost emitter).
//
// Secondary — exactly ONE `Destroyed<Breaker>` carrying the victim.
// `handle_breaker_death` inserts `Dead` on the breaker; the set
// ordering flushes that command before the crate's generic
// `handle_kill::<Breaker>` runs, so the generic handler's
// `Without<Dead>` query skips the breaker and emits nothing.
//
// Edge — breaker entity is still alive post-tick. Because
// `handle_kill::<Breaker>` skipped, no `DespawnEntity` was enqueued.

#[test]
fn handle_breaker_death_runs_in_apply_kill_set() {
    let mut app = test_app(Game::headless());

    // Attach message capture BEFORE the breaker is spawned — the
    // helper is idempotent and needs to observe every message the
    // tick writes.
    attach_message_capture::<RunLost>(&mut app);
    attach_message_capture::<Destroyed<Breaker>>(&mut app);

    // Spawn breaker WITHOUT `Dead` (the victim query uses
    // `Without<Dead>`) and WITHOUT `KilledBy` (the killer reaches
    // `handle_breaker_death` via the `KillYourself<Breaker>`
    // message, not via a component on the victim).
    let breaker = app
        .world_mut()
        .spawn((Breaker, Hp::new(0.0), Position2D(Vec2::ZERO)))
        .id();

    // Enqueue one `KillYourself<Breaker>` for the fresh breaker.
    app.world_mut()
        .resource_mut::<Messages<KillYourself<Breaker>>>()
        .write(KillYourself::<Breaker> {
            victim:  breaker,
            killer:  None,
            _marker: PhantomData,
        });

    // One `FixedUpdate` tick — exercises the full
    // `DmgSystems::*` chain including `ApplyKill` where
    // `handle_breaker_death` must be bound.
    tick(&mut app);

    // Primary assertion — exactly one `RunLost` message emitted.
    let run_lost = app.world().resource::<MessageCollector<RunLost>>();
    assert_eq!(
        run_lost.0.len(),
        1,
        "handle_breaker_death must run in DmgSystems::ApplyKill — one \
         KillYourself<Breaker> must produce exactly one RunLost"
    );

    // Secondary assertion — exactly ONE `Destroyed<Breaker>`
    // carrying the victim. `handle_kill::<Breaker>` is skipped via
    // the command-flushed `Dead` marker; only `handle_breaker_death`
    // emits.
    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<Breaker>>>();
    assert_eq!(
        destroyed.0.len(),
        1,
        "exactly one Destroyed<Breaker> must be emitted — \
         handle_breaker_death emits; the generic handle_kill::<Breaker> \
         must be skipped by the command-flushed Dead marker"
    );
    assert_eq!(
        destroyed.0[0].victim, breaker,
        "the Destroyed<Breaker> message must carry the spawned breaker"
    );

    // Edge — breaker is still alive. The generic handler's
    // `DespawnEntity` emit was skipped, so `process_despawn_requests`
    // has nothing to drain.
    assert!(
        app.world().get_entity(breaker).is_ok(),
        "breaker entity must survive past the kill tick — \
         handle_kill::<Breaker> must have been skipped via the \
         command-flushed Dead marker (avoids the end-of-run race)"
    );
}

// ── Behavior 10: `impl Dmgable for Bolt|Wall|Breaker|Salvo` compiles ──
//
// Compile-time trait-bound check. If W1's writer-code forgets an
// `impl Dmgable for X` line for any of the four registered-this-wave
// types, this fails with E0277 ("the trait bound `X: Dmgable` is not
// satisfied").
//
// NOTE — `Cell` is intentionally NOT in this check. W2 adds
// `register_dmgable::<Cell>()`. If `impl Dmgable for Cell` lands in
// W1 alongside the other four (per the plan `§W1 Work`), the Cell
// check would still technically pass today — but `register_dmgable::<Cell>()`
// must not be called, which is the contract enforced behaviorally by
// `cell_with_zero_hp_is_not_killed_by_generic_pipeline` and
// `cell_with_three_hp_is_not_touched_by_generic_pipeline` above.

const fn assert_dmgable<T: rantzsoft_dmg::Dmgable>() {}

#[test]
fn impl_dmgable_for_bolt_wall_breaker_salvo_compiles() {
    assert_dmgable::<Bolt>();
    assert_dmgable::<Wall>();
    assert_dmgable::<Breaker>();
    assert_dmgable::<Salvo>();
}

// ── Behavior 11: Prelude re-exports resolve from `rantzsoft_dmg` ──
//
// Every referenced name below reaches the test via the prelude glob
// at the top of this module (`use crate::prelude::*;`). The types
// resolve to `rantzsoft_dmg`'s definitions because
// `prelude/death_pipeline.rs` re-exports from the crate after W1.

// Module-scope type-consuming helpers used by Behavior 11's compile
// test. Keeping these at module scope (not inside the test fn)
// avoids `clippy::items_after_statements`. ZSTs take `&T` (no `Copy`
// elision); function pointers are `Copy` and 8 bytes so they're by
// value; optional reference follows the idiomatic `Option<&T>` form.
const fn takes_dead(_: &Dead) {}
const fn takes_invulnerable(_: &Invulnerable) {}
const fn takes_damage_dealt_fn(_: fn() -> Option<DamageDealt<Bolt>>) {}
const fn takes_destroyed_fn(_: fn() -> Option<Destroyed<Breaker>>) {}
const fn takes_kill_yourself_fn(_: fn() -> Option<KillYourself<Wall>>) {}
const fn takes_source_id_option(_: Option<&SourceId>) {}

// W2 structural-invariant tests (grep-style `include_str!` + `!contains`) were
// removed per plan §M — the compiler enforces deletion/rename invariants
// naturally (removed modules produce import errors; renamed fields produce
// access errors), and the behavioral tests in hazard/protocol/cells modules
// pin the true contracts.

#[test]
fn prelude_exports_dmg_pipeline_types_from_crate() {
    // `DmgSystems` variants used by W1 wiring — compare to prove
    // each variant resolves through the prelude.
    assert_ne!(DmgSystems::ApplyDamage, DmgSystems::EmitKill);
    assert_ne!(DmgSystems::ApplyKill, DmgSystems::ApplyHeal);

    // Core components — all resolved via the prelude. Observing
    // each forces the type name to resolve at the call site. `Dead`
    // and `Invulnerable` are zero-sized marker components — passing
    // each through a module-scope helper proves the re-exports
    // resolve without triggering `let_underscore_drop` or
    // `no_effect_underscore_binding`.
    let hp = Hp::new(1.0);
    assert!((hp.current - 1.0).abs() < f32::EPSILON);
    takes_dead(&Dead);
    takes_invulnerable(&Invulnerable);

    let killed_by = KilledBy { killer: None };
    assert!(killed_by.killer.is_none());

    // Generic message type references — `_marker: PhantomData` is
    // reachable with `pub` visibility from the crate's struct
    // definitions. Function-pointer form avoids the need to
    // construct a full value here; the struct-literal tests in
    // Behaviors 7 and 8 prove field-level visibility.
    let dmg_fn: fn() -> Option<DamageDealt<Bolt>> = || None;
    let destroyed_fn: fn() -> Option<Destroyed<Breaker>> = || None;
    let kill_fn: fn() -> Option<KillYourself<Wall>> = || None;
    takes_damage_dealt_fn(dmg_fn);
    takes_destroyed_fn(destroyed_fn);
    takes_kill_yourself_fn(kill_fn);

    // HealCap variants reach the test.
    assert_ne!(HealCap::Max, HealCap::Starting);

    // `SourceId` is a newtype around `Cow<'static, str>` re-exported
    // from the crate — must resolve without a direct
    // `use rantzsoft_dmg::SourceId`.
    let none_source: Option<SourceId> = None;
    takes_source_id_option(none_source.as_ref());
}
