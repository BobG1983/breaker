# Sub-Agents

Every sub-agent, what it does, and when to use it. Agent definitions live in `.claude/agents/`.

## Pipeline Agents

Used during the delegated implementation pipeline (see `delegating-to-subagents.md`).

| Agent | Purpose | When to use |
|-------|---------|-------------|
| **planning-writer-specs-tests** | Writes behavioral test specs to `.claude/specs/` | Starting a new feature — SPEC phase (parallel with specs-code) |
| **planning-writer-specs-code** | Writes implementation specs to `.claude/specs/` | Starting a new feature — SPEC phase (parallel with specs-tests) |
| **planning-reviewer-specs-tests** | Pressure-tests test specs for missing behaviors, incorrect values, scope | After test spec is written — before writer-tests |
| **planning-reviewer-specs-code** | Pressure-tests implementation specs for feasibility, alignment, patterns | After impl spec is written — before writer-code |
| **writer-tests** | Writes failing tests from a test spec file (RED phase) | After specs are reviewed and clean |
| **writer-code** | Implements production code to pass failing tests (GREEN phase) | After RED gate passes |
| **reviewer-tests** | Verifies writer-tests output matches spec behaviors | After each writer-tests completes, before RED gate |

## Verification Agents

| Agent | Purpose | When to use |
|-------|---------|-------------|
| **reviewer-completeness** | Verifies implementation delivers what the todo detail and plan wave promised | Standard Verification Tier (commit gate) — parallel with other reviewers |

See `.claude/rules/verification-tiers.md` for which agents run in each tier (Basic, Standard, Full), when each tier runs, and the pipeline flow.

## Research Agents

Used during pre-planning research (see `delegating-to-subagents.md` step 2) and ad-hoc investigation.

| Agent | Purpose | When to use |
|-------|---------|-------------|
| **researcher-system-dependencies** | Maps system read/write conflicts, message flow, ordering | Feature touches 2+ domains, or 3+ systems added |
| **researcher-bevy-api** | Verifies Bevy API usage, looks up signatures, checks deprecations | Unfamiliar Bevy 0.18 API or pattern |
| **researcher-impact** | Finds ALL references to a type/system/message before modifying it | Before renaming, refactoring, or changing signatures |
| **researcher-codebase** | Traces end-to-end data flow through ECS for a feature | Need to understand current behavior before modifying it |
| **researcher-rust-idioms** | Evaluates idiomatic Rust patterns for a specific situation | Choosing between idiom alternatives |
| **researcher-rust-errors** | Translates compiler errors into actionable fix instructions | Build failures that need diagnosis |
| **researcher-crates** | Evaluates crate options against project criteria | Choosing a new dependency |
| **researcher-git** | Analyzes git history for a file, function, or feature area | Modifying code with non-obvious history |

## Scenario Agents

| Agent | Purpose | When to use |
|-------|---------|-------------|
| **writer-scenarios** | Generates scenario RON files and invariant checkers | New mechanic needs adversarial scenario coverage |
| **reviewer-scenarios** | Audits scenario coverage against full mechanic list | Exhaustive coverage audit or post-refactor gap analysis |

## Release

| Agent | Purpose | When to use |
|-------|---------|-------------|
| **runner-release** | Version bump, changelog, GitHub Actions, itch.io distribution | Preparing a release or release infrastructure |
