---
name: let-underscore-drop in test cleanup
description: Test cleanup using let _ = fs::remove_dir_all() triggers clippy::let_underscore_drop because Result<(), io::Error> has a destructor
type: project
---

The `let _ = <expr>` idiom triggers `let_underscore_drop` whenever the expression returns a type with a destructor (e.g., `Result<T, io::Error>`, channel send results, `JoinHandle`, `writeln!` result).

**Files affected (as of 2026-04-07):**
- `breaker-scenario-runner/src/runner/tests/output_dir_tests.rs` — fs cleanup calls
- `breaker-scenario-runner/src/runner/run_log.rs` — channel sends, writer flushes, writeln!, handle.join()
- `breaker-scenario-runner/src/invariants/screenshot.rs:85` — `create_dir_all` call

**Why:** clippy warns that `let _` immediately drops the value without naming it, which for types with destructors is potentially surprising (vs `let _unused =` which keeps the binding alive).

**Correct fix:** Use `drop(<expr>)` instead of `let _ = <expr>`. This is the canonical form that clearly signals "intentionally ignoring the result".

This is a recurring pattern across the scenario runner codebase — writer-code should apply `drop(...)` consistently.
