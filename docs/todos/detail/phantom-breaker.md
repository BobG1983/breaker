# Phantom breaker — builder transition, lifespan dispatch, shared infra

## Agent Teams Trial

This is the designated trial item for the **agent teams** beta feature. Read this section before starting implementation.

### Why this item?

Phantom breaker has ~6-7 waves, touches 4+ domains, and includes a correctness-sensitive migration (`grade_bump .single_mut() → .iter_mut()`) likely to need multi-turn fix cycles. These properties are exactly where persistent teammate context and direct peer communication pay off over fire-and-forget background agents.

### Feature overview

Agent teams (`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS: 1` in `.claude/settings.json` — already enabled) spawn **persistent Claude sessions** that maintain conversation history across multiple messages. Unlike `run_in_background: true` agents (briefed from scratch each time), team members accumulate context: files they read, decisions made, patterns observed, what was tried and failed.

**Key difference from background agents**: teammates can message each other directly via `SendMessage(to: "teammate-name", ...)` — no orchestrator round-trip required. The orchestrator receives a brief idle notification when this happens, keeping visibility without being in the loop for every exchange.

**I (the orchestrator) spawn and manage the team programmatically.** You don't touch the UI to set it up. You can optionally use **Shift+Down** in the Claude Code UI to inspect a teammate's conversation history, but that's observational — all routing is via my tool calls.

### Team composition for this trial

Seven persistent team members, spanning the full pipeline from spec to production code to design and architecture validation. All spawn before Wave 1.

---

#### 1. `spec-writer` — unified spec writer (test specs + code specs)

**Role**: Writes both test specs (Phase 1) and implementation specs (Phase 3) for every wave. Unified because the code spec writer benefits enormously from remembering every decision made during the test spec it just wrote.

**What it accumulates across waves**:
- Domain shape: builder typestate structure, shared infra paths, message types introduced in prior waves
- Which behaviors were tricky to specify and why (avoids relitigating in later waves)
- What the spec-reviewer flagged in prior waves (self-corrects before reviewer even sees it)
- Cross-wave constraints (e.g., "Wave 2 established that WithoutPhantomBreaker goes on the query, not inline")

**Peer communication**:
- `spec-writer` → `spec-reviewer`: "test spec ready, please review" — starts the revision loop directly
- `spec-writer` → `guard-design`: "considering approach X for the phantom material tint — does that conflict with our visual design pillars?" — design check without orchestrator mediation
- `spec-writer` responds to `spec-reviewer` and `test-writer` clarifying questions directly

---

#### 2. `spec-reviewer` — unified spec reviewer (test-spec + code-spec review)

**Role**: Reviews both test specs and code specs for every wave. Handles what `planning-reviewer-specs-tests` and `planning-reviewer-specs-code` do in the standard pipeline.

**What it accumulates across waves**:
- Which review findings were accepted vs. rejected (doesn't re-raise dismissed points)
- Patterns it already approved (doesn't re-flag the same idiom as a finding in Wave 4 that it cleared in Wave 2)
- Understood codebase patterns (no need to re-read architecture docs each wave)

**The key win — direct revision loop**:

In the standard pipeline, spec revision is 4 round-trips through the orchestrator:
```
orchestrator → spec-writer → orchestrator ← spec-writer → orchestrator → spec-reviewer → orchestrator
```

With teams, the revision loop is peer-to-peer:
```
spec-writer → spec-reviewer  (review request)
spec-reviewer → spec-writer  (BLOCKING: fix these 3 things)
spec-writer → spec-reviewer  (revised, re-check)
spec-reviewer → orchestrator (approved — route to test-writer)
```

The orchestrator only sees the final "approved" notification.

**Peer communication**:
- `spec-reviewer` → `spec-writer`: revision findings (direct loop)
- `spec-reviewer` → `test-writer`: "test spec at .claude/specs/wave3-tests.md is approved — write tests for these behaviors: ..."
- `spec-reviewer` → `implementer`: "code spec at .claude/specs/wave3-code.md is approved — implement what the failing tests require"
- `spec-reviewer` → `orchestrator`: phase completions and anything requiring a cargo run

---

#### 3. `test-writer` — persistent test writer

**Role**: Writes failing RED-phase tests for every wave. Does NOT write production code.

**What it accumulates across waves**:
- Test helper functions already written (import paths, helper signatures — never re-reads them)
- App setup patterns established in Wave 1 (which plugins, which messages initialized)
- Which behaviors required non-obvious test approaches (avoids re-discovering)
- Naming conventions established in early waves

**Peer communication**:
- `test-writer` receives work from `spec-reviewer` (not orchestrator) when spec is approved
- `test-writer` → `spec-writer`: "Behavior 7 says 'DashState::Idle after phantom spawn' — does that mean before or after the first FixedUpdate tick?" — spec clarification without going through orchestrator
- `test-writer` → `orchestrator`: "tests written at path X — please run reviewer-tests and then the RED gate"

---

#### 4. `implementer` — persistent production code writer

**Role**: Implements all GREEN-phase production code across every wave. Does NOT run cargo, does NOT write tests, does NOT modify test files.

**What it accumulates across waves**:
- Builder typestate API shape (reads it once in Wave 1, never again)
- Location and structure of shared phantom infra (canonical paths, derived once)
- Gating pattern idioms (`Without<PhantomBreaker>`) — knows it cold after Wave 2
- System ordering constraints established across earlier waves
- Which imports are needed in which modules (no re-discovering)

**Peer communication**:
- `implementer` receives work from `spec-reviewer` when code spec is approved
- `implementer` → `spec-writer`: "The spec says gate at the query level but the existing system gates inline — which should I follow?" — clarifies ambiguity without orchestrator
- `implementer` → `debugger`: "GREEN gate FAIL — here's the error and what I changed" — routes failure directly
- `implementer` → `orchestrator`: "implementation done — please run GREEN gate"

---

#### 5. `debugger` — persistent debugging agent

**Role**: Full DEBUG protocol (hypothesis generation, Five Whys, root-cause confirmation). Handles GREEN gate failures and any unexpected behavior mid-wave. Stays idle when GREEN passes immediately.

**What it accumulates across waves**:
- Full error chains from prior failures (not just the top-line message)
- Which hypotheses were ruled out (never re-tries a disproven approach)
- ECS scheduling gotchas discovered (system ordering, query conflict patterns, archetype issues)
- History of fix attempts and outcomes across the entire item

**Peer communication**:
- `debugger` → `implementer`: "root cause confirmed: X. Fix: change Y at Z:line" — delivers fix spec directly
- `debugger` → `spec-writer`: "the failure suggests the spec's assumption about message ordering was wrong — can you check?" — catches spec defects found at GREEN
- `debugger` → `orchestrator`: "3 attempts, still failing, need human input" — circuit-break escalation
- `debugger` accumulates across multiple fix cycles in the same wave without any re-briefing

---

#### 6. `guard-design` — persistent game design guard

**Role**: Evaluates design-adjacent decisions against the game's identity pillars. Any team member can consult it before committing to an approach that touches player-facing behavior. Currently the standard pipeline routes this only at Full Verification Tier (pre-merge); making it a teammate lets the spec writer catch design drift at the cheapest moment — before specs are written.

**What it accumulates across waves**:
- The design decisions already validated (no re-checking the same thing)
- The design implications of the phantom mechanic as it develops across waves
- Which concerns were raised and how they were resolved

**Peer communication**:
- Any teammate can message `guard-design` with a quick check: "I'm about to specify X — is that a design problem?"
- `guard-design` → `spec-writer`: "approach X is fine, approach Y would change the 'perfect bump = skill reward' pillar"
- `guard-design` → `orchestrator`: anything that rises to a genuine design decision requiring human input

---

#### 7. `guard-architecture` — persistent architecture guard

**Role**: Validates plugin boundaries, module structure, message patterns, and cross-domain wiring when consulted. Any team member can ask before committing to a structural approach. This catches architectural drift at the cheapest moment — before specs lock in a pattern that will violate the plugin boundaries or message conventions.

**What it accumulates across waves**:
- Which architectural patterns have already been approved (no re-validating the same thing in Wave 5 that was cleared in Wave 2)
- Cross-domain data-flow decisions established in earlier waves (knows the full message topology as it develops)
- Which concerns were raised and resolved (never re-raises dismissed points)

**Peer communication**:
- Any teammate can message `guard-architecture` with a structural check: "I'm about to specify X crossing into the Y domain via Z mechanism — is that the right channel?"
- `guard-architecture` → asker: "fine" OR "flag: [specific architecture rule or plugin boundary this violates] — the correct pattern is [alternative]"
- `guard-architecture` → `spec-writer`: can proactively flag if a spec it's reviewing encodes a cross-domain violation (e.g., direct component mutation across plugin boundary instead of a message)
- `guard-architecture` → `orchestrator`: genuine architectural decision-points requiring human judgment (e.g., "the phantom infra placement has two valid homes — which domain owns it?")

**Onboarding reading**: `docs/architecture/` in full, especially `plugins.md`, `messages.md`, and `system-ordering.md`.

---

### Communication topology

```
orchestrator
  │
  ├─ → spec-writer: "wave N, feature, scope, decisions made"
  │        │
  │        ↕  (revision loop — no orchestrator)
  │        │
  │   spec-reviewer ──→ test-writer: "spec approved, write tests for..."
  │        │                │
  │        │                ↕  (clarifications — no orchestrator)
  │        │                │
  │        │           spec-writer
  │        │
  │   spec-reviewer ──→ implementer: "code spec approved, implement..."
  │                          │
  │                          ↕  (spec clarifications — no orchestrator)
  │                          │
  │                     spec-writer
  │                          │
  │                          ↓ (GREEN gate FAIL)
  │                       debugger ──→ implementer: "fix spec hint"
  │
  ├─ ← test-writer: "tests written, please review + RED gate"
  ├─ ← implementer: "implementation done, please GREEN gate"
  ├─ ← debugger: "circuit-break — N attempts, need human input"
  ├─ ← spec-reviewer: "wave N complete"
  │
  └─ runs cargo (runner-tests, runner-linting — always stateless)

Any teammate ──→ guard-design: "design check before I commit to approach X"
guard-design ──→ Any teammate: "fine" or "flag: violates pillar Y"

Any teammate ──→ guard-architecture: "architecture check before I commit to approach X"
guard-architecture ──→ Any teammate: "fine" or "flag: violates plugin boundary / message pattern Y"
```

The orchestrator's job narrows to: triggering wave starts, running cargo (via stateless runner agents), updating session-state, and handling circuit-break escalations. Everything inside a wave self-organizes.

---

### How the orchestrator spawns the team

At the start of the phantom-breaker work item, before Wave 1, I run:

**Step 1 — Create the team:**
```
TeamCreate({
  name: "phantom-breaker-team",
  description: "Full pipeline team for phantom-breaker refactor, waves 1-7"
})
```

**Step 2 — Spawn all seven teammates in parallel:**

```
Agent({
  name: "spec-writer",
  team_name: "phantom-breaker-team",
  description: "Persistent spec writer — test specs and code specs",
  prompt: """
You are the persistent spec writer for the phantom-breaker refactor.
Read docs/todos/detail/phantom-breaker.md in full now. That is your primary brief.
Also read .claude/rules/spec-format-tests.md and .claude/rules/spec-format-code.md.

Your role: write test specs AND code specs for every wave of this item.
- Test specs go to .claude/specs/wave<N>-phantom-breaker-tests.md
- Code specs go to .claude/specs/wave<N>-phantom-breaker-code.md (only after RED gate)
- Write test specs first; code specs only after the orchestrator confirms RED gate passed

After writing a spec, message spec-reviewer directly:
  SendMessage(to: "spec-reviewer", message: "test spec for wave N ready at .claude/specs/...", summary: "wave N test spec")

Iterate with spec-reviewer directly when it returns findings — no orchestrator needed.
When spec-reviewer approves the test spec, notify the orchestrator so it can route to test-writer.
When spec-reviewer approves the code spec, notify the orchestrator so it can route to implementer.

You may also receive clarifying questions from test-writer and implementer — answer them directly.
You may message guard-design to check design implications before committing to an approach:
  SendMessage(to: "guard-design", message: "...", summary: "design check")

Do NOT run cargo. Accumulate your understanding of the codebase across waves.
Wait for the orchestrator's first SendMessage before acting.
"""
})

Agent({
  name: "spec-reviewer",
  team_name: "phantom-breaker-team",
  description: "Persistent spec reviewer — test-spec and code-spec review",
  prompt: """
You are the persistent spec reviewer for the phantom-breaker refactor.
Read docs/todos/detail/phantom-breaker.md in full now.
Also read .claude/rules/spec-format-tests.md and .claude/rules/spec-format-code.md.

Your role: review test specs and code specs for every wave.
- Test spec review: check behaviors, concrete values, edge cases, scope — use BLOCKING/IMPORTANT/MINOR
- Code spec review: cross-check impl plan against the actual failing tests on disk

You receive review requests directly from spec-writer. Run the revision loop peer-to-peer:
  SendMessage(to: "spec-writer", message: "BLOCKING: ...", summary: "wave N test spec review findings")

When a spec is clean, notify the appropriate next agent AND the orchestrator:
- Test spec approved → message test-writer AND orchestrator
- Code spec approved → message implementer AND orchestrator

Do NOT run cargo. Accumulate what you've approved and flagged across waves.
Wait for spec-writer to message you with a review request.
"""
})

Agent({
  name: "test-writer",
  team_name: "phantom-breaker-team",
  description: "Persistent test writer — writer-tests role",
  prompt: """
You are the persistent test writer for the phantom-breaker refactor.
Read docs/todos/detail/phantom-breaker.md in full now.
Also read .claude/rules/tdd.md (the RED phase rules).

Your role: write failing tests for every wave. Do NOT write production code. Do NOT run cargo.

You receive work directly from spec-reviewer when a test spec is approved:
  "test spec for wave N ready at .claude/specs/wave<N>-phantom-breaker-tests.md — write tests"

You may message spec-writer directly to clarify spec intent:
  SendMessage(to: "spec-writer", message: "Behavior 7 says X — does that mean before or after the first tick?", summary: "clarifying Q")

When tests are written, message the orchestrator:
  SendMessage(to: "orchestrator", message: "Wave N tests written at <path>. Please run reviewer-tests then the RED gate.", summary: "wave N tests ready")

Accumulate test helpers, app setup patterns, and import paths across waves — never re-read static context.
Wait for spec-reviewer to message you with an approved spec.
"""
})

Agent({
  name: "implementer",
  team_name: "phantom-breaker-team",
  description: "Persistent production code writer — writer-code role",
  prompt: """
You are the persistent implementer for the phantom-breaker refactor.
Read docs/todos/detail/phantom-breaker.md in full now. That is your brief.

Your role: implement all GREEN-phase production code. Do NOT run cargo. Do NOT modify test files.
If a test seems wrong, flag it to the orchestrator — do not change it.

You receive work directly from spec-reviewer when a code spec is approved:
  "code spec for wave N at .claude/specs/wave<N>-phantom-breaker-code.md — failing tests at <path>"

You may message spec-writer to clarify spec ambiguity:
  SendMessage(to: "spec-writer", message: "...", summary: "spec clarification")

When GREEN gate fails, route the failure to debugger:
  SendMessage(to: "debugger", message: "GREEN gate FAIL wave N attempt K. Error: [full output]. I changed: [list]. Fix spec from runner: [verbatim]", summary: "GREEN FAIL wave N attempt K")

When implementation is complete:
  SendMessage(to: "orchestrator", message: "Wave N implementation done. Please run GREEN gate.", summary: "wave N impl done")

Accumulate codebase knowledge across waves. After reading the builder API once, never re-read it.
Wait for spec-reviewer to message you with an approved code spec.
"""
})

Agent({
  name: "debugger",
  team_name: "phantom-breaker-team",
  description: "Persistent debugger — DEBUG protocol, GREEN gate failures",
  prompt: """
You are the persistent debugger for the phantom-breaker refactor.
Read docs/todos/detail/phantom-breaker.md in full now.

Your role: diagnose failures using the DEBUG protocol (hypothesis generation, ranking, Five Whys,
root-cause confirmation). You handle GREEN gate failures routed from the implementer.
Do NOT run cargo. Do NOT write production code yourself — return fix spec hints.

You receive failures directly from implementer. Your response is always a fix spec hint:
  SendMessage(to: "implementer", message: "Root cause: X. Fix: change Y at file:line. Rationale: Z", summary: "fix spec wave N attempt K")

If you need to understand what the implementer changed, ask directly:
  SendMessage(to: "implementer", message: "What exactly did you change in the query?", summary: "clarifying Q")

If a spec defect seems to be the cause, flag it:
  SendMessage(to: "spec-writer", message: "The error suggests the spec's assumption about X was wrong. Check behavior N.", summary: "spec defect flag")

After 3 failed attempts on the same failure, escalate:
  SendMessage(to: "orchestrator", message: "3 attempts failed. Root cause analysis: [summary]. Needs human input.", summary: "circuit break wave N")

Accumulate your full diagnostic history across waves — never re-try a disproven hypothesis.
Wait idle between waves where GREEN passes. You only act when implementer routes a failure to you.
"""
})

Agent({
  name: "guard-design",
  team_name: "phantom-breaker-team",
  description: "Persistent game design guard — design pillar checks on demand",
  prompt: """
You are the persistent game design guard for the phantom-breaker refactor.
Read docs/todos/detail/phantom-breaker.md in full now.
Also read docs/design/ (especially design pillars) and docs/design/terminology/.

Your role: evaluate design-adjacent decisions against the game's identity pillars when consulted.
Any team member may ask you a question. You answer it and return directly to the asker.

You are NOT in the critical path — you only act when messaged. No proactive outreach.

When consulted:
- Evaluate the proposed approach against the design pillars
- Return: "fine" OR "flag: [specific pillar this conflicts with] — consider [alternative]"
- If a genuine new design decision is required (not a correctness fix), flag to orchestrator

You accumulate understanding of what's been validated and what concerns were raised, so you
don't re-raise dismissed points in later waves.

Message the orchestrator only for genuine new design decisions requiring human input:
  SendMessage(to: "orchestrator", message: "Design decision needed: ...", summary: "design decision required")

Wait to be consulted. Do not initiate.
"""
})

Agent({
  name: "guard-architecture",
  team_name: "phantom-breaker-team",
  description: "Persistent architecture guard — plugin boundary and message pattern checks on demand",
  prompt: """
You are the persistent architecture guard for the phantom-breaker refactor.
Read docs/todos/detail/phantom-breaker.md in full now.
Also read docs/architecture/ in full, with special attention to plugins.md, messages.md, and
any system-ordering doc. These define the rules you enforce.

Your role: validate plugin boundaries, module structure, message patterns, and cross-domain wiring
when consulted. Any team member may ask you a question. You answer it and return directly to the asker.

You are NOT in the critical path — you only act when messaged. No proactive outreach.

When consulted:
- Evaluate the proposed approach against the architecture rules (plugin ownership, message-only
  cross-domain communication, system ordering, component vs resource vs message placement)
- Return: "fine" OR "flag: [specific rule this violates] — the correct pattern is [alternative]"
- If a genuine new architectural decision is required (no existing rule covers it), flag to orchestrator

You accumulate understanding of what's been validated and what concerns were raised, so you
don't re-raise dismissed points in later waves. After seeing the phantom infra placement decision
in Wave 1, you don't question it again in Wave 4.

Message the orchestrator only for genuine architectural decisions requiring human input:
  SendMessage(to: "orchestrator", message: "Architectural decision needed: ...", summary: "arch decision required")

Wait to be consulted. Do not initiate.
"""
})
```

After all seven `Agent` calls complete, the team is idle. I trigger the pipeline with a single message to `spec-writer` per wave.

---

### Wave flow with the team

Here is the per-wave sequence. Bold lines are orchestrator actions; everything else is peer-to-peer.

**Orchestrator starts wave:**
```
SendMessage(to: "spec-writer", message: """
Wave 3: grade_bump .single_mut() → .iter_mut() migration.
Scope: system in breaker/systems/bump/system.rs; match incoming BoltImpactBreaker by entity.
Decisions: each breaker grades against its own messages; BumpPerformed carries breaker entity.
No cross-domain changes this wave.
Write the test spec to .claude/specs/wave3-phantom-breaker-tests.md.
""", summary: "wave 3 start")
```

**Peer loop (no orchestrator):**
- `spec-writer` writes test spec → messages `spec-reviewer`
- `spec-reviewer` ↔ `spec-writer` revision loop (0–3 rounds, direct)
- `spec-reviewer` messages `test-writer` + orchestrator: "test spec approved"

**Orchestrator runs reviewer-tests (stateless background agent), then RED gate:**
```
runner-tests (background, stateless) → RED gate
```

**If RED passes, orchestrator confirms to spec-writer:**
```
SendMessage(to: "spec-writer", message: "RED gate passed. Failing tests at breaker-game/src/breaker/systems/bump/tests/grade_bump_iter.rs — write code spec.", summary: "RED gate passed wave 3")
```

**Peer loop (no orchestrator):**
- `spec-writer` writes code spec → messages `spec-reviewer`
- `spec-reviewer` ↔ `spec-writer` revision loop (direct)
- `spec-reviewer` messages `implementer` + orchestrator: "code spec approved"

**Peer loop — GREEN phase (no orchestrator):**
- `implementer` reads spec, implements, messages orchestrator: "done, run GREEN gate"

**Orchestrator runs GREEN gate (stateless runner):**
- If PASS → orchestrator updates session-state, routes to verification tier
- If FAIL → implementer messages `debugger` directly with full error
  - `debugger` ↔ `implementer` fix loop (direct, 1-3 rounds)
  - `implementer` messages orchestrator: "fixed, please re-run GREEN gate"
  - If circuit-break: `debugger` messages orchestrator for human input

---

### What the orchestrator owns (unchanged)

| Orchestrator always does | Team handles |
|---|---|
| Start each wave | Spec writing + revision loop |
| Run cargo (runner-tests, runner-linting — stateless) | Test writing + spec clarifications |
| Update session-state after each notification | GREEN phase implementation |
| Standard + Full Verification tiers (stateless agents) | GREEN failure diagnosis + fix |
| Human judgment: new design decisions, wave scoping | Design-adjacent checks via guard-design |
| Circuit-break escalation (3 attempts) | In-wave peer communication |

### What still runs as stateless background agents

Runner agents (runner-tests, runner-linting, runner-scenarios) always remain stateless — cargo cannot live inside a team conversation turn. Standard and Full Verification tier reviewers (reviewer-correctness, reviewer-quality, reviewer-architecture, reviewer-bevy-api, reviewer-performance, reviewer-completeness, guard-docs, guard-security, guard-dependencies, guard-agent-memory) also remain stateless — they run at phase boundaries and their check is always a clean slate against the committed code.

### Shared task list

The team shares a task list at `~/.claude/tasks/phantom-breaker-team/`. Use it for wave-level coordination:
- Orchestrator creates "Wave 3 test spec", spec-writer claims it
- Spec-writer marks it done, creates "Wave 3 test-spec review"
- And so on — task list is the wave progress ledger the whole team reads

### Session-state notes for this trial

Update session-state after each notification from a teammate, exactly as with background agents. The key difference: many columns that normally require orchestrator round-trips (spec revision status, test-writer handoff, implementer briefing) are now peer-to-peer and only surface to session-state at phase completion points (spec approved, RED gate, GREEN gate).

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
