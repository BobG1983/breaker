# Brickbreaker Roguelite

Roguelite Arkanoid clone in Bevy 0.18 (Rust). See `docs/DESIGN.md` for design pillars, `docs/architecture/` for technical decisions + code standards + testing approach, `docs/PLAN.md` for build roadmap, `docs/TERMINOLOGY.md` for game vocabulary.

## Build & Run

```
cargo dev                    # Dev build + run (dynamic linking)
cargo dtest                  # Run all tests (dynamic linking)
cargo dcheck                 # Type check (dynamic linking)
cargo dclippy                # Lint (dynamic linking)
cargo dbuild                 # Dev build without running (dynamic linking)
cargo scenario -- --all      # Run all scenarios (release build)
cargo run --release          # Release build
cargo fmt --check            # Format check
```

**NEVER** use bare cargo commands — see @.claude/rules/cargo.md.

Dev builds use `.cargo/config.toml` aliases with `bevy/dynamic_linking` for fast compiles.

## Workspace

Cargo workspace with `breaker-<name>` crate directories at root: `breaker-game/` (main game), `breaker-derive/` (proc macros). New crates follow this convention.

## Architecture

**Plugin-per-domain** with message-driven decoupling. Each domain plugin (input, breaker, bolt, cells, upgrades, run, physics, audio, ui, debug) owns its components, resources, and systems. Domains communicate only through Bevy 0.18 messages. See `docs/architecture/` for full details, file tree, message table, and patterns.

## Terminology

All code identifiers MUST use game vocabulary (Breaker, Bolt, Cell, Node, Amp, Augment, Overclock, Bump, Flux). No generic terms. See `docs/TERMINOLOGY.md`.

## Decision Making

**ALWAYS ask before**:
- Creating new plugins, systems, or modules not in the architecture
- Choosing between component vs resource vs message for new data
- Any design decision not covered in `docs/PLAN.md`
- Architectural changes or refactors affecting multiple systems

**ALWAYS do**:
- Follow the RED → GREEN → REFACTOR cycle for new game logic: write **failing** tests first (RED), implement minimum code to pass (GREEN), then refactor (REFACTOR). Tests **must fail** before writing any implementation. See `docs/architecture/standards.md` Testing — TDD.
- Follow the git workflow in @.claude/rules/git.md
- Run command line tools individually, do not chain them with &&
- Fix lint errors in code — **never** suppress them with `#[allow(...)]` attributes or by modifying `[workspace.lints]` in `Cargo.toml`. The lint config in `Cargo.toml` is intentional and must not be changed without explicit approval.

**Move freely on**:
- Implementation within existing system boundaries
- Adding tests
- Bug fixes with obvious solutions
- Updating RON data files

## Design Rules

See `docs/DESIGN.md` for the full set of non-negotiable design pillars. The key mechanical rules are in `docs/architecture/` (bolt reflection, breaker state machine, bump grades).

## Agent Workflow

The main agent is the orchestrator. Invoke subagents automatically at these trigger points — do not wait to be asked.

See @.claude/rules/agent-flow.md for the full flow reference: hint formats, failure routing, and parallel-launch requirements.

### Delegated Implementation (writer-tests + writer-code)

For multi-domain work or context-heavy phases, delegate implementation to the **writer-tests** → **writer-code** TDD pair. This preserves main context and prevents implementation bias through context isolation. See @.claude/rules/delegated-implementation.md for full spec-writing guidelines.

**When to delegate**: Anything non-trivial — single domain or multi-domain. If it has 2+ behaviors to test and the spec can be written clearly, delegate it.

**When NOT to delegate**: Cross-cutting changes, exploratory work, new domain wiring, trivial additions (single function, one-liner config, rename).

**The flow** (RED → GREEN → REFACTOR):
1. Write ALL specs upfront (behavioral spec for writer-tests + implementation spec for writer-code, one pair per domain)
2. Launch ALL **writer-tests** as background agents (parallel across domains) — RED phase
3. As each writer-tests completes: review its output, immediately launch its paired **writer-code** — don't wait for other writer-tests
4. After ALL writer-codes complete (code only — no self-verification): run the post-implementation checklist below (runner-linting + runner-tests verify)
5. Handle shared wiring (`lib.rs`, `game.rs`, `shared.rs`) yourself — REFACTOR as needed

### Phase 1 — Before Writing Code (sequential)

| Trigger | Agent | Why |
|---------|-------|-----|
| Unfamiliar Bevy 0.18 API or pattern | **researcher-bevy-api** | Verify before using — Bevy APIs change between versions |

### Phase 2 — After Implementation (launch in parallel)

Launch all applicable agents simultaneously — they are independent:

| Trigger | Agent | Why |
|---------|-------|-----|
| Always after implementation | **runner-linting** | Auto-fmt and clippy; errors → writer-code |
| Always after implementation | **runner-tests** | Run tests; failures → writer-code or writer-tests |
| Always after implementation | **runner-scenarios** | Run all gameplay scenarios headlessly and diagnose failures |
| Always after implementation | **reviewer-correctness** | Logic bugs, ECS pitfalls, state machine holes, math |
| Always after implementation | **reviewer-quality** | Idioms, vocabulary, test coverage, documentation |
| Always after implementation | **reviewer-bevy-api** | Verify Bevy API usage is correct for this version |
| New system, plugin, or module added | **guard-architecture** | Validate plugin boundaries and message discipline |
| 3+ systems added, or cross-plugin data flow | **researcher-system-dependencies** | Detect ordering issues and conflicts |
| New components or systems touching many entities | **guard-performance** | Bevy-specific performance: queries, archetypes, scheduling |
| New gameplay mechanic or upgrade designed | **guard-game-design** | Validate against design pillars |
| Phase complete or significant structural change | **guard-docs** | Sync architecture docs, PLAN.md, TERMINOLOGY.md |

### Phase 3 — On Build/Test Failure (sequential, reactive)

| Trigger | Flow | Notes |
|---------|------|-------|
| Compiler errors that aren't obvious | **researcher-rust-errors** → describe fix | Sequential |
| runner-linting FAIL, clippy errors | runner-linting hint → **writer-code** | No writer-tests needed |
| runner-scenarios FAIL, high-confidence diagnosis | runner-scenarios hint → **writer-tests** (regression spec) → **writer-code** | writer-tests writes scenario RON or unit test |
| runner-scenarios FAIL, low-confidence diagnosis | Main agent investigates → writes spec → **writer-tests** → **writer-code** | Main agent reads src first |
| runner-tests FAIL, existing test broke | runner-tests hint → **writer-code** (fix spec) | writer-tests skipped — test already exists |
| runner-tests FAIL, no test for the broken behavior | runner-tests hint → **writer-tests** (regression spec) → **writer-code** | Rare — usually means a gap in coverage |

### Release (solo)

| Trigger | Agent | Why |
|---------|-------|-----|
| Preparing a release or release infrastructure | **runner-release** | Version bump, changelog, GitHub Actions, itch.io |

---

**Post-implementation checklist** (run before considering a task done):
1. Launch **runner-linting** + **runner-tests** + **runner-scenarios** + **reviewer-correctness** + **reviewer-quality** + **reviewer-bevy-api** in parallel (always)
2. If new systems/plugins added → also launch **guard-architecture** + **researcher-system-dependencies** in the same parallel wave
3. If new gameplay mechanic → also launch **guard-game-design** in the same parallel wave
4. If phase complete or docs may have drifted → also launch **guard-docs** in the same parallel wave
5. Run `/simplify` on changed code
6. Commit to the feature branch with a conventional commit message
