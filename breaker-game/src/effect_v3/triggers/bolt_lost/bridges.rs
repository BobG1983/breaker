//! Bolt lost trigger bridge system.
//!
//! Reads `BoltLost` messages and dispatches `BoltLostOccurred` triggers
//! to all entities with bound effects.

use bevy::prelude::*;

use crate::effect_v3::types::{Trigger, TriggerContext};

/// Global bridge: fires `BoltLostOccurred` on all entities with bound effects
/// when a bolt is lost.
pub fn on_bolt_lost_occurred() {
    todo!()
}
