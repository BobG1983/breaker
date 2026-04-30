use bevy::prelude::*;

use super::super::helpers::seed_active_protocols_with_reckless_dash;

pub(super) fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_reckless_dash(app, 0.7, 4.0, true);
}
