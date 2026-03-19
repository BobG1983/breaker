---
name: writer-tests must only write tests, never implement
description: The writer-tests agent should ONLY write failing tests and stub signatures — never implement system bodies or any production code
type: feedback
---

Promoted to `.claude/rules/tdd.md` — see RED/GREEN Boundary section.

The writer-tests agent must ONLY write failing tests. It should:
- Write `#[cfg(test)] mod tests` blocks with concrete assertions
- Write stub function signatures that compile but do nothing (empty body or `let _ = ...`)
- NEVER fill in production logic — that's the writer-code's job

**Why:** TDD discipline requires a clean red->green cycle. If writer-tests implements anything, writer-code has nothing to do and the TDD feedback loop is broken.

**How to apply:** When writing writer-tests specs, explicitly state "write FAILING tests and compilable stubs only — do NOT implement any production logic."
