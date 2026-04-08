---
name: Stray #[test] attribute on helper function
description: writer-tests placed #[test] immediately before a comment block and helper fn, causing a compile error (attribute applied to non-test fn)
type: feedback
---

A stray `#[test]` line was placed before a comment separator and helper function `test_birthing()` in pull_tests.rs. The comment line between the attribute and the fn broke the intended connection, but Rust still attaches `#[test]` to the following item (the helper fn), which is not a valid test signature — it returns a value and takes no parameters. This causes a compile error at the RED gate.

**Why:** The writer appears to have intended to mark a section comment as a boundary, then write the helper, but accidentally placed the `#[test]` attribute before the comment rather than after the helper declaration.

**How to apply:** When reviewing, check that every `#[test]` attribute is immediately followed (after any doc-comments, not regular comments) by a `fn test_name()` with no return type. A `#[test]` before a `fn helper() -> SomeType` is always a compile error.
