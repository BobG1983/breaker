//! Tests that verify `InvariantKind::ALL` enumeration parity.
//!
//! Reserved for tests that walk every `InvariantKind` variant to confirm the
//! `ALL` slice stays in lock-step with the enum definition (e.g. adding a new
//! variant without adding it to `ALL`). No dedicated parity tests exist yet —
//! coverage-focused walks over `InvariantKind::ALL` live in
//! `self_test_coverage` because their primary assertion target is the
//! missing/covered self-test lists, not enumeration parity itself.
