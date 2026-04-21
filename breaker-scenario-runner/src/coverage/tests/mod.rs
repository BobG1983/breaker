//! Coverage module tests — grouped by assertion target.
//!
//! - `self_test_coverage` pins `missing_self_tests` / `covered_self_tests`
//!   discovery.
//! - `layout_coverage` pins `used_layouts` / `unused_layouts` and layout-name
//!   normalization.
//! - `invariant_kind_enumeration` is reserved for future `InvariantKind::ALL`
//!   parity tests (see module doc).
//! - `formatting` pins `format_coverage_report` / `print_coverage_report`
//!   output and return value.

mod helpers;

mod formatting;
mod invariant_kind_enumeration;
mod layout_coverage;
mod self_test_coverage;
