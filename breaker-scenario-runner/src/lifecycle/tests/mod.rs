// Re-import all items from the parent (lifecycle) into the tests module scope.
// Because `tests` is a direct child of `lifecycle`, this includes private items
// (e.g., private `use` imports like `InputActions`, `ViolationLog`, etc.).
// Sub-modules then access these through `super::*` or via `helpers.rs` re-exports.
use super::*;

mod bypass_menu;
mod chip_select;
mod debug_setup;
mod entity_tagging;
mod force_bump_grade;
mod frame_counter;
mod frame_limit;
mod frame_mutations;
mod frozen_positions;
mod helpers;
mod initial_effects;
mod input_injection;
mod invariant_gating;
mod perfect_tracking;
mod playing_gating;
mod sentinels;
