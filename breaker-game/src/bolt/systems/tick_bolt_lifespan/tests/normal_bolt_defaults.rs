//! Non-phantom default behavior + birthing-filter tests:
//! Beh 3 — T16: default Despawn for non-phantom bolt
//! Beh 4 — Birthing filter preserved

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::components::{ExtraBolt, LifetimeEndBehavior},
    prelude::*,
    shared::Lifespan,
};

// ── Behavior 3 — T16: default Despawn for non-phantom bolt ──────────────────

/// T16 primary: non-phantom bolt with `Lifespan` and NO `LifetimeEndBehavior`
/// defaults to the Despawn branch — emits one `DespawnEntity`.
#[test]
fn non_phantom_bolt_no_behavior_defaults_to_despawn_entity() {
    let mut app = lifespan_emit_app();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, 0.0)),
            Lifespan { remaining: 0.01 },
        ))
        .id();

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        collector.0.len(),
        1,
        "no LifetimeEndBehavior defaults to Despawn; expected one DespawnEntity, got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].entity, bolt);

    let ky = app.world().resource::<CapturedKillYourselfBolt>();
    assert!(ky.0.is_empty(), "zero KillYourself<Bolt>");

    let lifespan = app
        .world()
        .get::<Lifespan>(bolt)
        .expect("Lifespan still on entity");
    let expected = 0.01_f32 - FIXED_DT;
    assert!(
        (lifespan.remaining - expected).abs() < 1e-4,
        "Lifespan.remaining ≈ {expected}, got {}",
        lifespan.remaining
    );

    assert!(
        app.world().get::<Bolt>(bolt).is_some(),
        "Bolt marker still present (system does not strip)"
    );
}

/// T16 edge 3a: `Lifespan { remaining: 0.0 }` boundary, no `LifetimeEndBehavior`
/// — exactly one `DespawnEntity` emitted.
#[test]
fn non_phantom_bolt_at_zero_boundary_defaults_to_despawn_entity() {
    let mut app = lifespan_emit_app();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, 0.0)),
            Lifespan { remaining: 0.0 },
        ))
        .id();

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        collector.0.len(),
        1,
        "zero-boundary with no behavior must emit one DespawnEntity"
    );
    assert_eq!(collector.0[0].entity, bolt);
}

/// T16 edge 3b: `Lifespan { remaining: 1.0 }`, no `LifetimeEndBehavior` —
/// pre-expiry tick is a no-op; `Lifespan.remaining` decremented, zero
/// `DespawnEntity`.
#[test]
fn non_phantom_bolt_pre_expiry_no_behavior_is_noop() {
    let mut app = lifespan_emit_app();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, 0.0)),
            Lifespan { remaining: 1.0 },
        ))
        .id();

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(collector.0.is_empty(), "no DespawnEntity before expiry");
    assert!(app.world().get_entity(bolt).is_ok());

    let lifespan = app.world().get::<Lifespan>(bolt).expect("Lifespan present");
    let expected = 1.0_f32 - FIXED_DT;
    assert!(
        (lifespan.remaining - expected).abs() < 1e-4,
        "Lifespan.remaining ≈ {expected}, got {}",
        lifespan.remaining
    );
}

/// T16 edge 3c: two bolts — one expires, one does not. Only the short-lifespan
/// bolt emits `DespawnEntity`.
#[test]
fn two_bolts_mixed_expiry_only_short_lifespan_emits_despawn_entity() {
    let mut app = lifespan_emit_app();
    let short_bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, 0.0)),
            Lifespan { remaining: 0.01 },
        ))
        .id();
    let long_bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(10.0, 0.0)),
            Lifespan { remaining: 1.0 },
        ))
        .id();

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        collector.0.len(),
        1,
        "exactly one DespawnEntity for two bolts with different lifespans"
    );
    assert_eq!(
        collector.0[0].entity, short_bolt,
        "DespawnEntity must target the short-lifespan bolt"
    );

    assert!(
        app.world().get_entity(long_bolt).is_ok(),
        "long-lifespan bolt still alive"
    );
    let long_lifespan = app
        .world()
        .get::<Lifespan>(long_bolt)
        .expect("Lifespan present");
    let expected = 1.0_f32 - FIXED_DT;
    assert!(
        (long_lifespan.remaining - expected).abs() < 1e-4,
        "long bolt Lifespan.remaining ≈ {expected}, got {}",
        long_lifespan.remaining
    );
}

// ── Behavior 4 — Birthing filter preserved ───────────────────────────────────

/// Behavior 4 primary: birthing bolt with expired `Lifespan` emits ZERO
/// `DespawnEntity`; `Lifespan.remaining` is UNCHANGED (system skips entirely).
#[test]
fn tick_bolt_lifespan_skips_birthing_bolt_entirely() {
    let mut app = lifespan_emit_app();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::ZERO),
            Lifespan { remaining: 0.01 },
            LifetimeEndBehavior::Despawn,
            test_birthing(),
        ))
        .id();

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(
        collector.0.is_empty(),
        "birthing bolt must emit zero DespawnEntity"
    );

    let lifespan = app.world().get::<Lifespan>(bolt).expect("Lifespan present");
    assert!(
        (lifespan.remaining - 0.01).abs() < f32::EPSILON,
        "Lifespan.remaining must be UNCHANGED for birthing bolt; got {}",
        lifespan.remaining
    );
}

/// Behavior 4 edge 4a: birthing + non-birthing coexist — only non-birthing bolt
/// emits `DespawnEntity`; birthing bolt's `Lifespan.remaining` unchanged.
#[test]
fn tick_bolt_lifespan_skips_birthing_processes_non_birthing() {
    let mut app = lifespan_emit_app();
    let birthing_bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::ZERO),
            Lifespan { remaining: 0.01 },
            LifetimeEndBehavior::Despawn,
            test_birthing(),
        ))
        .id();
    let normal_bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(5.0, 0.0)),
            Lifespan { remaining: 0.01 },
            LifetimeEndBehavior::Despawn,
        ))
        .id();

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        collector.0.len(),
        1,
        "exactly one DespawnEntity — the non-birthing bolt"
    );
    assert_eq!(
        collector.0[0].entity, normal_bolt,
        "DespawnEntity must be for the non-birthing bolt"
    );

    let birthing_lifespan = app
        .world()
        .get::<Lifespan>(birthing_bolt)
        .expect("Lifespan present on birthing bolt");
    assert!(
        (birthing_lifespan.remaining - 0.01).abs() < f32::EPSILON,
        "birthing bolt Lifespan.remaining must be UNCHANGED; got {}",
        birthing_lifespan.remaining
    );
}
