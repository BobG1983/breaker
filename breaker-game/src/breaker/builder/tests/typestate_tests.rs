use bevy::prelude::*;

use super::{super::core::*, helpers::test_breaker_definition};
use crate::breaker::{components::Breaker, definition::BreakerDefinition};

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
    let defaults = BreakerDefinition::default();
    let mut world = World::new();
    let bundle = Breaker::builder()
        .dimensions(0.0, 0.0, 0.0)
        .movement(MovementSettings {
            max_speed: defaults.max_speed,
            acceleration: defaults.acceleration,
            deceleration: defaults.deceleration,
            decel_ease: defaults.decel_ease,
            decel_ease_strength: defaults.decel_ease_strength,
        })
        .dashing(DashSettings {
            dash: DashParams {
                speed_multiplier: defaults.dash_speed_multiplier,
                duration: defaults.dash_duration,
                tilt_angle: defaults.dash_tilt_angle,
                tilt_ease: defaults.dash_tilt_ease,
            },
            brake: BrakeParams {
                tilt_angle: defaults.brake_tilt_angle,
                tilt_duration: defaults.brake_tilt_duration,
                tilt_ease: defaults.brake_tilt_ease,
                decel_multiplier: defaults.brake_decel_multiplier,
            },
            settle: SettleParams {
                duration: defaults.settle_duration,
                tilt_ease: defaults.settle_tilt_ease,
            },
        })
        .spread(defaults.reflection_spread)
        .bump(BumpSettings {
            perfect_window: defaults.perfect_window,
            early_window: defaults.early_window,
            late_window: defaults.late_window,
            perfect_cooldown: defaults.perfect_bump_cooldown,
            weak_cooldown: defaults.weak_bump_cooldown,
            feedback: BumpFeedbackSettings {
                duration: defaults.bump_visual_duration,
                peak: defaults.bump_visual_peak,
                peak_fraction: defaults.bump_visual_peak_fraction,
                rise_ease: defaults.bump_visual_rise_ease,
                fall_ease: defaults.bump_visual_fall_ease,
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
    let defaults = BreakerDefinition::default();
    let _builder: BreakerBuilder<
        NoDimensions,
        HasMovement,
        NoDashing,
        NoSpread,
        NoBump,
        Unvisual,
        NoRole,
    > = Breaker::builder().movement(MovementSettings {
        max_speed: defaults.max_speed,
        acceleration: defaults.acceleration,
        deceleration: defaults.deceleration,
        decel_ease: defaults.decel_ease,
        decel_ease_strength: defaults.decel_ease_strength,
    });
}

#[test]
fn movement_zero_max_speed_compiles() {
    let defaults = BreakerDefinition::default();
    let _builder = Breaker::builder().movement(MovementSettings {
        max_speed: 0.0,
        acceleration: defaults.acceleration,
        deceleration: defaults.deceleration,
        decel_ease: defaults.decel_ease,
        decel_ease_strength: defaults.decel_ease_strength,
    });
}

// ── Behavior 4: .dashing() transitions Da from NoDashing to HasDashing ──

#[test]
fn dashing_transitions_to_has_dashing() {
    let defaults = BreakerDefinition::default();
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
            speed_multiplier: defaults.dash_speed_multiplier,
            duration: defaults.dash_duration,
            tilt_angle: defaults.dash_tilt_angle,
            tilt_ease: defaults.dash_tilt_ease,
        },
        brake: BrakeParams {
            tilt_angle: defaults.brake_tilt_angle,
            tilt_duration: defaults.brake_tilt_duration,
            tilt_ease: defaults.brake_tilt_ease,
            decel_multiplier: defaults.brake_decel_multiplier,
        },
        settle: SettleParams {
            duration: defaults.settle_duration,
            tilt_ease: defaults.settle_tilt_ease,
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
    let defaults = BreakerDefinition::default();
    let _builder: BreakerBuilder<
        NoDimensions,
        NoMovement,
        NoDashing,
        NoSpread,
        HasBump,
        Unvisual,
        NoRole,
    > = Breaker::builder().bump(BumpSettings {
        perfect_window: defaults.perfect_window,
        early_window: defaults.early_window,
        late_window: defaults.late_window,
        perfect_cooldown: defaults.perfect_bump_cooldown,
        weak_cooldown: defaults.weak_bump_cooldown,
        feedback: BumpFeedbackSettings {
            duration: defaults.bump_visual_duration,
            peak: defaults.bump_visual_peak,
            peak_fraction: defaults.bump_visual_peak_fraction,
            rise_ease: defaults.bump_visual_rise_ease,
            fall_ease: defaults.bump_visual_fall_ease,
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
