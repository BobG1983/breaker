//! Unit and integration tests for the Armored cell modifier.
//!
//! Split by behavior per `.claude/rules/file-splitting.md`. Shared helpers
//! live in `helpers.rs`. Behavior 27 (cross-plugin registration) lives in
//! `cells/plugin.rs`, not here.

mod helpers;

mod builder_attachment;
mod direction_block;
mod direction_breakthrough;
mod direction_pass_through;
mod facings_and_multihit;
mod validation;
