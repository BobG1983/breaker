---
name: never_run_cargo
description: writer-tests must NEVER run cargo commands — only runner agents run cargo
type: feedback
---

**NEVER run cargo commands.** No `cargo check`, `cargo test`, `cargo build`, `cargo clippy`, or any cargo alias (`dcheck`, `dtest`, `dclippy`, etc.).

**Why:** The TDD pipeline separates concerns strictly. writer-tests writes tests and stubs. runner-tests runs the tests. If writer-tests runs cargo, it can mask issues, create build artifacts that interfere with other agents, and violate the separation of concerns. The orchestrator runs the RED gate via runner-tests after writer-tests completes.

**How to apply:** When tempted to verify compilation or run tests, DON'T. Write the code, trust the patterns from memory files, and let the orchestrator's RED gate catch any issues. Use `ls` and `Read` to check file existence and structure — never Bash with cargo.

**What to use Bash for:** Only `ls` to check directory/file existence. Nothing else.
