//! Section D — integration tests for `apply_damage_to_cells`.
//!
//! Pin the Diffusion redistribution branch inside the cells-domain Cell damage
//! system. The non-Diffusion semantics mirror the generic `apply_damage::<T>`
//! tests for `TestEntity` in `shared/death_pipeline/systems/tests/apply_damage.rs`;
//! these tests exercise the Cell-specific path plus the Diffusion BFS branch.
//!
//! Test helpers (`PendingCellDamage`, `enqueue_cell_damage`) are copied locally
//! from `shared/death_pipeline/systems/tests/helpers.rs` (Option A per the
//! spec): the upstream items are `pub(super)` and not importable from the
//! cells-domain test module. Low-duplication cost for the decoupling benefit.

mod helpers;

mod adjacency;
mod config_gating;
mod depth_bfs;
mod edge_cases;
mod multi_message;
mod share_split;
