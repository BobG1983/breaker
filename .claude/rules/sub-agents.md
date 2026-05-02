# Sub-Agents

> **Team mode**: when a team is active (see `.claude/rules/team-mode.md`), most of these agents are spawned ONCE as persistent members rather than per-phase. The role descriptions below still apply unchanged — only the lifecycle differs.

Every sub-agent, what it does, and when to use it. Agent definitions live in `.claude/agents/`.

## Pipeline Agents

Used during the delegated implementation pipeline (see `delegating-to-subagents.md`).

| Agent | Purpose | When to use |
|-------|---------|-------------|
| **planning-writer-specs-tests** | Writes behavioral test specs to `.claude/specs/` | Starting a new feature — TEST SPEC phase (before writer-tests) |
| **planning-reviewer-specs-tests** | Pressure-tests test specs for missing behaviors, incorrect values, scope | After test spec is written — before writer-tests |
| **writer-tests** | Writes failing tests from a test spec file (RED phase) | After test spec is reviewed and clean |
| **reviewer-tests** | Verifies writer-tests output matches spec behaviors | After each writer-tests completes, before RED gate |
| **planning-writer-specs-code** | Writes implementation specs to `.claude/specs/`, reading the failing tests on disk as the contract | CODE SPEC phase — **after RED gate passes**, before writer-code |
| **planning-reviewer-specs-code** | Pressure-tests implementation specs against the actual failing tests | After impl spec is written — before writer-code |
| **writer-code** | Implements production code to pass failing tests (GREEN phase) | After code spec is reviewed and clean |

## Runner Agents

Execute cargo commands and report results. Only runners run cargo — see `.claude/rules/cargo.md`.

| Agent | Purpose | When to use |
|-------|---------|-------------|
| **runner-cargo** | `cargo fmt` + `cargo all-dclippy` + `cargo all-dtest` + `cargo scenario -- --all` | All verification tiers — lint and tests (Basic), scenarios (Full); RED gate, GREEN gate |

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
| **reviewer-file-length** | Finds oversized files; orchestrator launches sub-agents (background forks) to perform splits in current branch before merge (never via todo, /implement, or /quickfix); Basic Verification Tier after | Full Verification Tier |

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
| **researcher-rust** | Decodes compiler/clippy errors AND evaluates idiomatic Rust patterns (pure Rust, not framework APIs) | Build failures needing diagnosis, or choosing between idiom alternatives |
| **researcher-crates** | Evaluates crate options against project criteria | Choosing a new dependency |
| **researcher-git** | Analyzes git history for a file, function, or feature area | Modifying code with non-obvious history |

## Debugger

| Agent | Purpose | When to use |
|-------|---------|-------------|
| **debugger** | Systematic root-cause analysis via the DEBUG protocol — hypotheses, Five Whys, regression spec hint output | Spawned by `/investigate`, especially after the 3-attempt circuit-break or for failures spanning multiple components |

## Scenario Agents

| Agent | Purpose | When to use |
|-------|---------|-------------|
| **writer-scenarios** | Generates scenario RON files and invariant checkers | New mechanic needs adversarial scenario coverage |
| **reviewer-scenarios** | Audits scenario coverage against full mechanic list | Exhaustive coverage audit or post-refactor gap analysis |

## Release

| Agent | Purpose | When to use |
|-------|---------|-------------|
| **runner-release** | Version bump, changelog, GitHub Actions, itch.io distribution | Preparing a release or release infrastructure |
