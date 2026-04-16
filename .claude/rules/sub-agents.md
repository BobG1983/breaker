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

## Runner Agents

Execute cargo commands and report results. Only runners run cargo — see `.claude/rules/cargo.md`.

| Agent | Purpose | When to use |
|-------|---------|-------------|
| **runner-linting** | `cargo fmt` + `cargo all-dclippy` across all workspace crates | Basic Verification Tier — after each writer-code wave, after fixes |
| **runner-tests** | `cargo all-dtest` across all workspace crates | Basic Verification Tier — RED gate, GREEN gate, after fixes |
| **runner-scenarios** | `cargo scenario -- --all` automated gameplay testing under chaos input | Full Verification Tier — pre-merge gate |

## Reviewer Agents

Read-only code review. Safe to run concurrently with each other and with runners.

| Agent | Purpose | When to use |
|-------|---------|-------------|
| **reviewer-completeness** | Verifies implementation delivers what the todo detail and plan wave promised | Standard Verification Tier (commit gate) — parallel with other reviewers |
| **reviewer-correctness** | Logic bugs, state machine holes, math errors | Standard Verification Tier |
| **reviewer-quality** | Rust idioms, game vocabulary, test coverage gaps | Standard Verification Tier |
| **reviewer-bevy-api** | Correct Bevy API usage for project's version | Standard Verification Tier |
| **reviewer-architecture** | Plugin boundaries, module structure, message patterns | Standard Verification Tier |
| **reviewer-performance** | Archetype fragmentation, query efficiency, hot-path allocations | Standard Verification Tier |
| **reviewer-file-length** | Finds oversized files, produces split spec | Full Verification Tier |

## Guard Agents

Cross-cutting concern validators. Read-only except for their own memory files.

| Agent | Purpose | When to use |
|-------|---------|-------------|
| **guard-docs** | Documentation drift from code | Full Verification Tier |
| **guard-game-design** | Mechanic changes against design pillars | Full Verification Tier |
| **guard-security** | Unsafe blocks, deserialization, supply chain risks | Full Verification Tier |
| **guard-dependencies** | Unused/outdated/duplicate deps, license compliance | Full Verification Tier |
| **guard-agent-memory** | Stale/duplicated memories, MEMORY.md accuracy | Full Verification Tier |

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
