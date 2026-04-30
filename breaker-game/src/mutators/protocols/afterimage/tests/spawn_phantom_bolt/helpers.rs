use bevy::prelude::*;

pub(super) use super::super::helpers::{
    build_afterimage_app, build_afterimage_app_no_config, phantom_bolt_count,
    phantom_bolts_owned_by, seed_active_protocols_with_afterimage, spawn_phantom_bolt_entity,
    spawn_phantom_breaker_at, spawn_real_bolt, tick_n, write_bump_performed,
};

pub(super) fn canonical_setup(app: &mut App) -> (Entity, Entity) {
    let phantom = spawn_phantom_breaker_at(app, Vec2::ZERO, 1.5);
    let real = spawn_real_bolt(app, Vec2::ZERO, Vec2::new(0.0, 400.0), 10.0, 6.0);
    (phantom, real)
}
