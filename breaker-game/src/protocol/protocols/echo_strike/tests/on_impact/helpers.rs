use bevy::prelude::*;

use super::super::helpers::seed_active_protocols_with_echo_strike;

pub(super) fn seed_canonical(app: &mut App) {
    // All system-behavior tests use 0.1 oldest (design-doc worked example).
    seed_active_protocols_with_echo_strike(app, 3, 0.5, 0.25, 0.1);
}
