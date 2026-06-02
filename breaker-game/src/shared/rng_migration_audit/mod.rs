//! Structural audit guards for the RNG-domain-partitioning migration (Wave 3).
//!
//! Groups A and D. These tests assert post-migration invariants about which
//! production files may reference `GameRng` and that the architecture doc exists.
//!
//! Exclusion rule (Behavior 8 — meta, no dedicated test):
//! The production file list below EXCLUDES every `.rs` file under any `tests/`
//! directory, files whose name ends in `_tests.rs`, files named `tests.rs`,
//! and all files under `shared/test_utils/`. Those files are ALLOWED to mention
//! `GameRng` (Wave 2 migration-guard tests and test utilities are exempt). The
//! hard-coded list below is a snapshot guard, not a forward-living invariant.

mod file_list;
mod guard_tests;
