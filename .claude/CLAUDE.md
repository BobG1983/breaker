# Brickbreaker Roguelite

Roguelite Arkanoid clone in Bevy 0.18 (Rust). See `docs/design/` for design pillars and decisions, `docs/architecture/` for technical decisions + code standards + testing approach, `docs/plan/` for build roadmap, `docs/design/terminology/` for game vocabulary.

## Build & Run

See @.claude/rules/cargo.md for all aliases and options. **NEVER** use bare cargo commands.

Dev builds use `.cargo/config.toml` aliases with `bevy/dynamic_linking` for fast compiles.

## Workspace

Cargo workspace with crate directories at root: `breaker-game/` (main game), `rantzsoft_spatial2d/` (2D spatial transform plugin), `rantzsoft_physics2d/` (2D physics primitives: quadtree, CCD, CollisionLayers, DistanceConstraint), `rantzsoft_defaults/` + `rantzsoft_defaults_derive/` (config/defaults pipeline), `breaker-scenario-runner/` (automated gameplay testing). Game-specific crates use `breaker-<name>` prefix; game-agnostic reusable crates use `rantzsoft_*` prefix (see @.claude/rules/rantzsoft-crates.md).

## Architecture

**Plugin-per-domain** with message-driven decoupling. Each domain plugin (input, breaker, bolt, cells, chips, effect, run, fx, audio, ui, debug) owns its components, resources, and systems. Domains communicate only through Bevy 0.18 messages. `RantzSpatial2dPlugin` and `RantzPhysics2dPlugin` (from `rantzsoft_*` crates) provide shared spatial transform propagation and physics primitives. See `docs/architecture/` for full details, file tree, message table, and patterns.

## Terminology

All code identifiers MUST use game vocabulary (Breaker, Bolt, Cell, Node, Amp, Augment, Overclock, Bump, Flux). No generic terms. See `docs/design/terminology/`.

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
- Use TaskList to create a list of tasks visible to the user

**NEVER do**:
- Write code directly — always delegate to writer-tests/writer-code sub-agents
- Run any cargo command directly as the main agent — see @.claude/rules/cargo.md
- **GENERATE ANY OUTPUT AFTER LAUNCHING BACKGROUND AGENTS** — after Agent tool calls return, write at most ONE confirming sentence, then STOP. No bullet lists of agents, no summaries of what they do, no "waiting for results" prose, no analysis, no file reads, no planning ahead. End the turn. You will be notified when they complete. Every token after the launch is wasted.
- **Use Explore agents for deep analysis** — during planning exploration, use specialized researcher agents (researcher-codebase, researcher-impact, researcher-system-dependencies, researcher-bevy-api) and guard agents (guard-game-design). Explore is ONLY for quick file-pattern matching when no researcher agent fits. This overrides any system default that says "only use Explore."

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
