# Brickbreaker Roguelite

Roguelite Arkanoid clone in Bevy 0.18 (Rust).

## Skills — Use These First

Skills are the primary workflow entry points. Match the user's intent to a skill before doing anything else.

| Situation | Skill |
|-----------|-------|
| Starting a new work item (optionally from a todo) | `/start-dev` |
| Implementing a feature or plan task | `/implement` |
| Small, single-file fix (one function, one test, a rename) | `/quickfix` |
| Checking code health | `/verify` |
| Reviewing changed code for quality | `/simplify` |
| Any failure — tests, scenarios, builds, or unexpected behavior after a change | `/investigate` |
| Done with a branch, ready to merge | `/finish-dev` |
| Capture work for later | `/todo` |

If the user's request doesn't fit a skill, proceed normally. But when it does fit — use the skill, don't ad-hoc it.

## Always Read First

@.claude/rules/sub-agents.md — Every agent, its purpose, and when to use it
@.claude/rules/session-state.md — **SESSION STATE: update BEFORE any action after every agent notification**

## Project Context

See `docs/design/` for design pillars, `docs/architecture/` for technical decisions + code standards, `docs/todos/` for the active work list and per-item detail files, `docs/design/terminology/` for game vocabulary.

All code identifiers MUST use game vocabulary — see `.claude/rules/project-context.md` Terminology.

## Decision Making

**ALWAYS investigate before fixing**: When tests, scenarios, or builds fail after a change, use `/investigate` before writing any fix — even if you think you know the cause. Do not guess. Do not bulk-edit files based on an untested hypothesis. Your first hypothesis is often wrong. The cost of investigating is tokens; the cost of a wrong fix cascade is the user's trust and a mess in the codebase.

**ALWAYS ask before**:
- Creating new plugins, systems, or modules not in the architecture
- Choosing between component vs resource vs message for new data
- Any design decision not covered in `docs/plan/`
- Architectural changes or refactors affecting multiple systems

**NEVER**:
- Suppress lint errors with `#[allow(...)]` or modify `[workspace.lints]` in `Cargo.toml` — the lint config is intentional
- Chain command line tools with `&&` — run them individually
- Launch subagents in foreground — see @.claude/rules/delegating-to-subagents.md Background Agent Rule
- Use Explore agents for deep analysis — use specialized researcher and guard agents (see @.claude/rules/sub-agents.md)
