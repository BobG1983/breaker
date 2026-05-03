use bevy::prelude::*;

use super::super::helpers::{
    build_afterimage_app, phantom_bolts_owned_by, seed_active_protocols_with_afterimage,
    spawn_phantom_breaker_at, spawn_real_bolt, write_bump_performed,
};
use crate::{breaker::messages::BumpGrade, prelude::*};

// ── I14 — spawn_phantom_bolt runs BEFORE tick_phantom_lifetime ────────────

#[test]
fn spawn_phantom_bolt_runs_before_tick_phantom_lifetime_in_same_tick() {
    use crate::effect_v3::effects::phantom_bolt::components::PhantomLifetime;

    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 10.0, 6.0);

    write_bump_performed(&mut app, phantom, Some(real_bolt), BumpGrade::Perfect);
    tick(&mut app);

    let owned = phantom_bolts_owned_by(&mut app, real_bolt);
    assert_eq!(owned.len(), 1);
    let lifetime = app.world().get::<PhantomLifetime>(owned[0]).unwrap().0;
    let expected = 3.0 - 1.0 / 64.0;
    assert!(
        (lifetime - expected).abs() < 1e-4,
        "spawn runs BEFORE tick_phantom_lifetime → lifetime must be ~{expected} \
         (one tick decrement), got {lifetime}"
    );
}
