//! Time trigger bridge system.
//!
//! Reads [`EffectTimerExpired`] messages and dispatches `TimeExpires` triggers
//! on the entity that owned the expired timer.

use bevy::prelude::*;

use super::messages::EffectTimerExpired;
use crate::effect_v3::types::{Trigger, TriggerContext};

/// Self bridge: fires `TimeExpires(original_duration)` on the entity whose
/// timer expired.
pub fn on_time_expires() {
    todo!()
}
