---
name: vetted_dependencies
description: Durable security baseline for the brickbreaker workspace — unsafe analysis patterns, known panic surface, vetted dep state
type: project
---

## Dependency Security Baseline

Current dep snapshot and duplicate/wontfix findings are in `guard-dependencies/dependency-snapshot.md` and `guard-dependencies/known-findings.md`. The security guard focuses on unsafe code, panic surface, and deserialization risk.

### cargo audit recurring pattern
- Single recurring warning: `paste 1.0.15` — RUSTSEC-2024-0436 (unmaintained, not a CVE)
- Transitive via: metal → wgpu-hal → wgpu → bevy_render → bevy. Not directly controllable.
- `cargo deny` exits code 1 due to deny.toml treating this warning as an error (expected).
- No new direct-dependency security advisories found through 2026-04-06.

### cargo machete
- No unused dependencies found through 2026-04-06 (all audits clean).

## Known Unsafe Blocks in Workspace
- None in breaker-game/src/ (workspace lint: `unsafe_code = "deny"`)
- No build.rs files anywhere in the workspace

## Vetted Patterns — rantzsoft_stateflow (added 2026-04-03)

Security-reviewed patterns in `rantzsoft_stateflow` — all confirmed safe:

- `Arc<dyn OutTransition/InTransition/OneShotTransition>` in TransitionType: cloned via Arc::clone, no double-free, no unsound downcasting.
- `Box<dyn FnOnce(...)>` in WorldCallback (PendingTransition): stored as Option, consumed once via `.take()` + if-let, never called twice. Sound.
- `Box<dyn Fn(&World) -> TransitionType>` in TransitionKind::Dynamic: called at dispatch time from &World, no mutable aliasing.
- `Box<dyn Fn(&mut World)>` closures in TransitionRegistry entries: called via `world.resource_scope` which provides exclusive World access. No aliasing.
- TransitionProgress elapsed/duration division: all 12 run systems guard `if progress.duration > 0.0` before dividing; zero-duration case returns `t = 1.0`. No panic surface.
- `Mutex<Vec<RegistrationFn>>` in RantzStateflowPlugin: `.expect("poisoned")` has `#[allow]` with reason. Lock only held during plugin build, never across await points. Poison is unrecoverable. Safe.
- `debug_assert!` in handle_transition_over for OutIn invariant: fires only in debug builds. The hard invariant violation path returns early after the assert.
- Deferred ChangeState re-queue pattern: dispatch_message_routes re-queues ChangeState if ActiveTransition is present. Bounded by transition duration (always finite). Not an infinite loop.

## Vetted Patterns — state/plugin.rs (added 2026-04-06)

- `resolve_node_next_state()` uses `world.resource::<NodeOutcome>()` — safe because NodeOutcome is `init_resource'd` in RunPlugin::build(), which runs before any route resolver fires.
- Double-registration of `cleanup_on_exit::<NodeState>`: registers on both `OnEnter(NodeState::Teardown)` and `OnEnter(RunState::Teardown)`. In the quit-from-pause path, NodeState::Teardown fires first, despawning all CleanupOnExit<NodeState> entities. The second run iterates zero entities — safe. Intentional safety-net.

## Known RON Deserialization Panic Surface

- `animate_transition/system.rs`: divides by `timer.duration` (f32 from RON config) with no zero guard. Only reachable when `transition.duration == 0.0` in RON data. See `ron_deserialization_patterns.md` for full analysis.
- All other RON deserialization routes through Bevy asset pipeline — same panic surface as prior audits.

## proptest Removal
proptest was removed from dev-dependencies in 2026-03-28. No dev-dependencies remain in breaker-game.

## breaker-scenario-runner new dependencies (added/confirmed 2026-04-07)

The following direct dependencies in breaker-scenario-runner/Cargo.toml have been vetted:
- `clap = "4"` with `derive` feature — widely audited CLI parser, no known CVEs.
- `ron = "0.12"` — same version used in the wider workspace; confirmed safe deserialization
  pattern (returns Result, not panic). Carries the same recursive-depth concern as prior
  audits (RON is not bounded by default) — mitigated because scenario files are dev assets.
- `tracing = "0.1"`, `tracing-subscriber = "0.3"` — standard logging crates, no known CVEs.
- `rand = "0.9"` — new version (codebase was on 0.8 in breaker-game). No known CVEs.
- `serde = "1"` with `derive` — vetted across the workspace.
- `breaker`, `rantzsoft_*` — workspace path dependencies; already audited.

NOTE: `cargo audit` and `cargo deny` could not be run in this session (Bash tool not available).
Last known cargo audit result: RUSTSEC-2024-0436 (paste crate — unmaintained, not a CVE) — still
transitive via bevy. No direct-dep security advisories known through 2026-04-07.

## cargo audit result — feature/bolt-birthing-animation (2026-04-08)

`cargo audit` run successfully. Result: 1 warning (allowed), 0 errors.
Warning: `paste 1.0.15` — RUSTSEC-2024-0436 (unmaintained, no CVE). Same transitive path as prior.
No new advisories. cargo machete: no unused dependencies found.

## cargo audit result — feature/cell-builder-pattern (2026-04-08)

`cargo audit` run successfully. Result: 1 warning (allowed), 0 errors.
Same RUSTSEC-2024-0436 (paste) recurring warning. No new advisories.
cargo machete: no unused dependencies found.

## cargo audit result — Toughness + HP Scaling (2026-04-08)

`cargo audit` run successfully. Result: 1 warning (allowed), 0 errors.
Same RUSTSEC-2024-0436 (paste — unmaintained, no CVE). No new direct-dep security advisories.
cargo machete: no unused dependencies found. No new dependencies introduced by this feature.

## Vetted Patterns — rantzsoft_stateflow TransitionType::None (added 2026-04-08)

- `TransitionType::None` variant: cloned safely (trivially `Clone`). `type_ids()` returns `vec![]` — correct.
- `begin_transition`: early-return on `TransitionType::None` before any pause/resource insert. Instant
  state change only. No resource leak, no double-change. Sound.
- `unreachable!("handled by early return above")` at system.rs:301: sound. The `begin_transition`
  function handles `None` via early return on line 219 before reaching the `match` at line 236.
  The compiler exhaustiveness checker sees `None` as reachable in the match, hence the arm exists,
  but it truly cannot be reached at runtime. No panic risk in practice.
- Dynamic closures calling `world.resource::<MainMenuSelection>()` in register_parent_routes:
  closures at system.rs:143, 151, 195, 204 — `MainMenuSelection` inserted by `spawn_main_menu`
  which runs `OnEnter(MenuState::Main)` before routing ever evaluates these closures. The routes
  only fire when `GameState::Menu` or `MenuState::Main` is the current state, which requires
  `MenuState::Main` to have been entered. Insertion is guaranteed to precede dispatch. Safe.
- `world.resource::<NodeOutcome>()` in `resolve_node_next_state` (system.rs:223): `NodeOutcome`
  is `init_resource`'d in RunPlugin::build(). Safe (confirmed in prior audit, still holds).

## Toughness + HP Scaling panic surface (added 2026-04-08)

- `ToughnessConfig` has no `validate()` method. Fields (`tier_multiplier`, `boss_multiplier`,
  `node_multiplier`, `*_base`) are raw f32 loaded from RON with no NaN/Inf/zero-guard.
  - `tier_multiplier.powi(tier_i32)` in `tier_scale()`: if `tier_multiplier` is NaN/Inf (from
    a bad RON), the result is NaN/Inf; this flows through to `Hp::max` and
    `Hp::current` (unified health component; `CellHealth` replaced by `Hp`). No panic in f32 arithmetic, but HP becomes NaN, causing gameplay
    corruption (cells never die). Warning-level.
  - `tier_multiplier = 0.0` → `0.0^0 = 1.0` (safe), `0.0^n = 0.0` (tier ≥ 1: HP zeroes out).
    Cells with 0.0 HP would immediately despawn on spawn. Not a panic, gameplay breakage only.
  - `boss_multiplier = 0.0` → boss cells have same HP as normal cells. Undesirable but not a crash.
  - `node_multiplier.mul_add(pos_f32, 1.0)` with very large `node_multiplier` and high
    `position_in_tier` can produce Inf HP. No panic, game stall (cells never die).
  - Threat model: these are shipped assets, not user-controlled. Info/Warning-level only.
- Hot-reload path in `propagate_node_layout_changes` passes `toughness_config: None`,
  falling back to `Toughness::default_base_hp()` (hardcoded). Safe — no NaN risk on hot-reload.
- `propagate_cell_type_changes` also uses `default_base_hp()` (not `ToughnessConfig`). Safe.

## Cell builder / guarded behavior panic surface (added 2026-04-08)

- `GuardedBehavior::validate()` checks `guardian_hp` and `slide_speed` but NOT `guardian_color_rgb`.
  NaN or Inf in color fields passes validation and reaches `Color::linear_rgb()` at spawn time.
  Bevy's `Color::linear_rgb()` accepts any f32 — no panic, but produces garbled color. Info-level.
- `ring_slot_offset(slot: u8)` wildcard arm uses `debug_assert!(false, ...)` + returns (0.0, 0.0).
  In release builds out-of-range slots silently return (0.0, 0.0). Only called from two sites:
  (1) `spawn_guardian_children` with slot from `collect_guardian_slots()` (0..=7 always),
  (2) `slide_guardian_cells` system with `slide_target.0` (a u8 mod 8, always 0..=7). Safe.
- `compute_grid_scale` divides by `default_grid_width` and `default_grid_height` at lines 60-61
  with no zero-guard. If `cols=0` or `rows=0` in a node RON, `grid_extent` returns a negative
  number (step * 0 - padding = -padding), so division produces a negative scale; `.min(1.0)`
  keeps it negative; `cell_width = config.width * negative_scale` goes negative; cell positions
  become nonsensical. Not a panic in f32, but a silent layout corruption. Warning-level.
  NOTE: `NodeLayout` has no `validate()` at load time — cols/rows are not guarded.
- `CellTypeRegistry::seed()` calls `validate()` and skips invalid definitions with `warn!()`.
  The validation gate is in production code (not test-only). Valid.

## cargo audit — feature/test-infrastructure-consolidation (2026-04-09)

`cargo audit` could not be run (Bash tool denied in this session).
No new dependencies introduced by this branch — breaker-game/Cargo.toml unchanged.
Expected result: same RUSTSEC-2024-0436 (paste) recurring warning, no new advisories.

## Lint config changes — feature/test-infrastructure-consolidation (2026-04-09)

Workspace `Cargo.toml` lint escalations confirmed intentional and safe:
- `let_underscore_drop`, `unreachable_pub`, `trivial_casts`, `trivial_numeric_casts`: warn → deny
- `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`: warn → deny
- `nursery` blanket group removed; replaced with explicit opt-in list (same or stricter coverage)
- `redundant_pub_crate = "allow"` removed (was a nursery-group override, now moot)
- Pre-commit hook: removed `-D warnings` flag (now redundant — workspace lints are all "deny")

Implication: `#[allow(...)]` attributes in `#[cfg(test)]` code must now carry a reason
(allow_attributes_without_reason = "deny" was already on). The three `.unwrap()` calls
in `walls/test_utils.rs` (lines 55, 74, 93) require either an `#[allow]` with reason
or replacement with `expect()` (but `expect_used` is now deny too) — need `.unwrap_or_else`
or restructured logic to satisfy the new lint level. This is an existing clippy finding.

## cargo audit result — tiling/visual-mode (2026-04-15)

`cargo audit` run successfully. Result: 3 allowed warnings, 0 errors.
- RUSTSEC-2024-0436: `paste 1.0.15` — unmaintained, no CVE. Same transitive path. Expected recurring.
- RUSTSEC-2026-0097: `rand 0.8.5` and `rand 0.9.2` — same advisory as 2026-04-14 (custom logger unsound).
  No new advisories introduced by this branch.
cargo machete: no unused dependencies found.

No new direct dependencies introduced by breaker-scenario-runner tiling feature (tiling.rs, app.rs,
streaming.rs use only existing workspace deps: bevy, breaker, std).

## cargo audit result — feature/effect-system-refactor (2026-04-14)

`cargo audit` run successfully. Result: 3 warnings, 0 errors.
- RUSTSEC-2024-0436: `paste 1.0.15` — unmaintained, no CVE. Same transitive path (bevy_render). Expected recurring.
- RUSTSEC-2026-0097: `rand 0.8.5` — NEW advisory. "Unsound with custom logger using rand::rng()". Severity: unknown.
  Pulled in by new dep `ordered-float 5.1.0` which depends on `rand 0.8.x`. NOT the project's own `rand 0.9` dep.
- RUSTSEC-2026-0097: `rand 0.9.2` — same advisory, different instance. This is the project's own `rand = "0.9"`.
cargo machete: no unused dependencies found.

Threat model assessment for RUSTSEC-2026-0097:
- This is a single-player desktop game with no networking, no multi-threaded logging, no custom logger.
- The advisory requires a custom logger that calls rand::rng() from within a Logger::log() implementation.
- This game uses tracing/tracing-subscriber with standard formatting subscribers. No custom Logger impls.
- Risk is theoretical-only in this codebase. Info-level for this threat model.

### New dependency: ordered-float = "5" (added on feature/effect-system-refactor)
- Version: 5.1.0
- Purpose: provides `OrderedFloat<f32>` and `NotNan<f32>` wrappers enabling `Ord`/`Hash` on floats
- Used for: `EffectType`, `SpeedBoostConfig`, `ChainLightningConfig`, `RandomEffectConfig` field types
- Features: `serde` (serialize/deserialize support for effect configs)
- Security notes:
  - `OrderedFloat<f32>` allows NaN values (unlike `NotNan<f32>`). NaN propagates silently into gameplay math.
  - Effect configs using `OrderedFloat` (SpeedBoostConfig.multiplier, ChainLightningConfig.damage_mult etc.)
    have no NaN guards. A NaN-valued config causes gameplay corruption (cells never die, speed broken)
    rather than a panic. Warning-level; mitigated because configs are Rust-authored, not RON-loaded.
  - Pulls in RUSTSEC-2026-0097 via rand 0.8.x. See audit note above.
  - No known CVEs against ordered-float itself.

## Lint config changes — feature/effect-system-refactor (2026-04-14)

Workspace `Cargo.toml` lint RELAXATIONS introduced on this branch:
- `todo = "warn"` (was `"deny"` after feature/test-infrastructure-consolidation)
- `unimplemented = "warn"` (was `"deny"` after feature/test-infrastructure-consolidation)
- `cast_precision_loss = "allow"` (new allow — previously not present)
- `cast_sign_loss = "allow"` (new allow — previously not present)
- `cast_possible_truncation = "allow"` (new allow — previously not present)

Security implication: `todo`/`unimplemented` downgrade means production code can now contain
`todo!()` or `unimplemented!()` macros that compile but panic at runtime if a code path is
reached. This is a regression from the 2026-04-09 hardening. The workspace previously denied these
to guarantee no "not yet implemented" paths could reach production. With warn, they can ship.
The three cast lints are arithmetic-only and have no direct security surface in a desktop game.

## effect_v3 RON deserialization surface (added 2026-04-14)

Tree/ScopedTree/Terminal/EffectType all derive Serialize/Deserialize. However, these types are NOT
deserialized from RON files at runtime — chip effect trees are built programmatically in Rust (stamp
commands, builder pattern). No user-facing or asset-facing RON parser touches these types at runtime.
The recursive type structure (Tree→Terminal→Tree, ScopedTree→Tree, EffectType→RandomEffectConfig→EffectType)
would create deep recursion under serde if deserialized from untrusted input. This risk is latent only
— it is not exercised by any current code path. If a RON loading path is added in the future, these
types must have a depth limit enforced before deserialization.

## Birthing animation panic surface (added 2026-04-08)

- `tick_birthing.rs`: `birthing.fraction()` calls `Timer::fraction()` — Bevy returns 0.0..=1.0,
  no division by zero possible (Bevy guards zero-duration internally).
- `scale.x = birthing.target_scale.x * t` — simple f32 multiply, no overflow in f32, no panic.
- `begin_node_birthing.rs`: all operations are ECS queries + component inserts — no panic surface.
- `shared/birthing.rs`: `Birthing::new` always sets BIRTHING_DURATION (0.3) — never zero.
  `fraction()` delegates to `Timer::fraction()` — safe.
