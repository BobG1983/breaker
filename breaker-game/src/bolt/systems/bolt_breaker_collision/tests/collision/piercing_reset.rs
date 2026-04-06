//! Tests for `PiercingRemaining` reset on breaker hit via `ActivePiercings`.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::{components::PiercingRemaining, systems::bolt_breaker_collision::tests::helpers::*},
    effect::effects::piercing::ActivePiercings,
};

// --- Piercing reset tests (using ActivePiercings) ---

/// Spec behavior 10: `bolt_breaker_collision` resets `PiercingRemaining` to `ActivePiercings.total()` on breaker hit.
#[test]
fn breaker_hit_resets_piercing_remaining_to_effective_piercing() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    spawn_breaker_at(&mut app, 0.0, y_pos);

    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((ActivePiercings(vec![3]), PiercingRemaining(0)));

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt should have reflected off breaker, got vy={}",
        vel.0.y
    );

    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 3,
        "breaker hit should reset PiercingRemaining to ActivePiercings.total() (3), got {}",
        pr.0
    );
}

/// Spec behavior 10 edge case: `PiercingRemaining(0)` without `ActivePiercings` stays 0.
#[test]
fn piercing_remaining_without_effective_piercing_does_not_reset_on_breaker_hit() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    spawn_breaker_at(&mut app, 0.0, y_pos);

    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);
    app.world_mut().entity_mut(bolt_entity).insert(
        PiercingRemaining(0),
        // No ActivePiercings
    );

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt should have reflected off breaker, got vy={}",
        vel.0.y
    );

    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 0,
        "PiercingRemaining(0) without ActivePiercings should stay at 0 on breaker hit, got {}",
        pr.0
    );
}

/// Behavior 3: `bolt_breaker_collision` resets `PiercingRemaining` from `ActivePiercings.total()`.
///
/// Given: Bolt with `ActivePiercings(vec![2, 1])`, `PiercingRemaining(0)`, no stale cache.
/// When: bolt hits breaker top surface.
/// Then: `PiercingRemaining` = 3 (2 + 1).
#[test]
fn breaker_hit_resets_piercing_remaining_from_active_piercings_total() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    spawn_breaker_at(&mut app, 0.0, y_pos);

    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((ActivePiercings(vec![2, 1]), PiercingRemaining(0)));

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt should have reflected off breaker, got vy={}",
        vel.0.y
    );

    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 3,
        "breaker hit should reset PiercingRemaining to ActivePiercings.total() (2 + 1 = 3), got {}",
        pr.0
    );
}

/// Behavior 4: `bolt_breaker_collision` uses default when no `ActivePiercings`.
///
/// Given: Bolt with `PiercingRemaining(0)`, NO `ActivePiercings`.
/// When: bolt hits breaker top surface.
/// Then: `PiercingRemaining` = 0 (no active piercings).
#[test]
fn breaker_hit_ignores_stale_effective_piercing() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    spawn_breaker_at(&mut app, 0.0, y_pos);

    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);
    app.world_mut().entity_mut(bolt_entity).insert(
        PiercingRemaining(0),
        // NO ActivePiercings — verifies PiercingRemaining stays at 0
    );

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt should have reflected off breaker, got vy={}",
        vel.0.y
    );

    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 0,
        "PiercingRemaining should stay at 0 (no ActivePiercings), got {}",
        pr.0
    );
}
