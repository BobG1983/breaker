# Phantom breaker — builder transition, lifespan dispatch, shared infra

## Agent Teams Trial

This is the designated trial item for the **agent teams** beta feature. Read this section before starting implementation.

### Why this item?

Phantom breaker has ~6-7 waves, touches 4+ domains, and includes a correctness-sensitive migration (`grade_bump .single_mut() → .iter_mut()`) likely to need multi-turn fix cycles. These properties are exactly where persistent teammate context and direct peer communication pay off over fire-and-forget background agents.

### Feature overview

Agent teams (`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS: 1` in `.claude/settings.json` — already enabled) spawn **persistent Claude sessions** that maintain conversation history across multiple messages. Unlike `run_in_background: true` agents (briefed from scratch each time), team members accumulate context: files they read, decisions made, patterns observed, what was tried and failed.

**Key difference from background agents**: teammates can message each other directly via `SendMessage(to: "teammate-name", ...)` — no orchestrator round-trip required. The orchestrator receives a brief idle notification when this happens, keeping visibility without being in the loop for every exchange.

**This trial uses the actual agent definitions** (`subagent_type:` fields) rather than custom hybrid agents. Each agent already has the right model, tools, rules, and memory for its role.

**Teammate discovery**: `TeamCreate` writes `~/.claude/teams/breaker-team/config.json` automatically as agents join. Every agent reads this file on startup to learn all teammates' names — no hardcoded names in prompts.

### Team composition

All agents spawn at once before Wave 1. They wait idle until messaged.

#### Pipeline coordinator — persistent

| Name | `subagent_type` | Role |
|------|----------------|------|
| `tdd-guard` | `general-purpose` | Drives wave transitions. Reads the plan file once at spawn. When it receives a wave-complete signal, it looks up the next wave and messages `planning-writer-specs-tests` with the kickoff. Escalates to orchestrator only for circuit breaks and design decisions. Does NOT need wave details — only needs to know which wave just completed and what the next wave is. |

Brief: "Read the plan file at `<path>`. When you receive `wave N complete`, message planning-writer-specs-tests with the wave N+1 kickoff from the plan. Message orchestrator with each wave transition for session-state. Escalate to orchestrator for circuit breaks or design decisions. Do not initiate."

#### Core pipeline — persistent

Spawn once, message across every wave. Accumulate codebase shape, spec decisions, test helper paths, import patterns, and failure history so they never re-read static context.

| Name | `subagent_type` | Accumulates across waves |
|------|----------------|--------------------------|
| `planning-writer-specs-tests` | `planning-writer-specs-tests` | domain shape, prior spec decisions, reviewer flags already cleared |
| `planning-reviewer-specs-tests` | `planning-reviewer-specs-tests` | what was approved vs dismissed — never re-raises cleared points |
| `writer-tests` | `writer-tests` | helper functions, app setup patterns, import paths, naming conventions |
| `reviewer-tests` | `reviewer-tests` | approved test patterns — doesn't re-flag the same idiom in wave 4 that cleared in wave 2 |
| `planning-writer-specs-code` | `planning-writer-specs-code` | failing test contracts from prior waves, system topology |
| `planning-reviewer-specs-code` | `planning-reviewer-specs-code` | approved impl patterns — same as above |
| `writer-code` | `writer-code` | builder API shape, module structure, gating idioms, ordering constraints |
| `debugger` | `debugger` | full failure history, disproven hypotheses, ECS scheduling gotchas |

#### Single persistent runner

One `runner-cargo` teammate handles all cargo execution. It serializes cargo naturally (only one run at a time) and reports results directly back to whoever requested the run — no orchestrator round-trip needed for routine gates. The orchestrator only hears about results when session-state needs updating (gate pass/fail).

| Name | `subagent_type` | Handles |
|------|----------------|---------|
| `runner-cargo` | `runner-cargo` | RED gate, GREEN gate, lint runs, scenario runs — all via `SendMessage` requests |

#### Verification tier — persistent

Spawn once. Used at the commit gate (Standard) and pre-merge gate (Full). Accumulate what was reviewed and approved so they don't re-flag already-cleared patterns in later waves.

| Name | `subagent_type` |
|------|----------------|
| `reviewer-completeness` | `reviewer-completeness` |
| `reviewer-correctness` | `reviewer-correctness` |
| `reviewer-quality` | `reviewer-quality` |
| `reviewer-bevy-api` | `reviewer-bevy-api` |
| `reviewer-architecture` | `reviewer-architecture` |
| `reviewer-performance` | `reviewer-performance` |
| `reviewer-file-length` | `reviewer-file-length` |
| `reviewer-scenarios` | `reviewer-scenarios` |

#### On-demand consultants — persistent

Spawn once; stay idle. **Any peer can message them directly** before committing to an approach. They never initiate. They accumulate what they've validated so they don't re-check the same decision in wave 5 that was cleared in wave 2. Escalate to the orchestrator only for genuine new decisions requiring human input.

| Name | `subagent_type` | Consult when |
|------|----------------|--------------|
| `guard-game-design` | `guard-game-design` | approach touches player-facing behavior |
| `guard-docs` | `guard-docs` | new/changed public types, deleted systems |
| `guard-security` | `guard-security` | unsafe, asset loading, RON deserialization |
| `guard-dependencies` | `guard-dependencies` | new crate added or version bumped |
| `guard-agent-memory` | `guard-agent-memory` | memory drift suspected |
| `researcher-codebase` | `researcher-codebase` | tracing data flow through unfamiliar domain |
| `researcher-system-dependencies` | `researcher-system-dependencies` | checking for system ordering conflicts |
| `researcher-bevy-api` | `researcher-bevy-api` | unfamiliar Bevy 0.18 API |
| `researcher-impact` | `researcher-impact` | before renaming or changing a signature |

#### Stateless on-demand (no cross-wave accumulation benefit)

Spawn fresh each time via background `Agent` calls as needed.

`writer-scenarios`, `researcher-rust-errors`, `researcher-rust-idioms`, `researcher-git`, `researcher-crates`

---

### Communication topology

```
orchestrator
  │
  ├─→ planning-writer-specs-tests: "wave N, scope, decisions, spec path"
  │        │
  │        ↕ (revision loop, peer-to-peer)
  │        │
  │   planning-reviewer-specs-tests ──→ writer-tests: "approved, write tests at spec path"
  │        │                                 │
  │        │                                 ↕ (spec clarifications)
  │        │                       planning-writer-specs-tests
  │        │
  │   planning-reviewer-specs-tests ──→ orchestrator: "test spec approved, tests written at <path>"
  │
  ├─→ runner-cargo: "run RED gate — tests at <path>"
  │   runner-cargo ──→ orchestrator: "RED PASS" or "RED FAIL: <output>"
  │
  ├─→ planning-writer-specs-code: "RED passed, failing tests at <path>"
  │        │
  │        ↕ (revision loop, peer-to-peer)
  │        │
  │   planning-reviewer-specs-code ──→ writer-code: "approved, implement, tests at <path>"
  │                                         │
  │                                         ↕ (spec clarifications)
  │                               planning-writer-specs-code
  │                                         │
  │                                         ↓  messages orchestrator: "done, run GREEN gate"
  │
  ├─→ runner-cargo: "run GREEN gate"
  │   runner-cargo ──→ orchestrator: "GREEN PASS" or "GREEN FAIL: <output>"
  │   (on FAIL) orchestrator ──→ writer-code: "GREEN FAIL, <verbatim output>"
  │                writer-code ──→ debugger: "fix spec"
  │                debugger ──→ writer-code: "root cause + fix hint"
  │                writer-code ──→ orchestrator: "fixed, re-run GREEN gate"
  │
  └─ updates session-state after every notification

Any pipeline peer ──→ guard-game-design / guard-docs / researcher-*: "check X before I commit"
On-demand consultant ──→ asker: "fine" or "flag: [issue]"
On-demand consultant ──→ orchestrator: only for genuine decisions requiring human input
```

The orchestrator's job: wave kickoffs, cargo requests to `runner`, session-state updates, circuit-break escalations. Everything inside a wave self-organizes.

---

### How the orchestrator spawns the team

**Step 1 — Create the team first.** This generates `~/.claude/teams/breaker-team/config.json`, which every subsequently spawned agent reads to discover teammates by name.

```
TeamCreate({
  team_name: "breaker-team",
  description: "Full pipeline team for phantom-breaker refactor, waves 1-7"
})
```

**Step 2 — Spawn all persistent teammates in parallel** using their actual `subagent_type`. Each prompt tells the agent to read the config file for teammate names and wait for a message before acting.

Core pipeline (8 agents):
```
Agent({ name: "planning-writer-specs-tests",  subagent_type: "planning-writer-specs-tests",  team_name: "breaker-team",
  prompt: "Read docs/todos/detail/phantom-breaker.md. Read ~/.claude/teams/breaker-team/config.json to learn teammate names. You are the persistent test-spec writer. After writing a test spec, message planning-reviewer-specs-tests directly. Wait for your first message before acting." })

Agent({ name: "planning-reviewer-specs-tests", subagent_type: "planning-reviewer-specs-tests", team_name: "breaker-team",
  prompt: "Read docs/todos/detail/phantom-breaker.md. Read ~/.claude/teams/breaker-team/config.json. Review test specs from planning-writer-specs-tests. Run revision loop peer-to-peer. When approved, message writer-tests AND orchestrator." })

Agent({ name: "writer-tests",          subagent_type: "writer-tests",                  team_name: "breaker-team",
  prompt: "Read docs/todos/detail/phantom-breaker.md. Read ~/.claude/teams/breaker-team/config.json. Write failing tests when planning-reviewer-specs-tests sends an approved spec. Message orchestrator when done." })

Agent({ name: "reviewer-tests",        subagent_type: "reviewer-tests",                team_name: "breaker-team",
  prompt: "Read docs/todos/detail/phantom-breaker.md. Read ~/.claude/teams/breaker-team/config.json. Review tests when orchestrator sends the test file path. Message orchestrator with findings." })

Agent({ name: "planning-writer-specs-code",     subagent_type: "planning-writer-specs-code",    team_name: "breaker-team",
  prompt: "Read docs/todos/detail/phantom-breaker.md. Read ~/.claude/teams/breaker-team/config.json. Write code specs after RED gate confirmation. Message planning-reviewer-specs-code directly." })

Agent({ name: "planning-reviewer-specs-code",   subagent_type: "planning-reviewer-specs-code",  team_name: "breaker-team",
  prompt: "Read docs/todos/detail/phantom-breaker.md. Read ~/.claude/teams/breaker-team/config.json. Review code specs from planning-writer-specs-code. When approved, message writer-code AND orchestrator." })

Agent({ name: "writer-code",          subagent_type: "writer-code",                   team_name: "breaker-team",
  prompt: "Read docs/todos/detail/phantom-breaker.md. Read ~/.claude/teams/breaker-team/config.json. Implement production code when planning-reviewer-specs-code sends approval. On GREEN FAIL, message debugger with full error. Message orchestrator when done." })

Agent({ name: "debugger",             subagent_type: "debugger",                      team_name: "breaker-team",
  prompt: "Read docs/todos/detail/phantom-breaker.md. Read ~/.claude/teams/breaker-team/config.json. Diagnose GREEN failures from writer-code. Return fix spec hints to writer-code. After 3 failed attempts, escalate to orchestrator." })
```

Single runner:
```
Agent({ name: "runner-cargo", subagent_type: "runner-cargo", team_name: "breaker-team",
  prompt: "Read ~/.claude/teams/breaker-team/config.json. You handle all cargo execution: tests (RED gate, GREEN gate), lint, scenarios. Any teammate may ask you to run a gate. Execute the requested cargo command and message back PASS or FAIL with the full output. Also message orchestrator with the result for session-state." })
```

Verification tier (8 agents) — same pattern, subagent_type per table above.

On-demand consultants (9 agents) — same pattern, subagent_type per table above, each with:
```
prompt: "Read docs/todos/detail/phantom-breaker.md. Read ~/.claude/teams/breaker-team/config.json.
Answer questions from any teammate. Do not initiate. Escalate to orchestrator only for genuine new decisions."
```

After all `Agent` calls complete, the team is idle. Trigger Wave 1:
```
SendMessage(to: "planning-writer-specs-tests", message: "Wave 1: <description>. Scope: <...>. Decisions: <...>. Write test spec to .claude/specs/wave1-phantom-breaker-tests.md.", summary: "wave 1 start")
```

---

### Wave flow with the team

**Orchestrator starts wave** (SendMessage to planning-writer-specs-tests with scope + decisions).

**Peer loop — test spec (no orchestrator):**
- `planning-writer-specs-tests` writes → messages `planning-reviewer-specs-tests`
- Revision loop runs peer-to-peer
- `planning-reviewer-specs-tests` messages `writer-tests` + orchestrator: "approved"

**Orchestrator routes writer-tests output:**
- Sends test file path to `reviewer-tests` for review
- Then: `SendMessage(to: "runner-cargo", message: "run RED gate — tests at <path>", summary: "RED gate wave N")`

**Runner reports back:**
- PASS: orchestrator messages `planning-writer-specs-code` with failing test paths
- FAIL (compile): orchestrator messages `writer-tests` with error

**Peer loop — code spec (no orchestrator):**
- `planning-writer-specs-code` writes → messages `planning-reviewer-specs-code` → revision loop → approved
- `planning-reviewer-specs-code` messages `writer-code` + orchestrator

**GREEN phase:**
- `writer-code` implements → messages orchestrator: "done, run GREEN gate"
- `SendMessage(to: "runner-cargo", message: "run GREEN gate", summary: "GREEN gate wave N")`
- Runner reports: PASS → orchestrator updates session-state, routes to verification tier
- Runner reports: FAIL → orchestrator forwards to writer-code → writer-code → debugger loop
- Circuit-break (3 attempts): `debugger` → orchestrator for human input

---

### What the orchestrator owns

| Orchestrator always does | Team handles |
|---|---|
| Start each wave (SendMessage to planning-writer-specs-tests) | Spec writing + revision loops |
| Request cargo runs (SendMessage to runner-cargo) | Test writing + spec clarifications |
| Update session-state after every notification | GREEN phase implementation |
| Trigger verification tiers (SendMessage to reviewer-*) | GREEN failure diagnosis + fix |
| Human judgment: new design decisions, wave scope | Design/arch checks via on-demand consultants |
| Circuit-break escalation (3 attempts) | In-wave peer communication |

### Session-state notes for this trial

Update session-state after each notification from a teammate, exactly as with background agents. Many columns that normally require orchestrator round-trips (spec revision status, test-writer handoff, implementer briefing) are now peer-to-peer and only surface to session-state at phase completion points (spec approved, RED gate, GREEN gate). The runner's PASS/FAIL report is the primary trigger for gate-column updates.

---

## Problem

Phantom breakers today are broken: invisible (no mesh/material/draw layer), lack the `Breaker` marker (so `With<Breaker>` queries miss them), use raw `Width`/`Height` (bypass SizeBoost), fall back to hardcoded `DEFAULT_PHANTOM_BASE_*` constants when components are missing, use a parallel `PhantomBreakerLifetime` type instead of a shared lifespan component, duplicate bolt-breaker bounce logic in `afterimage_check_phantom_bounce`, and get spawned via raw `commands.spawn((...))` that bypasses the breaker builder entirely.

Nine failure modes documented in `audit/builders/breaker_phantom.md` and `audit/protocols/afterimage.md` all resolve via a single transition: make `Breaker::builder().phantom(...)` the one spawn path.

## Scope

**In scope: phantom breaker alone.** Phantom bolts (`bolt-builder-phantom-transition.md`, `phantom-bolt-mutate-real-bolt.md`, `phantom-bolt-collision-semantics-tests.md`, `placeholder-vfx-for-deferred-items.md`) stay as their own remediations. This TODO introduces the *shared* phantom infra (`Lifespan`, `PhantomFlicker`, `tick_phantom_flicker`) because phantom breakers need it; phantom bolts will adopt the same infra later without re-introducing it.

## Design

### Phantom-breaker contract

A phantom breaker:

- **IS a `Breaker`.** Carries the `Breaker` marker. Existing `With<Breaker>` queries MUST see it. No parallel `Or<(With<Breaker>, With<PhantomBreaker>)>` queries.
- **Has fixed position.** Spawned at the real breaker's location (typically at a Perfect Bump moment). Horizontal movement input is gated out; does not slide, does not dash.
- **Receives bump input.** Participates in `update_bump` → `grade_bump` → `BumpPerformed`. A single `InputActions::Bump` press opens windows on ALL breakers (real + phantoms), each grading independently against its own `BumpState`.
- **Bounces the bolt.** `bolt_breaker_collision` reflects bolts off phantoms via `With<Breaker>`. Real-breaker-only side effects (tilt, spread override, last-impact offset, piercing-bolt handling) gate via `Without<PhantomBreaker>`.
- **Always has a lifespan.** `.phantom(...)` takes a required `lifespan: f32` field (unlike phantom bolts where lifespan is optional).
- **Always despawns on expiry.** No `LifetimeEndBehavior` enum for breakers — unconditional despawn at lifespan end. `LifetimeEndBehavior` stays bolt-only.
- **Is visually distinct when rendered.** Tinted material + `PhantomFlicker` placeholder. Headless phantoms skip visual components.
- **Cleanup orthogonal to phantom-ness.** `.phantom()` does NOT force `.primary()` or `.extra()`. Afterimage spawns phantoms as `.extra()` so they clean up at node exit; that's the caller's choice, not the method's.

### Builder API

Add `.phantom(BreakerPhantomParams)` as an **optional chainable method** on the breaker builder (alongside `with_lives`, `with_effects`, `with_color`). Works on any typestate — stashes params in `OptionalBreakerData` for the terminal to consume.

```rust
// breaker-game/src/breaker/builder/core/types.rs
pub(crate) struct OptionalBreakerData {
    // ... existing fields ...
    pub(crate) phantom: Option<BreakerPhantomParams>,
}

#[derive(Debug, Clone, Copy)]
pub struct BreakerPhantomParams {
    /// Lifespan in seconds. REQUIRED — every phantom breaker has a lifespan.
    pub lifespan: f32,
    /// Color tint mixed with the base breaker color (e.g., pale cyan).
    pub phantom_color_rgb: [f32; 3],
    /// Flicker frequency in Hz (placeholder for Phase 5 VFX).
    pub flicker_frequency: f32,
    /// Minimum alpha during flicker.
    pub flicker_min_alpha: f32,
}
```

```rust
// breaker-game/src/breaker/builder/core/transitions.rs
impl<D, Mv, Da, Sp, Bm, V, R> BreakerBuilder<D, Mv, Da, Sp, Bm, V, R> {
    #[must_use]
    pub const fn phantom(mut self, params: BreakerPhantomParams) -> Self {
        self.optional.phantom = Some(params);
        self
    }
}
```

### Terminal behavior

When `optional.phantom.is_some()`, the terminal (on top of the standard rendered/headless breaker it already produces) additionally:

1. Inserts `PhantomBreaker` marker.
2. Inserts `Lifespan { remaining: params.lifespan }`.
3. When visual typestate is `Rendered`: mixes `phantom_color_rgb` into the spawned material and inserts `PhantomFlicker { frequency, min_alpha }`.
4. When visual typestate is `Headless`: skips material mix and `PhantomFlicker` insert.

The default `CollisionLayers { layer: BREAKER_LAYER, mask: BOLT_LAYER }` applies unchanged — phantoms participate in the quadtree identically to real breakers.

### Gating in existing systems

Mechanical rule: systems whose effects are real-breaker-only add `Without<PhantomBreaker>` to their `With<Breaker>` queries. Systems where phantom participation is intentional stay untouched.

| System | File | Gate |
|--------|------|------|
| `move_breaker` horizontal velocity | `breaker/systems/move_breaker/` | `Without<PhantomBreaker>` |
| Dash-input transition | dash input system | `Without<PhantomBreaker>` |
| `perfect_bump_dash_cancel` | `breaker/systems/bump/` | `Without<PhantomBreaker>` |
| `update_breaker_state`, `trigger_bump_visual`, `animate_bump_visual`, `animate_tilt_visual`, `sync_breaker_scale`, `breaker_cell_collision`, `breaker_wall_collision`, node-reset systems | various | `Without<PhantomBreaker>` |
| `bolt_breaker_collision` real-breaker-only branches (tilt, spread override, last-impact offset, piercing-bolt) | `bolt/systems/bolt_breaker_collision/system.rs` | inline `Without<PhantomBreaker>` branches within the collision response |
| `bolt_breaker_collision` core reflection | same | **no gate** — phantoms bounce bolts |
| `update_bump` | `breaker/systems/bump/system.rs` | **no gate** — phantoms receive bump input |
| `grade_bump` | `breaker/systems/bump/system.rs` | **no gate** — but see migration below |

### CRITICAL: `grade_bump` `.single_mut()` → `.iter_mut()` migration

`grade_bump` currently calls `bump_query.single_mut()` — asserts exactly one breaker. As soon as a phantom spawns, this panics. Migration is mandatory:

```rust
// Before
let (entity, mut bump_state) = bump_query.single_mut();
// process messages targeting this one breaker

// After
for (entity, mut bump_state) in &mut bump_query {
    // match incoming BoltImpactBreaker messages by breaker entity:
    //   if msg.breaker == entity { grade using bump_state }
}
```

Each breaker (real or phantom) grades against its own `BoltImpactBreaker` messages using its own `BumpState`. No cross-referencing between breakers. Write `BumpPerformed { grade, bolt, breaker: entity }` with the per-breaker entity so downstream consumers can distinguish hits on phantoms vs real.

This migration has a regression-test requirement: spawn a phantom, fire a bolt, assert no panic. See tests #8 below.

### Lifespan-end dispatch via unified death pipeline

Phantom breakers integrate with the death pipeline introduced in TODO #0 (`rantzsoft_dmg`). The pipeline defines `DespawnEntity { entity: Entity }` as the single despawn primitive consumed by `process_despawn_requests` in `FixedPostUpdate`. This is the one authorized despawn path.

**`KillYourself<Breaker>` cannot be used.** `handle_breaker_death` (the specialized Breaker kill handler) emits `RunLost` on every `KillYourself<Breaker>` — a phantom expiring would end the player's run. Phantom death is not a run-ending event.

The lifespan-tick system emits `DespawnEntity` directly:

```rust
fn tick_phantom_breaker_lifespan(
    time: Res<Time<Fixed>>,
    mut query: Query<(Entity, &mut Lifespan), (With<Breaker>, With<PhantomBreaker>)>,
    mut despawn_writer: MessageWriter<DespawnEntity>,
) {
    let dt = time.delta_secs();
    for (entity, mut lifespan) in &mut query {
        lifespan.remaining -= dt;
        if lifespan.remaining <= 0.0 {
            despawn_writer.write(DespawnEntity { entity });
        }
    }
}
```

Runs in `FixedUpdate`, ordered before `DeathPipelineSystems::ProcessDespawn`. No `KillYourself<Breaker>`, no `Destroyed<Breaker>` (phantom death does not fire the effect bridges wired for real-breaker death), no `RunLost`. The pipeline's single despawn system removes the entity.

The `With<PhantomBreaker>` filter is mandatory — without it, a real breaker carrying a `Lifespan` (if one ever exists) would silently despawn mid-run. Real breakers don't carry `Lifespan` today and the design does not add one, but the filter makes the contract explicit.

### Shared phantom infrastructure

Introduced by this TODO, reused by phantom bolts later:

- **`Lifespan { remaining: f32 }`** component. Shared type, canonical location under `shared::lifespan::` (or equivalent).
- **`PhantomFlicker { frequency, min_alpha }`** component. Shared type, canonical location under `shared::phantom::` (or equivalent).
- **`tick_phantom_flicker`** FX system. Iterates entities with `PhantomFlicker`, modulates material alpha. Runs in `Update`. Registered once by `fx::plugin`.

**Not shared**:

- `LifetimeEndBehavior` — bolt-only (phantoms don't need the enum; they always despawn).
- Lifespan-tick dispatchers — breakers get `tick_phantom_breaker_lifespan`; bolts will get their own separately.

### Afterimage spawn path migration

Open `breaker-game/src/mutators/protocols/afterimage/system/spawn_phantom_breaker.rs`. Replace the raw `commands.spawn((...))` with the builder, reading the real breaker's runtime components (width, height, movement, dash, spread, bump, color) so the phantom matches current state (post-SizeBoost, post-any-other-modifier):

```rust
Breaker::builder()
    .dimensions(real.width, real.height, real.y_position)
    .movement(real.movement_settings)
    .dashing(real.dash_settings)
    .spread(real.spread_degrees)
    .bump(real.bump_settings)
    .with_color(real.color_rgb)
    .phantom(BreakerPhantomParams {
        lifespan:          config.phantom_duration,
        phantom_color_rgb: [0.4, 0.8, 1.0],
        flicker_frequency: 4.0,
        flicker_min_alpha: 0.3,
    })
    .extra()
    .rendered(&mut meshes, &mut materials)
    .spawn(&mut commands);
```

Spawn system gains `ResMut<Assets<Mesh>>` + `ResMut<Assets<ColorMaterial>>` params (currently missing).

Global-singleton enforcement (despawn existing phantoms before spawning a new one) stays as-is — the despawn loop runs before the builder call.

### Code deletions

- `DEFAULT_PHANTOM_BASE_WIDTH` / `DEFAULT_PHANTOM_BASE_HEIGHT` constants — builder's dimension path covers this.
- `afterimage_check_phantom_bounce` system + file + 880-line `tests/check_phantom_bounce.rs` — the phantom is a `Breaker` now; `bolt_breaker_collision` handles the bounce.
- `PhantomBreakerLifetime` type — replaced by shared `Lifespan`.
- Afterimage entry in `docs/architecture/plugins.md` Velocity2D exception registry — the raw `Velocity2D` writes vanish with `check_phantom_bounce`; the exception can be narrowed or removed in lockstep.

## Tests to author

1. `.phantom(...)` on a `Rendered` terminal inserts: `Breaker`, `PhantomBreaker`, `PhantomFlicker`, `Lifespan`, `Mesh2d`, `MeshMaterial2d`, `GameDrawLayer::Breaker`, `Width`, `Height`, `BumpState`, `HasBump`, `CollisionLayers`.
2. `.phantom(...)` on a `Headless` terminal inserts: `Breaker`, `PhantomBreaker`, `Lifespan`, `Width`, `Height`, `BumpState`, `HasBump`, `CollisionLayers`. No `PhantomFlicker`, `Mesh2d`, or `MeshMaterial2d`.
3. `With<Breaker>` queries include the phantom.
4. Movement input drives the real breaker but NOT the phantom.
5. Dash input triggers dash on the real breaker but NOT the phantom.
6. Bump input triggers bump on BOTH the real breaker and the phantom (no gating on bump).
7. Bolt→phantom-breaker collision produces a standard velocity reflection (no tilt, no spread override, no last-impact offset, no piercing-bolt handling).
8. **Regression:** `grade_bump` runs without panicking when a phantom is present (`.single_mut()` → `.iter_mut()` migration).
9. `grade_bump` grades a bump against the phantom's own `BumpState`; `BumpPerformed { breaker: <phantom entity> }` is emitted when a bolt hits the phantom within the window.
10. Phantom with `BreakerPhantomParams { lifespan: t, ... }` emits `DespawnEntity { entity }` and is removed by the death pipeline when `Lifespan` reaches 0.
11. `CleanupOnExit<NodeState>` despawns an `.extra()` phantom on `OnExit(NodeState::Playing)` regardless of remaining lifespan.
12. Phantom expiry does NOT emit `KillYourself<Breaker>`, `Destroyed<Breaker>`, or `RunLost`.
13. Rendered phantom's material color differs from real breaker's base color (phantom tint mixed in).
14. `perfect_bump_dash_cancel` does NOT cancel the real breaker's dash when a Perfect is registered on a phantom-only hit. (Design call per `delete-check-phantom-bounce.md` — default is "phantom perfect does not cancel real dash.")
15. Spawning a phantom when the real breaker has `Width(150.0)` + `Height(30.0)` produces a phantom with the same dimensions (reads live components, not definitions).

## Code change summary

| File | Change |
|------|--------|
| `breaker/builder/core/types.rs` | Add `phantom: Option<BreakerPhantomParams>` to `OptionalBreakerData`; add `BreakerPhantomParams` struct. |
| `breaker/builder/core/transitions.rs` | Add `.phantom(...)` optional-chainable method. |
| `breaker/builder/core/terminal.rs` | Phantom-params integration: color mix when Rendered, marker + `Lifespan` + `PhantomFlicker` insert. |
| `breaker/systems/bump/system.rs` | `grade_bump` migration `.single_mut()` → `.iter_mut()`; match `BoltImpactBreaker` by breaker entity. |
| `breaker/systems/move_breaker/system.rs` | Add `Without<PhantomBreaker>` to horizontal-velocity query. |
| Dash input system | Add `Without<PhantomBreaker>` to dash-transition query. |
| `breaker/systems/bump/perfect_bump_dash_cancel.rs` (or wherever) | Add `Without<PhantomBreaker>`. |
| `bolt/systems/bolt_breaker_collision/system.rs` | Narrow `Without<PhantomBreaker>` branches for tilt, spread override, last-impact offset, piercing bolt; core reflection stays on `With<Breaker>`. |
| `breaker/systems/update_breaker_state`, `trigger_bump_visual`, `animate_bump_visual`, `animate_tilt_visual`, `sync_breaker_scale`, `breaker_cell_collision`, `breaker_wall_collision`, node-reset systems | Add `Without<PhantomBreaker>` where they operate on the real breaker only. |
| `breaker/systems/tick_phantom_breaker_lifespan.rs` | New system. `FixedUpdate`, before `DeathPipelineSystems::ProcessDespawn`. Emits `DespawnEntity`. |
| `shared::phantom` (or `shared::lifespan`) | New shared module: `Lifespan` + `PhantomFlicker` components. |
| `fx::plugin` | Register `tick_phantom_flicker`. |
| `mutators/protocols/afterimage/system/spawn_phantom_breaker.rs` | Migrate to builder; add mesh/material asset params; read real breaker's runtime components. |
| `mutators/protocols/afterimage/system/check_phantom_bounce.rs` | **DELETED.** |
| `mutators/protocols/afterimage/system/register.rs` | Remove `check_phantom_bounce` registration. |
| `mutators/protocols/afterimage/system/components.rs` | Remove `PhantomBreakerLifetime`. |
| `breaker-game/src/mutators/protocols/afterimage/tests/check_phantom_bounce.rs` | **DELETED** (880 lines). |
| `docs/architecture/plugins.md` | Narrow/remove afterimage entry from Velocity2D exception registry. |

## Dependencies

- **TODO #0 (unified death pipeline crate)** — `DespawnEntity` message, `DeathPipelineSystems::ProcessDespawn` set, and `process_despawn_requests` system all move to the crate. `tick_phantom_breaker_lifespan` writes the crate's message. Must land after #1.
- **TODO #1 (mutators domain refactor)** — afterimage moves to `mutators/protocols/afterimage/`. The phantom-builder migration touches files under the new path. Must land after #2.

## Ordering

Lands after TODO #0 and TODO #1. Independent of TODO #2 (greed skip) and TODO #3 (bolt-loss behavior) — can interleave with them.

Companion: `bolt-builder-phantom-transition.md` shares infra (`Lifespan`, `PhantomFlicker`, `tick_phantom_flicker`). Not bundled into this TODO per scope decision, but once this lands the bolt side is a smaller follow-up (builder method + bolt-specific lifespan tick; shared infra already in place).

## Subsumes

- `audit/remediations/phantom-breaker-dimension-fallback.md` — builder reads live `Width`/`Height` components; `DEFAULT_PHANTOM_BASE_*` constants deleted.
- `audit/remediations/delete-check-phantom-bounce.md` — `afterimage_check_phantom_bounce` deleted; phantoms flow through `bolt_breaker_collision` with `Without<PhantomBreaker>` branches.
- `audit/remediations/breaker-builder-phantom-transition.md` — this file IS that spec, expanded and lifted to a TODO.

## Scope boundary

In scope:
- Breaker builder `.phantom()` method + terminal integration
- Gating `Without<PhantomBreaker>` across real-breaker-only systems
- `grade_bump` `.single_mut()` → `.iter_mut()` migration
- Shared `Lifespan` + `PhantomFlicker` components
- `tick_phantom_breaker_lifespan` dispatcher
- Afterimage spawn-path migration to builder
- Deletions: `check_phantom_bounce`, `DEFAULT_PHANTOM_BASE_*`, `PhantomBreakerLifetime`

Out of scope:
- Phantom bolts (separate remediations)
- Phantom bolt mutate-real-bolt logic
- Phase 5 VFX polish for phantom visuals (placeholder flicker only)
- Changes to when/why afterimage spawns phantoms (the mechanic stays unchanged; only the spawn *mechanism* changes)

## TODO entry

> **[ready / BLOCKED by #1, #2]** Phantom breaker — builder transition `Breaker::builder().phantom(BreakerPhantomParams { lifespan, ... })`, shared `Lifespan` + `PhantomFlicker` infra, `grade_bump` `.single_mut()` → `.iter_mut()` migration (panics on phantom spawn otherwise), lifespan dispatch via `DespawnEntity` (not `KillYourself<Breaker>` which would trigger `RunLost`), `Without<PhantomBreaker>` gating across movement/dash/tilt systems, afterimage spawn path migrated to builder, `afterimage_check_phantom_bounce` deleted (880-line test file too). Subsumes `breaker-builder-phantom-transition.md`, `phantom-breaker-dimension-fallback.md`, `delete-check-phantom-bounce.md`. Depends on #1 (`DespawnEntity`) and #2 (afterimage relocation). — [detail](detail/phantom-breaker.md)
