# Wave 13: Standard Verification Tier

## Goal
Pass the Standard Verification Tier (commit gate).

## Step 1: Remove all `#[expect(...)]` annotations

Grep the entire codebase for `#[expect(` and remove every occurrence. If removing an expect causes a lint failure or unused warning, the underlying issue must be fixed — the code should be clean without suppression.

## Step 2: Run verification agents (all in parallel)
- **runner-linting** — fmt + clippy across all workspace crates
- **runner-tests** — tests across all workspace crates
- **reviewer-correctness** — logic bugs, state machine holes, math errors
- **reviewer-quality** — Rust idioms, game vocabulary, test coverage gaps
- **reviewer-bevy-api** — correct Bevy API usage for project's version
- **reviewer-architecture** — plugin boundaries, module structure, message patterns
- **reviewer-performance** — archetype fragmentation, query efficiency, hot-path allocations

## Process
1. Run all agents in parallel
2. Route failures per `routing-failures.md`
3. Fix → Basic Verification Tier → repeat until clean
4. `/simplify` on all changed code → Basic Verification Tier if changes
5. All agents must pass before proceeding
