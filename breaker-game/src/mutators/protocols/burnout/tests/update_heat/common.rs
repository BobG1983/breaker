//! Shared setup helpers for the `burnout_update_heat` test files.

use bevy::prelude::*;

use super::super::helpers::seed_active_protocols_with_burnout;

pub(super) const MOVING: Vec2 = Vec2::new(200.0, 0.0);

pub(super) fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_burnout(app, 4.0, 2.0, 1.5, 4.0, 2.0);
}
