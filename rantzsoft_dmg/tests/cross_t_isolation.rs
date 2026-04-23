//! P7 crate integration tests — cross-`T` isolation (behaviors 11–15).
//!
//! Each sub-module registers two distinct `Dmgable` marker types (`T1`,
//! `T2`) via `common.rs` and pins that per-`T` pipelines do not bleed into
//! each other. This file is its own integration-test binary; Cargo
//! compiles it separately from `pipeline_integration.rs`.
//!
//! For an integration-test binary root, `mod foo;` looks for
//! `tests/foo.rs` (sibling of this file), not
//! `tests/cross_t_isolation/foo.rs`. Using `#[path = "..."]` attributes
//! keeps the sub-files under `cross_t_isolation/` while resolving modules
//! correctly. Sub-modules do NOT need their own crate-level
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

#[path = "cross_t_isolation/common.rs"]
mod common;

#[path = "cross_t_isolation/damage.rs"]
mod damage;

#[path = "cross_t_isolation/dealer_boost.rs"]
mod dealer_boost;

#[path = "cross_t_isolation/heal.rs"]
mod heal;

#[path = "cross_t_isolation/invulnerable.rs"]
mod invulnerable;

#[path = "cross_t_isolation/kill.rs"]
mod kill;
