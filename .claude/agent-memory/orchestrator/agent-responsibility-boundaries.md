---
name: agent-responsibility-boundaries
description: Never mix agent responsibilities in prompts — each runner agent has exactly one job
type: feedback
---

Never ask runner-tests to run clippy. Never ask runner-linting to run tests.

**Why:** Each runner agent has a single responsibility. runner-tests runs `cargo dtest` and `cargo dstest` only. runner-linting runs `cargo fmt` and `cargo dclippy`/`cargo dsclippy` only. Mixing them confuses outputs and violates the separation of concerns.

**How to apply:** When the orchestrator needs both lint and test verification, launch runner-linting and runner-tests as separate parallel agents. Never combine "run clippy then dtest" in a single agent prompt.
