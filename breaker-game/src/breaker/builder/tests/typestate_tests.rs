use bevy::{math::curve::easing::EaseFunction, prelude::*};

use super::{super::core::*, helpers::test_breaker_definition};
use crate::breaker::components::Breaker;

// ── Behavior 1: Breaker::builder() returns a builder in the fully-unconfigured state ──

#[test]
fn breaker_builder_returns_unconfigured_builder() {
    let _builder: BreakerBuilder<
        NoDimensions,
        NoMovement,
        NoDashing,
        NoSpread,
        NoBump,
        Unvisual,
        NoRole,
    > = Breaker::builder();
}

#[test]
fn breaker_builder_twice_produces_independent_builders() {
    let builder_a = Breaker::builder();
    let builder_b = Breaker::builder();
    let _a = builder_a.dimensions(120.0, 20.0, -250.0);
    let _b = builder_b.dimensions(200.0, 30.0, -300.0);
}

// ── Behavior 2: .dimensions() transitions D from NoDimensions to HasDimensions ──

#[test]
fn dimensions_transitions_to_has_dimensions() {
    let _builder: BreakerBuilder<
        HasDimensions,
        NoMovement,
        NoDashing,
        NoSpread,
        NoBump,
        Unvisual,
        NoRole,
    > = Breaker::builder().dimensions(120.0, 20.0, -250.0);
}

#[test]
fn dimensions_zero_compiles_and_stores_values() {
    let mut world = World::new();
    let bundle = Breaker::builder()
        .dimensions(0.0, 0.0, 0.0)
        .movement(MovementSettings {
            max_speed: 500.0,
            acceleration: 3000.0,
            deceleration: 2500.0,
            decel_ease: EaseFunction::QuadraticIn,
            decel_ease_strength: 1.0,
        })
        .dashing(DashSettings {
            dash: DashParams {
                speed_multiplier: 4.0,
                duration: 0.15,
                tilt_angle: 15.0,
                tilt_ease: EaseFunction::QuadraticInOut,
            },
            brake: BrakeParams {
                tilt_angle: 25.0,
                tilt_duration: 0.2,
                tilt_ease: EaseFunction::CubicInOut,
                decel_multiplier: 2.0,
            },
            settle: SettleParams {
                duration: 0.25,
                tilt_ease: EaseFunction::CubicOut,
            },
        })
        .spread(75.0)
        .bump(BumpSettings {
            perfect_window: 0.15,
            early_window: 0.15,
            late_window: 0.15,
            perfect_cooldown: 0.0,
            weak_cooldown: 0.15,
            feedback: BumpFeedbackSettings {
                duration: 0.15,
                peak: 24.0,
                peak_fraction: 0.3,
                rise_ease: EaseFunction::CubicOut,
                fall_ease: EaseFunction::QuadraticIn,
            },
        })
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();
    // Stub returns only Breaker marker; BaseWidth(0.0) will be missing (test FAILS in RED)
    let bw = world.get::<crate::shared::BaseWidth>(entity);
    assert!(
        bw.is_some(),
        "entity should have BaseWidth(0.0) for zero-dimension breaker"
    );
    assert!(
        (bw.unwrap().0 - 0.0).abs() < f32::EPSILON,
        "BaseWidth should be 0.0"
    );
}

#[test]
fn dimensions_negative_y_stores_value() {
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&test_breaker_definition())
        .with_y_position(-500.0)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();
    let base_y = world.get::<crate::breaker::components::BreakerBaseY>(entity);
    assert!(base_y.is_some(), "entity should have BreakerBaseY(-500.0)");
    assert!(
        (base_y.unwrap().0 - (-500.0)).abs() < f32::EPSILON,
        "BreakerBaseY should be -500.0"
    );
}

// ── Behavior 3: .movement() transitions Mv from NoMovement to HasMovement ──

#[test]
fn movement_transitions_to_has_movement() {
    let _builder: BreakerBuilder<
        NoDimensions,
        HasMovement,
        NoDashing,
        NoSpread,
        NoBump,
        Unvisual,
        NoRole,
    > = Breaker::builder().movement(MovementSettings {
        max_speed: 500.0,
        acceleration: 3000.0,
        deceleration: 2500.0,
        decel_ease: EaseFunction::QuadraticIn,
        decel_ease_strength: 1.0,
    });
}

#[test]
fn movement_zero_max_speed_compiles() {
    let _builder = Breaker::builder().movement(MovementSettings {
        max_speed: 0.0,
        acceleration: 3000.0,
        deceleration: 2500.0,
        decel_ease: EaseFunction::QuadraticIn,
        decel_ease_strength: 1.0,
    });
}

// ── Behavior 4: .dashing() transitions Da from NoDashing to HasDashing ──

#[test]
fn dashing_transitions_to_has_dashing() {
    let _builder: BreakerBuilder<
        NoDimensions,
        NoMovement,
        HasDashing,
        NoSpread,
        NoBump,
        Unvisual,
        NoRole,
    > = Breaker::builder().dashing(DashSettings {
        dash: DashParams {
            speed_multiplier: 4.0,
            duration: 0.15,
            tilt_angle: 15.0,
            tilt_ease: EaseFunction::QuadraticInOut,
        },
        brake: BrakeParams {
            tilt_angle: 25.0,
            tilt_duration: 0.2,
            tilt_ease: EaseFunction::CubicInOut,
            decel_multiplier: 2.0,
        },
        settle: SettleParams {
            duration: 0.25,
            tilt_ease: EaseFunction::CubicOut,
        },
    });
}

// ── Behavior 5: .spread() transitions Sp from NoSpread to HasSpread ──

#[test]
fn spread_transitions_to_has_spread() {
    let _builder: BreakerBuilder<
        NoDimensions,
        NoMovement,
        NoDashing,
        HasSpread,
        NoBump,
        Unvisual,
        NoRole,
    > = Breaker::builder().spread(75.0);
}

#[test]
fn spread_zero_compiles() {
    let _builder = Breaker::builder().spread(0.0);
}

// ── Behavior 6: .bump() transitions Bm from NoBump to HasBump ──

#[test]
fn bump_transitions_to_has_bump() {
    let _builder: BreakerBuilder<
        NoDimensions,
        NoMovement,
        NoDashing,
        NoSpread,
        HasBump,
        Unvisual,
        NoRole,
    > = Breaker::builder().bump(BumpSettings {
        perfect_window: 0.15,
        early_window: 0.15,
        late_window: 0.15,
        perfect_cooldown: 0.0,
        weak_cooldown: 0.15,
        feedback: BumpFeedbackSettings {
            duration: 0.15,
            peak: 24.0,
            peak_fraction: 0.3,
            rise_ease: EaseFunction::CubicOut,
            fall_ease: EaseFunction::QuadraticIn,
        },
    });
}

// ── Behavior 7: .rendered() transitions V from Unvisual to Rendered ──

#[test]
fn rendered_transitions_to_rendered() {
    #[derive(Resource)]
    struct RenderedTypeChecked(bool);

    // We need both Assets<Mesh> and Assets<ColorMaterial> mutably at the same time.
    // Use a system to get them naturally.
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<ColorMaterial>();

    app.insert_resource(RenderedTypeChecked(false));

    app.add_systems(
        Update,
        |mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<ColorMaterial>>,
         mut flag: ResMut<RenderedTypeChecked>| {
            let _builder: BreakerBuilder<
                NoDimensions,
                NoMovement,
                NoDashing,
                NoSpread,
                NoBump,
                Rendered,
                NoRole,
            > = Breaker::builder().rendered(&mut meshes, &mut materials);
            flag.0 = true;
        },
    );
    app.update();

    assert!(
        app.world().resource::<RenderedTypeChecked>().0,
        "rendered typestate transition system should have run"
    );
}

// ── Behavior 8: .headless() transitions V from Unvisual to Headless ──

#[test]
fn headless_transitions_to_headless() {
    let _builder: BreakerBuilder<
        NoDimensions,
        NoMovement,
        NoDashing,
        NoSpread,
        NoBump,
        Headless,
        NoRole,
    > = Breaker::builder().headless();
}

// ── Behavior 9: .primary() transitions R from NoRole to Primary ──

#[test]
fn primary_transitions_to_primary() {
    let _builder: BreakerBuilder<
        NoDimensions,
        NoMovement,
        NoDashing,
        NoSpread,
        NoBump,
        Unvisual,
        Primary,
    > = Breaker::builder().primary();
}

// ── Behavior 10: .extra() transitions R from NoRole to Extra ──

#[test]
fn extra_transitions_to_extra() {
    let _builder: BreakerBuilder<
        NoDimensions,
        NoMovement,
        NoDashing,
        NoSpread,
        NoBump,
        Unvisual,
        Extra,
    > = Breaker::builder().extra();
}
