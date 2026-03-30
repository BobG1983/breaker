# Verification Tiers

Single source of truth for what agents run at each verification tier. All other rules reference this file.

## Basic Verification Tier

**When**: After each writer-code wave, after each fix agent cycle, after inline fixes.

| Agent | Purpose |
|-------|---------|
| **runner-linting** | fmt + clippy across all workspace crates |
| **runner-tests** | tests across all workspace crates |

**Question answered**: "Does it compile, pass tests, and lint clean?"

All compiler and clippy errors and failing tests must be fixed for this tier to be considered complete.

## Standard Verification Tier

**When**: Commit gate — after Basic Verification Tier is clean and /simplify finds nothing.

Includes everything in Basic Verification Tier, plus:

| Agent | Purpose |
|-------|---------|
| **reviewer-correctness** | Logic bugs, state machine holes, math errors |
| **reviewer-quality** | Rust idioms, game vocabulary, test coverage gaps |
| **reviewer-bevy-api** | Correct Bevy API usage for project's version |
| **reviewer-architecture** | Plugin boundaries, module structure, message patterns |
| **reviewer-performance** | Archetype fragmentation, query efficiency, hot-path allocations |

**Question answered**: "Is the code correct, idiomatic, and well-structured?"

All compiler and clippy errors and failing tests must be complete, and all feedback from reviewers investigated and fixed (if required) for this tier to be complete.

## Full Verification Tier

**When**: Pre-merge gate — before `git flow <type> finish`, at phase boundaries.

Includes everything in Standard Verification Tier, plus:

| Agent | Purpose |
|-------|---------|
| **runner-scenarios** | Automated gameplay testing under chaos input |
| **guard-security** | Unsafe blocks, deserialization, supply chain risks |
| **guard-docs** | Documentation drift from code |
| **guard-game-design** | Mechanic changes against design pillars |
| **guard-dependencies** | Unused/outdated/duplicate deps, license compliance |
| **guard-agent-memory** | Stale/duplicated memories, MEMORY.md accuracy |
| **reviewer-file-length** | Finds oversized files, produces split spec for writer-code |
| **reviewer-scenarios** | Scenario coverage gaps, weak invariants, missing mechanics |

**Question answered**: "Is everything clean across all cross-cutting concerns?"

ALL compiler and clippy ERRORS AND WARNINGS, ALL failing tests, ALL failing scenarios, and all feedback from all agents MUST be fixed for this tier to be complete, DEFER NONE, FIX EVERYTHING, UNLESS IT IS A SUGGESTION FROM ANOTHER AGENT. SUGGESTIONS MUST BE INVESTIGATED AND EITHER FIXED OR CALLED OUT AS WILL NEVER FIX. ***IF YOU WOULD HAVE TO FIX IT EVENTUALLY IT MUST BE FIXED NOW***

## Pipeline Summary

```
GREEN gate (runner-tests only)
    ↓
Basic Verification Tier (lint + tests)
    ↓ fix failures → Basic Verification Tier again
/simplify
    ↓ if changes → Basic Verification Tier again
Wiring
    ↓
Standard Verification Tier (commit gate)
    ↓ fix failures → Basic Verification Tier until clean → Standard Verification Tier again
Commit
    ↓
Full Verification Tier (pre-merge gate)
    ↓ fix failures → Basic Verification Tier until clean → Standard Verification Tier → Full Verification Tier again
Merge
```

## Parallelism Rules

- All agents within a tier launch in parallel
- Cargo commands serialize automatically (only one runner at a time)
- Reviewers and guards are read-only — safe to run concurrently with each other
- After a fix cycle, re-run Basic Verification Tier first (fast), then Standard Verification Tier (if Basic passes)
- Never skip a tier — Basic before Standard, Standard before Full
