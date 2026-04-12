//! `evaluate_conditions` — per-frame condition polling system for During nodes.

use bevy::prelude::*;

use super::{is_combo_active, is_node_active, is_shield_active};
use crate::effect_v3::types::Condition;

/// Poll all registered conditions each frame and fire/reverse During
/// entries on state transitions.
///
/// Runs in `EffectV3Systems::Conditions`.
///
/// NOTE: The full During state tracking (per-entity, per-source condition
/// state with fire/reverse transitions) is deferred to a future wave.
/// This system currently evaluates each condition and returns the result,
/// but the fire/reverse dispatch requires additional per-entity state
/// infrastructure that is not yet implemented.
pub fn evaluate_conditions(world: &World) {
    // Evaluate global condition values. These are available for the
    // During state machine once per-entity tracking is wired up.
    let _node_active = is_node_active(world);
    let _shield_active = is_shield_active(world);

    // ComboActive conditions are per-threshold, so they'll be evaluated
    // per-entity when the During state machine is implemented.
    // For now this system is a no-op placeholder that compiles and
    // does not panic.
}

/// Helper to evaluate a single condition against world state.
///
/// Used by the future During state machine to check condition transitions.
#[must_use]
pub fn evaluate_condition(condition: &Condition, world: &World) -> bool {
    match condition {
        Condition::Node => is_node_active(world),
        Condition::Shield => is_shield_active(world),
        Condition::Combo(threshold) => is_combo_active(world, *threshold),
    }
}
