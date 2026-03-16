# Brickbreaker Roguelite

Roguelite Arkanoid clone in Bevy 0.18 (Rust). See `docs/DESIGN.md` for design pillars, `docs/architecture/` for technical decisions + code standards + testing approach, `docs/PLAN.md` for build roadmap, `docs/TERMINOLOGY.md` for game vocabulary.

## Build & Run

```
cargo dev                    # Dev build + run (dynamic linking)
cargo dtest                  # Run all tests (dynamic linking)
cargo dcheck                 # Type check (dynamic linking)
cargo dclippy                # Lint (dynamic linking)
cargo dbuild                 # Dev build without running (dynamic linking)
cargo run --release          # Release build
cargo fmt --check            # Format check
```

Dev builds use `.cargo/config.toml` aliases with `bevy/dynamic_linking` for fast compiles.

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
- Write tests FIRST for new game logic (see `docs/architecture/standards.md` Testing — TDD)
- Follow the git workflow in @.claude/rules/git.md
- Run command line tools individually, do not chain them with &&

**Move freely on**:
- Implementation within existing system boundaries
- Adding tests
- Bug fixes with obvious solutions
- Updating RON data files

## Design Rules

See `docs/DESIGN.md` for the full set of non-negotiable design pillars. The key mechanical rules are in `docs/architecture/` (bolt reflection, breaker state machine, bump grades).

## Agent Workflow

The main agent is the orchestrator. Invoke subagents automatically at these trigger points — do not wait to be asked.

### Delegated Implementation (test-writer + code-writer)

For multi-domain work or context-heavy phases, delegate implementation to the **test-writer** → **code-writer** TDD pair. This preserves main context and prevents implementation bias through context isolation. See @.claude/rules/delegated-implementation.md for full spec-writing guidelines.

**When to delegate**: 2+ independent domains in one phase, or single domain where spec is clear and context preservation matters.

**When NOT to delegate**: Cross-cutting changes, exploratory work, new domain wiring, trivial additions.

**The flow**:
1. Write behavioral spec → launch **test-writer** (parallel across domains)
2. **Review tests** (mandatory checkpoint — verify they capture intent)
3. Write implementation spec → launch **code-writer** (parallel across domains)
4. Handle shared wiring (`lib.rs`, `game.rs`, `shared.rs`) yourself
5. Run the post-implementation checklist below

### Phase 1 — Before Writing Code (sequential)

| Trigger | Agent | Why |
|---------|-------|-----|
| Unfamiliar Bevy 0.18 API or pattern | **bevy-api-expert** | Verify before using — Bevy APIs change between versions |

### Phase 2 — After Implementation (launch in parallel)

Launch all applicable agents simultaneously — they are independent:

| Trigger | Agent | Why |
|---------|-------|-----|
| Always after implementation | **test-runner** | Full validation suite (fmt, clippy, tests) |
| Always after implementation | **correctness-reviewer** | Logic bugs, ECS pitfalls, state machine holes, math |
| Always after implementation | **quality-reviewer** | Idioms, vocabulary, test coverage, documentation |
| Always after implementation | **bevy-api-reviewer** | Verify Bevy API usage is correct for this version |
| New system, plugin, or module added | **architecture-guard** | Validate plugin boundaries and message discipline |
| 3+ systems added, or cross-plugin data flow | **system-dependency-mapper** | Detect ordering issues and conflicts |
| New components or systems touching many entities | **perf-guard** | Bevy-specific performance: queries, archetypes, scheduling |
| New gameplay mechanic or upgrade designed | **game-design-guard** | Validate against design pillars |
| Phase complete or significant structural change | **doc-guard** | Sync architecture docs, PLAN.md, TERMINOLOGY.md |

### Phase 3 — On Build/Test Failure (sequential, reactive)

| Trigger | Agent | Why |
|---------|-------|-----|
| Compiler errors that aren't obvious | **rust-error-decoder** | Translate diagnostics into actionable fixes |

### Release (solo)

| Trigger | Agent | Why |
|---------|-------|-----|
| Preparing a release or release infrastructure | **release** | Version bump, changelog, GitHub Actions, itch.io |

---

**Post-implementation checklist** (run before considering a task done):
1. Launch **test-runner** + **correctness-reviewer** + **quality-reviewer** + **bevy-api-reviewer** in parallel (always)
2. If new systems/plugins added → also launch **architecture-guard** + **system-dependency-mapper** in the same parallel wave
3. If new gameplay mechanic → also launch **game-design-guard** in the same parallel wave
4. If phase complete or docs may have drifted → also launch **doc-guard** in the same parallel wave
5. Run `/simplify` on changed code
6. Commit to the feature branch with a conventional commit message
