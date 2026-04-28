//! P7 crate integration tests — full pipeline, kills, heals, wiring, edges.
//!
//! This file is the top-level integration-test binary. Cargo compiles it as
//! a single binary. Sub-modules `damage`, `heal`, `kills`, `wiring`, `edge`
//! resolve to `tests/pipeline_integration/<name>.rs` via Rust 2018+
//! non-`mod.rs`-style module resolution (the child directory is named after
//! the parent file's stem).
//!
//! Per the P7 spec, sub-modules do NOT need their own crate-level
//! `#![cfg_attr(...)]` header — only this file does, because it is the
//! binary crate root.

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "integration tests use unwrap/expect/panic for assertion clarity"
    )
)]

// For an integration-test binary root, `mod foo;` looks for `tests/foo.rs`
// (sibling of this file), not `tests/pipeline_integration/foo.rs`. Using
// `#[path = "..."]` attributes keeps the five sub-files under
// `pipeline_integration/` as the spec prescribes while resolving the
// modules correctly.

#[path = "pipeline_integration/damage/mod.rs"]
mod damage;

#[path = "pipeline_integration/edge.rs"]
mod edge;

#[path = "pipeline_integration/filter_boost/mod.rs"]
mod filter_boost;

#[path = "pipeline_integration/filter_vulnerable.rs"]
mod filter_vulnerable;

#[path = "pipeline_integration/heal.rs"]
mod heal;

#[path = "pipeline_integration/kills.rs"]
mod kills;

#[path = "pipeline_integration/wiring.rs"]
mod wiring;
