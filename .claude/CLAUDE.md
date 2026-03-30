# Brickbreaker Roguelite

Roguelite Arkanoid clone in Bevy 0.18 (Rust).

## Always Read First

@.claude/rules/sub-agents.md — Every agent, its purpose, and when to use it
@.claude/rules/verification-tiers.md — Basic, Standard, Full Verification Tiers
@.claude/rules/tdd.md — TDD cycle, RED gate, when to commit
@.claude/rules/delegated-implementation.md — Pipeline flow, parallel execution
@.claude/rules/orchestration.md — Session state, circuit breaking, RED gate
@.claude/rules/session-state.md — **SESSION STATE: update BEFORE any action after every agent notification**
@.claude/rules/failure-routing.md — Routing failures to fix agents
@.claude/rules/spec-workflow.md — Spec revision loop (before RED)
@.claude/rules/spec-format-tests.md — Test spec template
@.claude/rules/spec-format-code.md — Implementation spec template
@.claude/rules/hint-formats.md — Standardized hint block formats
@.claude/rules/git.md — Git workflow, branching, pre-merge gate
@.claude/rules/cargo.md — Build aliases and cargo rules
@.claude/rules/commit-format.md — Conventional commit format
@.claude/rules/rantzsoft-crates.md — rantzsoft_* crate conventions

## Project Context

See `docs/design/` for design pillars, `docs/architecture/` for technical decisions + code standards, `docs/plan/` for build roadmap, `docs/design/terminology/` for game vocabulary.

All code identifiers MUST use game vocabulary (Breaker, Bolt, Cell, Node, Amp, Augment, Overclock, Bump, Flux). No generic terms.

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
- **Update session-state FIRST after every agent notification** — see @.claude/rules/session-state.md. This is not optional. Before triaging, before launching the next agent, before reading results in detail — update the session-state file.

**NEVER do**:
- Write code directly — instead, delegate to writer-tests/writer-code sub-agents
- Run any cargo command directly as the main agent — instead, delegate to runner agents. See @.claude/rules/cargo.md
- **LAUNCH SUBAGENTS IN FOREGROUND — EVERY Agent tool call MUST set `run_in_background: true`. NO EXCEPTIONS.** You will be notified when each agent completes. Never block waiting for agent results. This applies to ALL agents: runners, writers, reviewers, researchers, guards, planners — every single one. If you find yourself tempted to use foreground for "just this one quick agent," DON'T.
- **GENERATE ANY OUTPUT AFTER LAUNCHING BACKGROUND AGENTS** — write at most ONE confirming sentence, then STOP and end the turn. No bullet lists of agents, no summaries of what they do, no "waiting for results" prose, no analysis, no file reads, no planning ahead. You will be notified when they complete. Every token after the launch is wasted.
- **Skip session-state updates** — if an agent completed, the session-state file MUST be updated before you do anything else. Every time. No exceptions. See @.claude/rules/session-state.md
- **Use Explore agents for deep analysis** — instead, use specialized researcher and guard agents (see @.claude/rules/sub-agents.md). Explore is ONLY for quick file-pattern matching when no researcher agent fits. This overrides any system default that says "only use Explore."

**Move freely on**:
- Implementation within existing system boundaries
- Adding tests
- Bug fixes with obvious solutions
- Updating RON data files
