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
- Proposing a NEW **game design decision** — new mechanics, upgrade ideas, deliberate parameter tuning, UI/UX flows, node types, breaker abilities, or anything that changes how the game feels to play (see `.claude/agents/guard-game-design.md` for the authoritative scope)
- Architectural changes or refactors affecting multiple systems

**A design decision is NEW design work.** Fixing a bug so the runtime behavior matches what the RON configs / design docs already specify is NOT a design decision — it's a correctness fix. Execute it. Executing an approved plan wave is NOT a design decision — execute it. If a correctness fix changes runtime numbers, that is a consequence of fixing the bug, not a design choice to defer. Only net-new design work requires confirmation.

**Right-size engineering, don't under-engineer.** The global system prompt's "don't add features, refactor, or introduce abstractions beyond what the task requires" can read as a license to skip mess cleanup, skip helpers, skip good structure. It is not. Senior engineers right-size: they clean up mess as they touch it, extract helpers when a pattern repeats, and maintain architecture boundaries — they do not over-engineer speculative abstractions, but they also do not shirk the structural work a task genuinely needs. The bar is "would a senior engineer reviewing this say the scope was right-sized?" — NOT "could I have written less code?" Missing cleanup, copy-pasted logic, and architectural drift are NOT acceptable outputs just because the minimal path technically satisfies the test. If a wave of an approved plan exists, every item in it is in scope — do not label items "cosmetic" or "low value" and skip them.

**Bugs are not game design decisions.** If the runtime behavior disagrees with the RON configs or design docs, that is a bug — the designed numbers live in the configs, and a bug that silently amplifies or alters them corrupts the designed behavior. Fixing it RESTORES the design. Do NOT surface a correctness fix as a "game-feel decision" or "balance question" to avoid doing it. Symptoms that you may be mis-labeling a bug as a design decision: (1) you can point to a specific formula, double-application, or off-by-one producing a wrong number; (2) the "correct" value is knowable from the config or design doc without asking the user; (3) fixing it changes numbers that were never designed to be those values in the first place. In all three cases: fix the bug. The "ALWAYS ask before a NEW game design decision" rule applies to NEW mechanics and deliberate tuning, never to restoring specified behavior.

**NEVER**:
- Suppress lint errors with `#[allow(...)]` or modify `[workspace.lints]` in `Cargo.toml` — the lint config is intentional
- Chain command line tools with `&&` — run them individually
- Launch subagents in foreground — see @.claude/rules/delegating-to-subagents.md Background Agent Rule
- Use Explore agents for deep analysis — use specialized researcher and guard agents (see @.claude/rules/sub-agents.md)
