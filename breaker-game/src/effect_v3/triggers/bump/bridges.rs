//! Bump trigger bridge systems.
//!
//! Each bridge reads a bump message, builds a [`TriggerContext`], and dispatches
//! the corresponding trigger to entities with bound effects.

use bevy::prelude::*;

use crate::effect_v3::types::{Trigger, TriggerContext};

/// Local bridge: fires `Bumped` on the bolt and breaker entities involved in a
/// successful bump of any grade.
pub fn on_bumped() {
    todo!()
}

/// Local bridge: fires `PerfectBumped` on the bolt and breaker entities involved
/// in a perfect-timed bump.
pub fn on_perfect_bumped() {
    todo!()
}

/// Local bridge: fires `EarlyBumped` on the bolt and breaker entities involved
/// in an early-timed bump.
pub fn on_early_bumped() {
    todo!()
}

/// Local bridge: fires `LateBumped` on the bolt and breaker entities involved
/// in a late-timed bump.
pub fn on_late_bumped() {
    todo!()
}

/// Global bridge: fires `BumpOccurred` on all entities with bound effects when
/// any successful bump happens.
pub fn on_bump_occurred() {
    todo!()
}

/// Global bridge: fires `PerfectBumpOccurred` on all entities with bound effects
/// when a perfect bump happens.
pub fn on_perfect_bump_occurred() {
    todo!()
}

/// Global bridge: fires `EarlyBumpOccurred` on all entities with bound effects
/// when an early bump happens.
pub fn on_early_bump_occurred() {
    todo!()
}

/// Global bridge: fires `LateBumpOccurred` on all entities with bound effects
/// when a late bump happens.
pub fn on_late_bump_occurred() {
    todo!()
}

/// Global bridge: fires `BumpWhiffOccurred` on all entities with bound effects
/// when a bump timing window expires without contact.
pub fn on_bump_whiff_occurred() {
    todo!()
}

/// Global bridge: fires `NoBumpOccurred` on all entities with bound effects
/// when a bolt hits the breaker without any bump input.
pub fn on_no_bump_occurred() {
    todo!()
}
