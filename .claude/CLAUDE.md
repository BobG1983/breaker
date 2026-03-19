# Brickbreaker Roguelite

Roguelite Arkanoid clone in Bevy 0.18 (Rust). See `docs/design/` for design pillars and decisions, `docs/architecture/` for technical decisions + code standards + testing approach, `docs/plan/` for build roadmap, `docs/design/terminology.md` for game vocabulary.

## Build & Run

See @.claude/rules/cargo.md for all aliases and options. **NEVER** use bare cargo commands.

Dev builds use `.cargo/config.toml` aliases with `bevy/dynamic_linking` for fast compiles.

## Workspace

Cargo workspace with `breaker-<name>` crate directories at root: `breaker-game/` (main game), `breaker-derive/` (proc macros). New crates follow this convention.

## Architecture

**Plugin-per-domain** with message-driven decoupling. Each domain plugin (input, breaker, bolt, cells, upgrades, run, physics, audio, ui, debug) owns its components, resources, and systems. Domains communicate only through Bevy 0.18 messages. See `docs/architecture/` for full details, file tree, message table, and patterns.

## Terminology

All code identifiers MUST use game vocabulary (Breaker, Bolt, Cell, Node, Amp, Augment, Overclock, Bump, Flux). No generic terms. See `docs/design/terminology.md`.

## Decision Making

**ALWAYS ask before**:
- Creating new plugins, systems, or modules not in the architecture
- Choosing between component vs resource vs message for new data
- Any design decision not covered in `docs/plan/`
- Architectural changes or refactors affecting multiple systems

**ALWAYS do**:
- Follow the TDD cycle. See @.claude/rules/tdd.md.
- Consider **scenario runner coverage** for every new gameplay mechanic: new invariants to check, new scenario RON files to add, or new layouts that exercise the feature under chaos input. See `docs/architecture/standards.md` Scenario Coverage.
- Follow the git workflow in @.claude/rules/git.md
- Run command line tools individually, do not chain them with &&
- Fix lint errors in code — **never** suppress them with `#[allow(...)]` attributes or by modifying `[workspace.lints]` in `Cargo.toml`. The lint config in `Cargo.toml` is intentional and must not be changed without explicit approval.

**NEVER do**:
- Run `cargo dtest`, `cargo dclippy`, `cargo scenario`, or any cargo command directly as the main agent. Always delegate to **runner-tests**, **runner-linting**, or **runner-scenarios** agents. These agents produce output in hint formats that downstream writer agents consume.

**Move freely on**:
- Implementation within existing system boundaries
- Adding tests
- Bug fixes with obvious solutions
- Updating RON data files

## Design Rules

See `docs/design/` for the full set of non-negotiable design pillars. The key mechanical rules are in `docs/architecture/` (bolt reflection, breaker state machine, bump grades).

## Agent Workflow

The main agent is the orchestrator — it describes features, reviews outputs, routes failures, and handles shared wiring. All implementation goes through the delegated pipeline.

See @.claude/rules/tdd.md for the TDD cycle (RED → GREEN → REFACTOR), RED gate, and agent boundaries.
See @.claude/rules/delegated-implementation.md for spec formats, the full pipeline flow, and parallel execution rules.
See @.claude/rules/agent-flow.md for hint formats, failure routing, and parallel-launch requirements.
See @.claude/rules/orchestration.md for session state, verification tiers, circuit breaking, and context pruning.

### Phase 1 — Before Writing Code (sequential)

| Trigger | Agent | Why |
|---------|-------|-----|
| Unfamiliar Bevy 0.18 API or pattern | **researcher-bevy-api** | Verify before using — Bevy APIs change between versions |
| Choosing between Rust idiom alternatives | **researcher-rust-idioms** | Research idiomatic patterns before committing to an approach |
| Feature ready for spec writing | **planner-spec** | Produce behavioral + implementation specs per domain |
| Specs produced — novel, cross-domain, or uncertain | **planner-review** | Pressure-test specs before committing to writers |

### Phase 2 — After Implementation (launch in parallel)

Launch per verification tier defined in @.claude/rules/orchestration.md (Standard or Full).

All agents in a tier launch in a **single message** — separate messages make them sequential. Add conditional agents (researcher-system-dependencies, guard-game-design, guard-docs, guard-security, guard-dependencies, writer-scenarios, guard-agent-memory) to the same wave when triggered.

### Phase 3 — On Build/Test Failure (sequential, reactive)

| Trigger | Flow | Notes |
|---------|------|-------|
| Compiler errors that aren't obvious | **researcher-rust-errors** → describe fix | Sequential |
| runner-linting FAIL, clippy errors | runner-linting hint → **writer-code** | No writer-tests needed |
| runner-scenarios FAIL, high-confidence | runner-scenarios hint → **writer-tests** → **writer-code** | writer-tests writes scenario RON or unit test |
| runner-scenarios FAIL, low-confidence | Main agent investigates → writes spec → **writer-tests** → **writer-code** | Main agent reads src first |
| runner-tests FAIL, existing test broke | runner-tests hint → **writer-code** (fix spec) | writer-tests skipped — test already exists |
| runner-tests FAIL, no test for behavior | runner-tests hint → **writer-tests** → **writer-code** | Rare — gap in coverage |

### Release (solo)

| Trigger | Agent | Why |
|---------|-------|-----|
| Preparing a release or release infrastructure | **runner-release** | Version bump, changelog, GitHub Actions, itch.io |

---

**Post-implementation checklist** (run before considering a task done):
1. Launch verification agents per tier (Standard or Full) — see @.claude/rules/orchestration.md
2. Add conditional agents to the same parallel wave when triggered
3. Run `/simplify` on changed code
4. Repeat until all agents pass and `/simplify` finds nothing to change
5. Commit to the feature branch with a conventional commit message
