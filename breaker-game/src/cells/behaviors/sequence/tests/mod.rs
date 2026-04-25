//! Unit and integration tests for the Sequence cell modifier.
//!
//! Split by behavior per `.claude/rules/file-splitting.md`. Shared helpers
//! live in `helpers.rs`.

mod helpers;

mod advance_on_death;
mod builder_attachment;
mod cross_group_isolation;
mod damage_gating;
mod init_groups;
mod single_cell_groups;
mod volatile_combo;
