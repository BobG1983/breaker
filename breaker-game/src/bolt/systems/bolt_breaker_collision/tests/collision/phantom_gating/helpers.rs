//! Local test helpers for phantom-gating tests.

use crate::bolt::systems::bolt_breaker_collision::tests::helpers::*;

pub(super) fn breaker_y() -> f32 {
    -250.0
}

pub(super) fn start_y_above(breaker_cy: f32) -> f32 {
    let hh = default_breaker_height();
    breaker_cy + hh.half_height() + default_bolt_radius().0 + 3.0
}
