---
name: test_file_path_mismatch
description: Code specs frequently cite wrong test file paths that don't match the test spec's Constraints section — always cross-check
type: feedback
---

Code specs list failing test files in "Failing Tests" section. This path must exactly match the path in the test spec's "Constraints → Tests go in:" line.

Common mismatch pattern: code spec says `tests/fire_tests.rs` (directory module) but test spec says `tests.rs` (flat file). Rust resolves `mod tests;` to EITHER `tests.rs` OR `tests/mod.rs`, not to `tests/fire_tests.rs` directly.

**Why:** Seen in wave1c-spawn-phantom-fifo-code.md vs wave1c-spawn-phantom-fifo-tests.md. The code spec said `tests/fire_tests.rs`; the test spec said `tests.rs`. If writer-tests creates `tests.rs` and writer-code points to `tests/fire_tests.rs`, the failing-tests pointer is wrong and writer-code may not read the right tests.

**How to apply:** In every implementation spec review, read the test spec's Constraints section and compare the file path verbatim against the code spec's "Failing Tests" section. Flag any path that doesn't match exactly.
