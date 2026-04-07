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
