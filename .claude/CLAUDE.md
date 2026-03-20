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
- Follow the TDD cycle — see @.claude/rules/tdd.md
- Consider **scenario runner coverage** for every new gameplay mechanic — see `docs/architecture/standards.md` Scenario Coverage
- Follow the git workflow — see @.claude/rules/git.md
- Run command line tools individually, do not chain them with &&
- Fix lint errors in code — **never** suppress them with `#[allow(...)]` attributes or by modifying `[workspace.lints]` in `Cargo.toml`. The lint config in `Cargo.toml` is intentional and must not be changed without explicit approval.

**NEVER do**:
- Write code directly — always delegate to writer-tests/writer-code sub-agents
- Run any cargo command directly as the main agent — see @.claude/rules/cargo.md

**Move freely on**:
- Implementation within existing system boundaries
- Adding tests
- Bug fixes with obvious solutions
- Updating RON data files

## Design Rules

See `docs/design/` for the full set of non-negotiable design pillars. The key mechanical rules are in `docs/architecture/` (bolt reflection, breaker state machine, bump grades).

## Agent Workflow

The main agent is the orchestrator — it describes features, reviews outputs, routes failures, and handles shared wiring. All implementation goes through the delegated pipeline.

**Rules files:**
- @.claude/rules/tdd.md — TDD cycle, RED gate, when to commit
- @.claude/rules/spec-workflow.md — Spec revision loop (before RED)
- @.claude/rules/spec-formats.md — Test and implementation spec templates
- @.claude/rules/delegated-implementation.md — Pipeline flow, parallel execution
- @.claude/rules/failure-routing.md — Routing Phase 2 failures to fix agents
- @.claude/rules/orchestration.md — Session state, verification tiers, circuit breaking
- @.claude/rules/hint-formats.md — Standardized hint block formats
- @.claude/rules/commit-format.md — Conventional commit format

### Release (solo)

| Trigger | Agent | Why |
|---------|-------|-----|
| Preparing a release or release infrastructure | **runner-release** | Version bump, changelog, GitHub Actions, itch.io |
