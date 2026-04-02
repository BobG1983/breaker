//! Breaker domain `QueryData` structs — named-field query types.

use bevy::{ecs::query::QueryData, prelude::*};
use rantzsoft_spatial2d::components::{
    MaxSpeed, Position2D, PreviousPosition, Scale2D, Velocity2D,
};

use crate::{
    breaker::components::{
        BaseHeight, BaseWidth, BrakeDecel, BrakeTilt, BreakerAcceleration, BreakerBaseY,
        BreakerDeceleration, BreakerReflectionSpread, BreakerTilt, BumpEarlyWindow, BumpLateWindow,
        BumpPerfectCooldown, BumpPerfectWindow, BumpState, BumpWeakCooldown, DashDuration,
        DashSpeedMultiplier, DashState, DashStateTimer, DashTilt, DashTiltEase, DecelEasing,
        SettleDuration, SettleTiltEase,
    },
    effect::{
        AnchorActive, AnchorPlanted,
        effects::{
            flash_step::FlashStepActive, size_boost::ActiveSizeBoosts,
            speed_boost::ActiveSpeedBoosts,
        },
    },
    shared::NodeScalingFactor,
};

// ── QueryData structs ───────────────────────────────────────────────────

/// Breaker collision data for bolt-breaker collision (read-only).
#[derive(QueryData)]
pub(crate) struct BreakerCollisionData {
    /// World position.
    pub position: &'static Position2D,
    /// Current tilt angle.
    pub tilt: &'static BreakerTilt,
    /// Base width in world units.
    pub base_width: &'static BaseWidth,
    /// Base height in world units.
    pub base_height: &'static BaseHeight,
    /// Maximum reflection angle.
    pub reflection_spread: &'static BreakerReflectionSpread,
    /// Active size boost multipliers.
    pub size_boosts: Option<&'static ActiveSizeBoosts>,
    /// Node scaling factor.
    pub node_scale: Option<&'static NodeScalingFactor>,
}

/// Breaker entity data for cell/wall collision (read-only).
#[derive(QueryData)]
pub(crate) struct BreakerSizeData {
    /// The breaker entity.
    pub entity: Entity,
    /// World position.
    pub position: &'static Position2D,
    /// Base width in world units.
    pub base_width: &'static BaseWidth,
    /// Base height in world units.
    pub base_height: &'static BaseHeight,
    /// Node scaling factor.
    pub node_scale: Option<&'static NodeScalingFactor>,
    /// Active size boost multipliers.
    pub size_boosts: Option<&'static ActiveSizeBoosts>,
}

/// Breaker movement data — mutable position/velocity, read-only config.
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BreakerMovementData {
    /// Mutable world position.
    pub position: &'static mut Position2D,
    /// Mutable velocity.
    pub velocity: &'static mut Velocity2D,
    /// Current dash state (read-only).
    pub state: &'static DashState,
    /// Maximum movement speed.
    pub max_speed: &'static MaxSpeed,
    /// Input acceleration rate.
    pub acceleration: &'static BreakerAcceleration,
    /// Deceleration rate.
    pub deceleration: &'static BreakerDeceleration,
    /// Deceleration easing parameters.
    pub decel_easing: &'static DecelEasing,
    /// Base width for playfield clamping.
    pub base_width: &'static BaseWidth,
    /// Active speed boost multipliers.
    pub speed_boosts: Option<&'static ActiveSpeedBoosts>,
    /// Active size boost multipliers.
    pub size_boosts: Option<&'static ActiveSizeBoosts>,
}

/// Breaker dash state machine data — full state, velocity, tilt, and all timing params.
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BreakerDashData {
    /// Mutable dash state.
    pub state: &'static mut DashState,
    /// Mutable velocity.
    pub velocity: &'static mut Velocity2D,
    /// Mutable tilt.
    pub tilt: &'static mut BreakerTilt,
    /// Mutable dash state timer.
    pub timer: &'static mut DashStateTimer,
    /// Maximum movement speed.
    pub max_speed: &'static MaxSpeed,
    /// Deceleration rate.
    pub deceleration: &'static BreakerDeceleration,
    /// Deceleration easing parameters.
    pub decel_easing: &'static DecelEasing,
    /// Dash speed multiplier.
    pub dash_speed: &'static DashSpeedMultiplier,
    /// Dash duration in seconds.
    pub dash_duration: &'static DashDuration,
    /// Dash tilt angle.
    pub dash_tilt: &'static DashTilt,
    /// Dash tilt easing function.
    pub dash_tilt_ease: &'static DashTiltEase,
    /// Brake tilt configuration.
    pub brake_tilt: &'static BrakeTilt,
    /// Brake deceleration multiplier.
    pub brake_decel: &'static BrakeDecel,
    /// Settle duration in seconds.
    pub settle_duration: &'static SettleDuration,
    /// Settle tilt easing function.
    pub settle_tilt_ease: &'static SettleTiltEase,
    /// Flash step active marker.
    pub flash_step: Option<&'static FlashStepActive>,
    /// Mutable position (optional — for flash step teleport).
    pub position: Option<&'static mut Position2D>,
    /// Base width (optional — for flash step playfield clamping).
    pub base_width: Option<&'static BaseWidth>,
    /// Active speed boost multipliers.
    pub speed_boosts: Option<&'static ActiveSpeedBoosts>,
    /// Active size boost multipliers.
    pub size_boosts: Option<&'static ActiveSizeBoosts>,
}

/// Breaker reset data — mutable state cleared at node start.
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BreakerResetData {
    /// Mutable world position.
    pub position: &'static mut Position2D,
    /// Mutable dash state.
    pub state: &'static mut DashState,
    /// Mutable velocity.
    pub velocity: &'static mut Velocity2D,
    /// Mutable tilt.
    pub tilt: &'static mut BreakerTilt,
    /// Mutable dash state timer.
    pub timer: &'static mut DashStateTimer,
    /// Mutable bump state.
    pub bump: &'static mut BumpState,
    /// Base Y position (read-only).
    pub base_y: &'static BreakerBaseY,
    /// Previous position snapshot (optional, mutable).
    pub prev_position: Option<&'static mut PreviousPosition>,
}

/// Bump timing window data — state, timing/cooldown params.
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BreakerBumpTimingData {
    /// Mutable bump state.
    pub bump: &'static mut BumpState,
    /// Perfect bump window duration.
    pub perfect_window: &'static BumpPerfectWindow,
    /// Early bump window duration.
    pub early_window: &'static BumpEarlyWindow,
    /// Late bump window duration.
    pub late_window: &'static BumpLateWindow,
    /// Perfect bump cooldown duration.
    pub perfect_cooldown: &'static BumpPerfectCooldown,
    /// Weak bump cooldown duration.
    pub weak_cooldown: &'static BumpWeakCooldown,
    /// Anchor planted marker.
    pub anchor_planted: Option<&'static AnchorPlanted>,
    /// Anchor active configuration.
    pub anchor_active: Option<&'static AnchorActive>,
}

/// Bump grading data — state, timing windows, and cooldowns for `grade_bump`.
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BreakerBumpGradingData {
    /// Mutable bump state.
    pub bump: &'static mut BumpState,
    /// Perfect bump window duration.
    pub perfect_window: &'static BumpPerfectWindow,
    /// Late bump window duration.
    pub late_window: &'static BumpLateWindow,
    /// Perfect bump cooldown duration.
    pub perfect_cooldown: &'static BumpPerfectCooldown,
    /// Weak bump cooldown duration.
    pub weak_cooldown: &'static BumpWeakCooldown,
    /// Anchor planted marker.
    pub anchor_planted: Option<&'static AnchorPlanted>,
    /// Anchor active configuration.
    pub anchor_active: Option<&'static AnchorActive>,
}

/// Breaker data for the `sync_breaker_scale` system.
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct SyncBreakerScaleData {
    /// Base width in world units.
    pub base_width: &'static BaseWidth,
    /// Base height in world units.
    pub base_height: &'static BaseHeight,
    /// Mutable scale for rendering.
    pub scale: &'static mut Scale2D,
    /// Active size boost multipliers.
    pub size_boosts: Option<&'static ActiveSizeBoosts>,
    /// Node scaling factor.
    pub node_scale: Option<&'static NodeScalingFactor>,
    /// Minimum width constraint.
    pub min_w: Option<&'static crate::shared::size::MinWidth>,
    /// Maximum width constraint.
    pub max_w: Option<&'static crate::shared::size::MaxWidth>,
    /// Minimum height constraint.
    pub min_h: Option<&'static crate::shared::size::MinHeight>,
    /// Maximum height constraint.
    pub max_h: Option<&'static crate::shared::size::MaxHeight>,
}

/// Breaker bump telemetry — state, bump, tilt, velocity, and window sizes.
#[cfg(feature = "dev")]
#[derive(QueryData)]
pub(crate) struct BreakerTelemetryData {
    /// Current dash state.
    pub state: &'static DashState,
    /// Bump state.
    pub bump: &'static BumpState,
    /// Current tilt.
    pub tilt: &'static BreakerTilt,
    /// Current velocity.
    pub velocity: &'static Velocity2D,
    /// Perfect bump window duration.
    pub perfect_window: &'static BumpPerfectWindow,
    /// Early bump window duration.
    pub early_window: &'static BumpEarlyWindow,
    /// Late bump window duration.
    pub late_window: &'static BumpLateWindow,
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use bevy::{math::curve::easing::EaseFunction, prelude::*};
    use rantzsoft_spatial2d::components::{
        MaxSpeed, Position2D, PreviousPosition, Scale2D, Velocity2D,
    };

    use super::*;
    use crate::{
        breaker::components::{
            BaseHeight, BaseWidth, BrakeDecel, BrakeTilt, Breaker, BreakerAcceleration,
            BreakerBaseY, BreakerDeceleration, BreakerReflectionSpread, BreakerTilt,
            BumpEarlyWindow, BumpLateWindow, BumpPerfectCooldown, BumpPerfectWindow, BumpState,
            BumpWeakCooldown, DashDuration, DashSpeedMultiplier, DashState, DashStateTimer,
            DashTilt, DashTiltEase, DecelEasing, SettleDuration, SettleTiltEase,
        },
        effect::{
            AnchorActive, AnchorPlanted,
            effects::{
                flash_step::FlashStepActive, size_boost::ActiveSizeBoosts,
                speed_boost::ActiveSpeedBoosts,
            },
        },
        shared::{
            NodeScalingFactor,
            size::{MaxHeight, MaxWidth, MinHeight, MinWidth},
        },
    };

    /// Tracks whether a test system's query loop body actually executed.
    #[derive(Resource, Default)]
    struct QueryMatched(bool);

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<QueryMatched>();
        app
    }

    /// Asserts the system's query matched at least one entity.
    fn assert_query_matched(app: &App) {
        assert!(
            app.world().resource::<QueryMatched>().0,
            "QueryData system should have matched at least one entity"
        );
    }

    /// Accumulates one fixed timestep then runs one update.
    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── Part A: BreakerCollisionData (read-only) ────────────────────

    // Behavior 1: BreakerCollisionData can be queried with all required components
    #[test]
    fn breaker_collision_data_query_matches_with_required_components() {
        let mut app = test_app();
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(0.0, -200.0)),
            BreakerTilt::default(),
            BaseWidth(120.0),
            BaseHeight(20.0),
            BreakerReflectionSpread(0.5),
        ));

        let mut query = app
            .world_mut()
            .query_filtered::<BreakerCollisionData, With<Breaker>>();
        let results: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(
            results.len(),
            1,
            "expected exactly 1 result from BreakerCollisionData query"
        );
        let data = &results[0];
        assert_eq!(data.position.0, Vec2::new(0.0, -200.0));
        assert!((data.tilt.angle - 0.0).abs() < f32::EPSILON);
        assert!((data.base_width.0 - 120.0).abs() < f32::EPSILON);
        assert!((data.base_height.0 - 20.0).abs() < f32::EPSILON);
        assert!((data.reflection_spread.0 - 0.5).abs() < f32::EPSILON);
        assert!(data.size_boosts.is_none());
        assert!(data.node_scale.is_none());
    }

    // Behavior 1 edge case: entity missing BreakerReflectionSpread -- query returns 0 results
    #[test]
    fn breaker_collision_data_query_skips_entity_missing_required_component() {
        let mut app = test_app();
        // Spawn entity WITHOUT BreakerReflectionSpread -- should not match
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(0.0, -200.0)),
            BreakerTilt::default(),
            BaseWidth(120.0),
            BaseHeight(20.0),
        ));

        let count = app
            .world_mut()
            .query_filtered::<BreakerCollisionData, With<Breaker>>()
            .iter(app.world())
            .count();
        assert_eq!(
            count, 0,
            "entity missing BreakerReflectionSpread should not match BreakerCollisionData"
        );
    }

    // Behavior 2: BreakerCollisionData with optional components present
    #[test]
    fn breaker_collision_data_query_includes_optional_components() {
        let mut app = test_app();
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(0.0, -200.0)),
            BreakerTilt::default(),
            BaseWidth(120.0),
            BaseHeight(20.0),
            BreakerReflectionSpread(0.5),
            ActiveSizeBoosts(vec![1.5]),
            NodeScalingFactor(0.8),
        ));

        let mut query = app
            .world_mut()
            .query_filtered::<BreakerCollisionData, With<Breaker>>();
        let data = query.single(app.world()).unwrap();
        assert!(
            data.size_boosts.is_some(),
            "ActiveSizeBoosts should be Some"
        );
        assert!((data.node_scale.unwrap().0 - 0.8).abs() < f32::EPSILON);
    }

    // Behavior 2 edge case: only one optional component present
    #[test]
    fn breaker_collision_data_query_partial_optionals() {
        let mut app = test_app();
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(0.0, -200.0)),
            BreakerTilt::default(),
            BaseWidth(120.0),
            BaseHeight(20.0),
            BreakerReflectionSpread(0.5),
            NodeScalingFactor(0.8),
            // No ActiveSizeBoosts
        ));

        let mut query = app
            .world_mut()
            .query_filtered::<BreakerCollisionData, With<Breaker>>();
        let data = query.single(app.world()).unwrap();
        assert!(
            data.node_scale.is_some(),
            "NodeScalingFactor should be Some"
        );
        assert!(
            data.size_boosts.is_none(),
            "ActiveSizeBoosts should be None"
        );
    }

    // ── Part B: BreakerSizeData (read-only) ─────────────────────────

    // Behavior 3: BreakerSizeData can be queried
    #[test]
    fn breaker_size_data_query_matches_with_all_components() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                Position2D(Vec2::new(50.0, -200.0)),
                BaseWidth(120.0),
                BaseHeight(20.0),
                ActiveSizeBoosts(vec![1.5]),
                NodeScalingFactor(0.8),
            ))
            .id();

        let mut query = app
            .world_mut()
            .query_filtered::<BreakerSizeData, With<Breaker>>();
        let results: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(
            results.len(),
            1,
            "expected exactly 1 result from BreakerSizeData query"
        );
        let data = &results[0];
        assert_eq!(data.entity, entity);
        assert_eq!(data.position.0, Vec2::new(50.0, -200.0));
        assert!((data.base_width.0 - 120.0).abs() < f32::EPSILON);
        assert!((data.base_height.0 - 20.0).abs() < f32::EPSILON);
        assert!(data.size_boosts.is_some());
        assert!((data.node_scale.unwrap().0 - 0.8).abs() < f32::EPSILON);
    }

    // Behavior 3 edge case: optionals absent, query still matches
    #[test]
    fn breaker_size_data_query_matches_without_optionals() {
        let mut app = test_app();
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(50.0, -200.0)),
            BaseWidth(120.0),
            BaseHeight(20.0),
        ));

        let mut query = app
            .world_mut()
            .query_filtered::<BreakerSizeData, With<Breaker>>();
        let data = query.single(app.world()).unwrap();
        assert!(data.size_boosts.is_none());
        assert!(data.node_scale.is_none());
    }

    // ── Part C: BreakerMovementData (mutable) ───────────────────────

    // Behavior 4: BreakerMovementData position mutation
    #[test]
    fn breaker_movement_data_position_mutation_takes_effect() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                Position2D(Vec2::new(100.0, -200.0)),
                Velocity2D(Vec2::new(300.0, 0.0)),
                DashState::Idle,
                MaxSpeed(600.0),
                BreakerAcceleration(2000.0),
                BreakerDeceleration(1500.0),
                DecelEasing {
                    ease: EaseFunction::QuadraticIn,
                    strength: 1.0,
                },
                BaseWidth(120.0),
            ))
            .id();

        app.add_systems(
            FixedUpdate,
            |mut query: Query<BreakerMovementData, With<Breaker>>| {
                for mut data in &mut query {
                    data.position.0.x += 10.0;
                }
            },
        );
        tick(&mut app);

        let pos = app.world().get::<Position2D>(entity).unwrap();
        assert_eq!(pos.0, Vec2::new(110.0, -200.0));
    }

    // Behavior 4: read-only config fields accessible
    #[test]
    fn breaker_movement_data_readonly_fields_accessible() {
        let mut app = test_app();
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(100.0, -200.0)),
            Velocity2D(Vec2::new(300.0, 0.0)),
            DashState::Idle,
            MaxSpeed(600.0),
            BreakerAcceleration(2000.0),
            BreakerDeceleration(1500.0),
            DecelEasing {
                ease: EaseFunction::QuadraticIn,
                strength: 1.0,
            },
            BaseWidth(120.0),
        ));

        app.add_systems(
            FixedUpdate,
            |query: Query<BreakerMovementDataReadOnly, With<Breaker>>,
             mut matched: ResMut<QueryMatched>| {
                for data in &query {
                    matched.0 = true;
                    assert_eq!(*data.state, DashState::Idle);
                    assert!((data.max_speed.0 - 600.0).abs() < f32::EPSILON);
                    assert!((data.acceleration.0 - 2000.0).abs() < f32::EPSILON);
                    assert!((data.deceleration.0 - 1500.0).abs() < f32::EPSILON);
                    assert!((data.decel_easing.strength - 1.0).abs() < f32::EPSILON);
                    assert!((data.base_width.0 - 120.0).abs() < f32::EPSILON);
                }
            },
        );
        tick(&mut app);
        assert_query_matched(&app);
    }

    // Behavior 4 edge case: optional speed/size boosts
    #[test]
    fn breaker_movement_data_optional_boosts_present() {
        let mut app = test_app();
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(100.0, -200.0)),
            Velocity2D(Vec2::new(300.0, 0.0)),
            DashState::Idle,
            MaxSpeed(600.0),
            BreakerAcceleration(2000.0),
            BreakerDeceleration(1500.0),
            DecelEasing {
                ease: EaseFunction::QuadraticIn,
                strength: 1.0,
            },
            BaseWidth(120.0),
            ActiveSpeedBoosts(vec![1.5]),
            ActiveSizeBoosts(vec![2.0]),
        ));

        app.add_systems(
            FixedUpdate,
            |query: Query<BreakerMovementDataReadOnly, With<Breaker>>,
             mut matched: ResMut<QueryMatched>| {
                for data in &query {
                    matched.0 = true;
                    assert!(
                        data.speed_boosts.is_some(),
                        "ActiveSpeedBoosts should be Some"
                    );
                    assert!(
                        data.size_boosts.is_some(),
                        "ActiveSizeBoosts should be Some"
                    );
                }
            },
        );
        tick(&mut app);
        assert_query_matched(&app);
    }

    // Behavior 5: velocity mutation takes effect
    #[test]
    fn breaker_movement_data_velocity_mutation_takes_effect() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                Position2D(Vec2::new(100.0, -200.0)),
                Velocity2D(Vec2::new(300.0, 0.0)),
                DashState::Idle,
                MaxSpeed(600.0),
                BreakerAcceleration(2000.0),
                BreakerDeceleration(1500.0),
                DecelEasing {
                    ease: EaseFunction::QuadraticIn,
                    strength: 1.0,
                },
                BaseWidth(120.0),
            ))
            .id();

        app.add_systems(
            FixedUpdate,
            |mut query: Query<BreakerMovementData, With<Breaker>>| {
                for mut data in &mut query {
                    data.velocity.0 = Vec2::new(500.0, 0.0);
                }
            },
        );
        tick(&mut app);

        let vel = app.world().get::<Velocity2D>(entity).unwrap();
        assert_eq!(vel.0, Vec2::new(500.0, 0.0));
    }

    // Behavior 5 edge case: both position and velocity mutation in same invocation
    #[test]
    fn breaker_movement_data_both_position_and_velocity_mutable() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                Position2D(Vec2::new(100.0, -200.0)),
                Velocity2D(Vec2::new(300.0, 0.0)),
                DashState::Idle,
                MaxSpeed(600.0),
                BreakerAcceleration(2000.0),
                BreakerDeceleration(1500.0),
                DecelEasing {
                    ease: EaseFunction::QuadraticIn,
                    strength: 1.0,
                },
                BaseWidth(120.0),
            ))
            .id();

        app.add_systems(
            FixedUpdate,
            |mut query: Query<BreakerMovementData, With<Breaker>>| {
                for mut data in &mut query {
                    data.position.0.x += 50.0;
                    data.velocity.0 = Vec2::new(500.0, 0.0);
                }
            },
        );
        tick(&mut app);

        let pos = app.world().get::<Position2D>(entity).unwrap();
        assert_eq!(pos.0, Vec2::new(150.0, -200.0));
        let vel = app.world().get::<Velocity2D>(entity).unwrap();
        assert_eq!(vel.0, Vec2::new(500.0, 0.0));
    }

    // ── Part D: BreakerDashData (mutable) ───────────────────────────

    // Behavior 6: BreakerDashData mutable state, tilt, timer access
    #[test]
    fn breaker_dash_data_state_tilt_timer_mutation() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                (
                    Breaker,
                    DashState::Dashing,
                    Velocity2D(Vec2::new(500.0, 0.0)),
                    BreakerTilt {
                        angle: 10.0,
                        ease_start: 0.0,
                        ease_target: 10.0,
                    },
                    DashStateTimer { remaining: 0.05 },
                    MaxSpeed(600.0),
                    BreakerDeceleration(1500.0),
                    DecelEasing {
                        ease: EaseFunction::QuadraticIn,
                        strength: 1.0,
                    },
                ),
                (
                    DashSpeedMultiplier(2.0),
                    DashDuration(0.15),
                    DashTilt(15.0),
                    DashTiltEase(EaseFunction::CubicIn),
                    BrakeTilt {
                        angle: -5.0,
                        duration: 0.1,
                        ease: EaseFunction::CubicIn,
                    },
                    BrakeDecel(3000.0),
                    SettleDuration(0.1),
                    SettleTiltEase(EaseFunction::CubicOut),
                ),
            ))
            .id();

        app.add_systems(
            FixedUpdate,
            |mut query: Query<BreakerDashData, With<Breaker>>| {
                for mut data in &mut query {
                    *data.state = DashState::Braking;
                    data.tilt.angle = -5.0;
                    data.timer.remaining = 0.0;
                }
            },
        );
        tick(&mut app);

        let state = app.world().get::<DashState>(entity).unwrap();
        assert_eq!(*state, DashState::Braking);
        let tilt = app.world().get::<BreakerTilt>(entity).unwrap();
        assert!((tilt.angle - (-5.0)).abs() < f32::EPSILON);
        let timer = app.world().get::<DashStateTimer>(entity).unwrap();
        assert!((timer.remaining - 0.0).abs() < f32::EPSILON);
    }

    // Behavior 6: read-only config fields
    #[test]
    fn breaker_dash_data_readonly_config_fields() {
        let mut app = test_app();
        app.world_mut().spawn((
            (
                Breaker,
                DashState::Dashing,
                Velocity2D(Vec2::new(500.0, 0.0)),
                BreakerTilt {
                    angle: 10.0,
                    ease_start: 0.0,
                    ease_target: 10.0,
                },
                DashStateTimer { remaining: 0.05 },
                MaxSpeed(600.0),
                BreakerDeceleration(1500.0),
                DecelEasing {
                    ease: EaseFunction::QuadraticIn,
                    strength: 1.0,
                },
            ),
            (
                DashSpeedMultiplier(2.0),
                DashDuration(0.15),
                DashTilt(15.0),
                DashTiltEase(EaseFunction::CubicIn),
                BrakeTilt {
                    angle: -5.0,
                    duration: 0.1,
                    ease: EaseFunction::CubicIn,
                },
                BrakeDecel(3000.0),
                SettleDuration(0.1),
                SettleTiltEase(EaseFunction::CubicOut),
            ),
        ));

        app.add_systems(
            FixedUpdate,
            |query: Query<BreakerDashDataReadOnly, With<Breaker>>,
             mut matched: ResMut<QueryMatched>| {
                for data in &query {
                    matched.0 = true;
                    assert!((data.max_speed.0 - 600.0).abs() < f32::EPSILON);
                    assert!((data.dash_speed.0 - 2.0).abs() < f32::EPSILON);
                    assert!((data.dash_duration.0 - 0.15).abs() < f32::EPSILON);
                    assert!((data.dash_tilt.0 - 15.0).abs() < f32::EPSILON);
                    assert!((data.brake_tilt.angle - (-5.0)).abs() < f32::EPSILON);
                    assert!((data.brake_decel.0 - 3000.0).abs() < f32::EPSILON);
                    assert!((data.settle_duration.0 - 0.1).abs() < f32::EPSILON);
                }
            },
        );
        tick(&mut app);
        assert_query_matched(&app);
    }

    // Behavior 6 edge case: FlashStepActive present
    #[test]
    fn breaker_dash_data_flash_step_optional_present() {
        let mut app = test_app();
        app.world_mut().spawn((
            (
                Breaker,
                DashState::Dashing,
                Velocity2D(Vec2::new(500.0, 0.0)),
                BreakerTilt::default(),
                DashStateTimer { remaining: 0.05 },
                MaxSpeed(600.0),
                BreakerDeceleration(1500.0),
                DecelEasing {
                    ease: EaseFunction::QuadraticIn,
                    strength: 1.0,
                },
            ),
            (
                DashSpeedMultiplier(2.0),
                DashDuration(0.15),
                DashTilt(15.0),
                DashTiltEase(EaseFunction::CubicIn),
                BrakeTilt {
                    angle: -5.0,
                    duration: 0.1,
                    ease: EaseFunction::CubicIn,
                },
                BrakeDecel(3000.0),
                SettleDuration(0.1),
                SettleTiltEase(EaseFunction::CubicOut),
                FlashStepActive,
            ),
        ));

        app.add_systems(
            FixedUpdate,
            |query: Query<BreakerDashDataReadOnly, With<Breaker>>,
             mut matched: ResMut<QueryMatched>| {
                for data in &query {
                    matched.0 = true;
                    assert!(data.flash_step.is_some(), "FlashStepActive should be Some");
                }
            },
        );
        tick(&mut app);
        assert_query_matched(&app);
    }

    // Behavior 7: BreakerDashData mutable position for flash step teleport
    #[test]
    fn breaker_dash_data_position_mutation_for_flash_step() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                (
                    Breaker,
                    DashState::Dashing,
                    Velocity2D(Vec2::new(500.0, 0.0)),
                    BreakerTilt::default(),
                    DashStateTimer { remaining: 0.05 },
                    MaxSpeed(600.0),
                    BreakerDeceleration(1500.0),
                    DecelEasing {
                        ease: EaseFunction::QuadraticIn,
                        strength: 1.0,
                    },
                ),
                (
                    DashSpeedMultiplier(2.0),
                    DashDuration(0.15),
                    DashTilt(15.0),
                    DashTiltEase(EaseFunction::CubicIn),
                    BrakeTilt {
                        angle: -5.0,
                        duration: 0.1,
                        ease: EaseFunction::CubicIn,
                    },
                    BrakeDecel(3000.0),
                    SettleDuration(0.1),
                    SettleTiltEase(EaseFunction::CubicOut),
                    FlashStepActive,
                    Position2D(Vec2::new(0.0, -200.0)),
                ),
            ))
            .id();

        app.add_systems(
            FixedUpdate,
            |mut query: Query<BreakerDashData, With<Breaker>>| {
                for mut data in &mut query {
                    if let Some(ref mut pos) = data.position {
                        pos.0.x = 200.0;
                    }
                }
            },
        );
        tick(&mut app);

        let pos = app.world().get::<Position2D>(entity).unwrap();
        assert_eq!(pos.0, Vec2::new(200.0, -200.0));
    }

    // Behavior 7 edge case: position absent (None), no mutation possible
    #[test]
    fn breaker_dash_data_position_none_when_absent() {
        let mut app = test_app();
        app.world_mut().spawn((
            (
                Breaker,
                DashState::Dashing,
                Velocity2D(Vec2::new(500.0, 0.0)),
                BreakerTilt::default(),
                DashStateTimer { remaining: 0.05 },
                MaxSpeed(600.0),
                BreakerDeceleration(1500.0),
                DecelEasing {
                    ease: EaseFunction::QuadraticIn,
                    strength: 1.0,
                },
            ),
            (
                DashSpeedMultiplier(2.0),
                DashDuration(0.15),
                DashTilt(15.0),
                DashTiltEase(EaseFunction::CubicIn),
                BrakeTilt {
                    angle: -5.0,
                    duration: 0.1,
                    ease: EaseFunction::CubicIn,
                },
                BrakeDecel(3000.0),
                SettleDuration(0.1),
                SettleTiltEase(EaseFunction::CubicOut),
                // No Position2D spawned
            ),
        ));

        app.add_systems(
            FixedUpdate,
            |query: Query<BreakerDashDataReadOnly, With<Breaker>>,
             mut matched: ResMut<QueryMatched>| {
                for data in &query {
                    matched.0 = true;
                    // Breaker's #[require(Spatial2D)] auto-inserts Position2D,
                    // so it's always Some on a Breaker entity even when not explicitly spawned.
                    assert!(
                        data.position.is_some(),
                        "Position2D auto-inserted via Breaker #[require]"
                    );
                }
            },
        );
        tick(&mut app);
        assert_query_matched(&app);
    }

    // Behavior 8: velocity mutation through BreakerDashData
    #[test]
    fn breaker_dash_data_velocity_mutation() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                (
                    Breaker,
                    DashState::Dashing,
                    Velocity2D(Vec2::new(500.0, 0.0)),
                    BreakerTilt::default(),
                    DashStateTimer { remaining: 0.05 },
                    MaxSpeed(600.0),
                    BreakerDeceleration(1500.0),
                    DecelEasing {
                        ease: EaseFunction::QuadraticIn,
                        strength: 1.0,
                    },
                ),
                (
                    DashSpeedMultiplier(2.0),
                    DashDuration(0.15),
                    DashTilt(15.0),
                    DashTiltEase(EaseFunction::CubicIn),
                    BrakeTilt {
                        angle: -5.0,
                        duration: 0.1,
                        ease: EaseFunction::CubicIn,
                    },
                    BrakeDecel(3000.0),
                    SettleDuration(0.1),
                    SettleTiltEase(EaseFunction::CubicOut),
                ),
            ))
            .id();

        app.add_systems(
            FixedUpdate,
            |mut query: Query<BreakerDashData, With<Breaker>>| {
                for mut data in &mut query {
                    data.velocity.0 = Vec2::ZERO;
                }
            },
        );
        tick(&mut app);

        let vel = app.world().get::<Velocity2D>(entity).unwrap();
        assert_eq!(vel.0, Vec2::ZERO);
    }

    // ── Part E: BreakerBumpTimingData (mutable) ─────────────────────

    // Behavior 9: BreakerBumpTimingData all bump timing fields
    #[test]
    fn breaker_bump_timing_data_all_fields_accessible() {
        let mut app = test_app();
        app.world_mut().spawn((
            Breaker,
            BumpState::default(),
            BumpPerfectWindow(0.05),
            BumpEarlyWindow(0.15),
            BumpLateWindow(0.1),
            BumpPerfectCooldown(0.5),
            BumpWeakCooldown(0.2),
        ));

        app.add_systems(
            FixedUpdate,
            |query: Query<BreakerBumpTimingDataReadOnly, With<Breaker>>,
             mut matched: ResMut<QueryMatched>| {
                for data in &query {
                    matched.0 = true;
                    assert!((data.perfect_window.0 - 0.05).abs() < f32::EPSILON);
                    assert!((data.early_window.0 - 0.15).abs() < f32::EPSILON);
                    assert!((data.late_window.0 - 0.1).abs() < f32::EPSILON);
                    assert!((data.perfect_cooldown.0 - 0.5).abs() < f32::EPSILON);
                    assert!((data.weak_cooldown.0 - 0.2).abs() < f32::EPSILON);
                    assert!(data.anchor_planted.is_none());
                    assert!(data.anchor_active.is_none());
                }
            },
        );
        tick(&mut app);
        assert_query_matched(&app);
    }

    // Behavior 9 edge case: anchors present
    #[test]
    fn breaker_bump_timing_data_with_anchors_present() {
        let mut app = test_app();
        app.world_mut().spawn((
            Breaker,
            BumpState::default(),
            BumpPerfectWindow(0.05),
            BumpEarlyWindow(0.15),
            BumpLateWindow(0.1),
            BumpPerfectCooldown(0.5),
            BumpWeakCooldown(0.2),
            AnchorPlanted,
            AnchorActive {
                bump_force_multiplier: 1.5,
                perfect_window_multiplier: 2.0,
                plant_delay: 0.5,
            },
        ));

        app.add_systems(
            FixedUpdate,
            |query: Query<BreakerBumpTimingDataReadOnly, With<Breaker>>,
             mut matched: ResMut<QueryMatched>| {
                for data in &query {
                    matched.0 = true;
                    assert!(
                        data.anchor_planted.is_some(),
                        "AnchorPlanted should be Some"
                    );
                    assert!(data.anchor_active.is_some(), "AnchorActive should be Some");
                }
            },
        );
        tick(&mut app);
        assert_query_matched(&app);
    }

    // Behavior 10: BumpState mutation through BreakerBumpTimingData
    #[test]
    fn breaker_bump_timing_data_bump_state_mutation() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BumpState::default(),
                BumpPerfectWindow(0.05),
                BumpEarlyWindow(0.15),
                BumpLateWindow(0.1),
                BumpPerfectCooldown(0.5),
                BumpWeakCooldown(0.2),
            ))
            .id();

        app.add_systems(
            FixedUpdate,
            |mut query: Query<BreakerBumpTimingData, With<Breaker>>| {
                for mut data in &mut query {
                    data.bump.active = true;
                }
            },
        );
        tick(&mut app);

        let bump = app.world().get::<BumpState>(entity).unwrap();
        assert!(
            bump.active,
            "BumpState.active should be true after mutation"
        );
    }

    // ── Part F: BreakerBumpGradingData (mutable) ────────────────────

    // Behavior 11: BreakerBumpGradingData fields (no early_window)
    #[test]
    fn breaker_bump_grading_data_fields_accessible_no_early_window() {
        let mut app = test_app();
        app.world_mut().spawn((
            Breaker,
            BumpState::default(),
            BumpPerfectWindow(0.05),
            BumpLateWindow(0.1),
            BumpPerfectCooldown(0.5),
            BumpWeakCooldown(0.2),
        ));

        app.add_systems(
            FixedUpdate,
            |query: Query<BreakerBumpGradingDataReadOnly, With<Breaker>>,
             mut matched: ResMut<QueryMatched>| {
                for data in &query {
                    matched.0 = true;
                    assert!((data.perfect_window.0 - 0.05).abs() < f32::EPSILON);
                    assert!((data.late_window.0 - 0.1).abs() < f32::EPSILON);
                    assert!((data.perfect_cooldown.0 - 0.5).abs() < f32::EPSILON);
                    assert!((data.weak_cooldown.0 - 0.2).abs() < f32::EPSILON);
                    assert!(data.anchor_planted.is_none());
                    assert!(data.anchor_active.is_none());
                }
            },
        );
        tick(&mut app);
        assert_query_matched(&app);
    }

    // Behavior 12: BumpState mutation through BreakerBumpGradingData
    #[test]
    fn breaker_bump_grading_data_bump_state_mutation() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BumpState::default(),
                BumpPerfectWindow(0.05),
                BumpLateWindow(0.1),
                BumpPerfectCooldown(0.5),
                BumpWeakCooldown(0.2),
            ))
            .id();

        app.add_systems(
            FixedUpdate,
            |mut query: Query<BreakerBumpGradingData, With<Breaker>>| {
                for mut data in &mut query {
                    data.bump.cooldown = 0.5;
                }
            },
        );
        tick(&mut app);

        let bump = app.world().get::<BumpState>(entity).unwrap();
        assert!(
            (bump.cooldown - 0.5).abs() < f32::EPSILON,
            "BumpState.cooldown should be 0.5 after mutation"
        );
    }

    // Behavior 13: BreakerBumpGradingData with anchors present
    #[test]
    fn breaker_bump_grading_data_with_anchors() {
        let mut app = test_app();
        app.world_mut().spawn((
            Breaker,
            BumpState::default(),
            BumpPerfectWindow(0.05),
            BumpLateWindow(0.1),
            BumpPerfectCooldown(0.5),
            BumpWeakCooldown(0.2),
            AnchorPlanted,
            AnchorActive {
                bump_force_multiplier: 1.5,
                perfect_window_multiplier: 2.0,
                plant_delay: 0.5,
            },
        ));

        app.add_systems(
            FixedUpdate,
            |query: Query<BreakerBumpGradingDataReadOnly, With<Breaker>>,
             mut matched: ResMut<QueryMatched>| {
                for data in &query {
                    matched.0 = true;
                    assert!(
                        data.anchor_planted.is_some(),
                        "AnchorPlanted should be Some"
                    );
                    assert!(data.anchor_active.is_some(), "AnchorActive should be Some");
                }
            },
        );
        tick(&mut app);
        assert_query_matched(&app);
    }

    // ── Part G: BreakerResetData (mutable) ──────────────────────────

    // Behavior 14: BreakerResetData full mutable reset
    #[test]
    fn breaker_reset_data_full_mutable_reset() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                Position2D(Vec2::new(100.0, -180.0)),
                DashState::Dashing,
                Velocity2D(Vec2::new(300.0, 0.0)),
                BreakerTilt {
                    angle: 15.0,
                    ease_start: 0.0,
                    ease_target: 15.0,
                },
                DashStateTimer { remaining: 0.1 },
                BumpState {
                    active: true,
                    ..BumpState::default()
                },
                BreakerBaseY(-200.0),
                PreviousPosition(Vec2::new(90.0, -180.0)),
            ))
            .id();

        app.add_systems(
            FixedUpdate,
            |mut query: Query<BreakerResetData, With<Breaker>>| {
                for mut data in &mut query {
                    data.position.0 = Vec2::new(0.0, -200.0);
                    data.velocity.0 = Vec2::ZERO;
                    *data.state = DashState::Idle;
                    data.tilt.angle = 0.0;
                    data.timer.remaining = 0.0;
                    data.bump.active = false;
                }
            },
        );
        tick(&mut app);

        let pos = app.world().get::<Position2D>(entity).unwrap();
        assert_eq!(pos.0, Vec2::new(0.0, -200.0));
        let vel = app.world().get::<Velocity2D>(entity).unwrap();
        assert_eq!(vel.0, Vec2::ZERO);
        let state = app.world().get::<DashState>(entity).unwrap();
        assert_eq!(*state, DashState::Idle);
        let tilt = app.world().get::<BreakerTilt>(entity).unwrap();
        assert!((tilt.angle - 0.0).abs() < f32::EPSILON);
        let timer = app.world().get::<DashStateTimer>(entity).unwrap();
        assert!((timer.remaining - 0.0).abs() < f32::EPSILON);
        let bump = app.world().get::<BumpState>(entity).unwrap();
        assert!(!bump.active);
    }

    // Behavior 14: base_y is read-only
    #[test]
    fn breaker_reset_data_base_y_readable() {
        let mut app = test_app();
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(100.0, -180.0)),
            DashState::Dashing,
            Velocity2D(Vec2::new(300.0, 0.0)),
            BreakerTilt::default(),
            DashStateTimer { remaining: 0.1 },
            BumpState::default(),
            BreakerBaseY(-200.0),
        ));

        app.add_systems(
            FixedUpdate,
            |query: Query<BreakerResetDataReadOnly, With<Breaker>>,
             mut matched: ResMut<QueryMatched>| {
                for data in &query {
                    matched.0 = true;
                    assert!((data.base_y.0 - (-200.0)).abs() < f32::EPSILON);
                }
            },
        );
        tick(&mut app);
        assert_query_matched(&app);
    }

    // Behavior 14 edge case: prev_position absent
    #[test]
    fn breaker_reset_data_prev_position_none_when_absent() {
        let mut app = test_app();
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(100.0, -180.0)),
            DashState::Dashing,
            Velocity2D(Vec2::new(300.0, 0.0)),
            BreakerTilt::default(),
            DashStateTimer { remaining: 0.1 },
            BumpState::default(),
            BreakerBaseY(-200.0),
            // No PreviousPosition
        ));

        app.add_systems(
            FixedUpdate,
            |query: Query<BreakerResetDataReadOnly, With<Breaker>>,
             mut matched: ResMut<QueryMatched>| {
                for data in &query {
                    matched.0 = true;
                    // Breaker's #[require(Spatial2D)] auto-inserts PreviousPosition,
                    // so it's always Some on a Breaker entity even when not explicitly spawned.
                    assert!(
                        data.prev_position.is_some(),
                        "PreviousPosition auto-inserted via Breaker #[require]"
                    );
                }
            },
        );
        tick(&mut app);
        assert_query_matched(&app);
    }

    // Behavior 15: prev_position mutation works when present
    #[test]
    fn breaker_reset_data_prev_position_mutation() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                Position2D(Vec2::new(100.0, -180.0)),
                DashState::Dashing,
                Velocity2D(Vec2::new(300.0, 0.0)),
                BreakerTilt::default(),
                DashStateTimer { remaining: 0.1 },
                BumpState::default(),
                BreakerBaseY(-200.0),
                PreviousPosition(Vec2::new(90.0, -180.0)),
            ))
            .id();

        app.add_systems(
            FixedUpdate,
            |mut query: Query<BreakerResetData, With<Breaker>>| {
                for mut data in &mut query {
                    if let Some(ref mut prev) = data.prev_position {
                        **prev = PreviousPosition(Vec2::ZERO);
                    }
                }
            },
        );
        tick(&mut app);

        let prev = app.world().get::<PreviousPosition>(entity).unwrap();
        assert_eq!(prev.0, Vec2::ZERO);
    }

    // ── Part H: SyncBreakerScaleData (mutable) ──────────────────────

    // Behavior 16: SyncBreakerScaleData named field access
    #[test]
    fn sync_breaker_scale_data_named_field_access() {
        let mut app = test_app();
        app.world_mut().spawn((
            Breaker,
            BaseWidth(120.0),
            BaseHeight(20.0),
            Scale2D { x: 1.0, y: 1.0 },
            ActiveSizeBoosts(vec![1.5]),
            NodeScalingFactor(0.8),
            MinWidth(60.0),
            MaxWidth(200.0),
            MinHeight(10.0),
            MaxHeight(50.0),
        ));

        app.add_systems(
            FixedUpdate,
            |query: Query<SyncBreakerScaleDataReadOnly, With<Breaker>>,
             mut matched: ResMut<QueryMatched>| {
                for data in &query {
                    matched.0 = true;
                    assert!((data.base_width.0 - 120.0).abs() < f32::EPSILON);
                    assert!((data.base_height.0 - 20.0).abs() < f32::EPSILON);
                    assert!(data.size_boosts.is_some());
                    assert!((data.node_scale.unwrap().0 - 0.8).abs() < f32::EPSILON);
                    assert!((data.min_w.unwrap().0 - 60.0).abs() < f32::EPSILON);
                    assert!((data.max_w.unwrap().0 - 200.0).abs() < f32::EPSILON);
                    assert!((data.min_h.unwrap().0 - 10.0).abs() < f32::EPSILON);
                    assert!((data.max_h.unwrap().0 - 50.0).abs() < f32::EPSILON);
                }
            },
        );
        tick(&mut app);
        assert_query_matched(&app);
    }

    // Behavior 16 edge case: all optionals absent
    #[test]
    fn sync_breaker_scale_data_all_optionals_absent() {
        let mut app = test_app();
        app.world_mut().spawn((
            Breaker,
            BaseWidth(120.0),
            BaseHeight(20.0),
            Scale2D { x: 1.0, y: 1.0 },
        ));

        app.add_systems(
            FixedUpdate,
            |query: Query<SyncBreakerScaleDataReadOnly, With<Breaker>>,
             mut matched: ResMut<QueryMatched>| {
                for data in &query {
                    matched.0 = true;
                    assert!(data.size_boosts.is_none());
                    assert!(data.node_scale.is_none());
                    assert!(data.min_w.is_none());
                    assert!(data.max_w.is_none());
                    assert!(data.min_h.is_none());
                    assert!(data.max_h.is_none());
                }
            },
        );
        tick(&mut app);
        assert_query_matched(&app);
    }

    // Behavior 17: Scale2D mutation through SyncBreakerScaleData
    #[test]
    fn sync_breaker_scale_data_scale_mutation() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BaseWidth(120.0),
                BaseHeight(20.0),
                Scale2D { x: 1.0, y: 1.0 },
            ))
            .id();

        app.add_systems(
            FixedUpdate,
            |mut query: Query<SyncBreakerScaleData, With<Breaker>>| {
                for mut data in &mut query {
                    data.scale.x = 160.0;
                    data.scale.y = 26.666;
                }
            },
        );
        tick(&mut app);

        let scale = app.world().get::<Scale2D>(entity).unwrap();
        assert!((scale.x - 160.0).abs() < f32::EPSILON);
        assert!((scale.y - 26.666).abs() < 1e-3);
    }

    // ── Part I: BreakerTelemetryData (read-only, dev-only) ──────────

    // Behavior 18: BreakerTelemetryData compiles and queries under dev feature
    #[cfg(feature = "dev")]
    #[test]
    fn breaker_telemetry_data_query_under_dev_feature() {
        let mut app = test_app();
        app.world_mut().spawn((
            Breaker,
            DashState::Idle,
            BumpState::default(),
            BreakerTilt::default(),
            Velocity2D(Vec2::new(100.0, 0.0)),
            BumpPerfectWindow(0.05),
            BumpEarlyWindow(0.15),
            BumpLateWindow(0.1),
        ));

        let mut query = app
            .world_mut()
            .query_filtered::<BreakerTelemetryData, With<Breaker>>();
        let data = query.single(app.world()).unwrap();
        assert_eq!(*data.state, DashState::Idle);
        assert!(!data.bump.active);
        assert!((data.tilt.angle - 0.0).abs() < f32::EPSILON);
        assert_eq!(data.velocity.0, Vec2::new(100.0, 0.0));
        assert!((data.perfect_window.0 - 0.05).abs() < f32::EPSILON);
        assert!((data.early_window.0 - 0.15).abs() < f32::EPSILON);
        assert!((data.late_window.0 - 0.1).abs() < f32::EPSILON);
    }

    // ── Part J: Cross-struct consistency ─────────────────────────────

    // Behavior 19: All 9 QueryData structs importable and usable in queries
    #[test]
    fn all_querydata_structs_importable_and_queryable() {
        let mut app = test_app();

        // Read-only structs — queried directly (no ReadOnly variant)
        drop(
            app.world_mut()
                .query_filtered::<BreakerCollisionData, With<Breaker>>(),
        );
        drop(
            app.world_mut()
                .query_filtered::<BreakerSizeData, With<Breaker>>(),
        );

        // Mutable structs — ReadOnly variant also exists
        drop(
            app.world_mut()
                .query_filtered::<BreakerMovementDataReadOnly, With<Breaker>>(),
        );
        drop(
            app.world_mut()
                .query_filtered::<BreakerDashDataReadOnly, With<Breaker>>(),
        );
        drop(
            app.world_mut()
                .query_filtered::<BreakerResetDataReadOnly, With<Breaker>>(),
        );
        drop(
            app.world_mut()
                .query_filtered::<BreakerBumpTimingDataReadOnly, With<Breaker>>(),
        );
        drop(
            app.world_mut()
                .query_filtered::<BreakerBumpGradingDataReadOnly, With<Breaker>>(),
        );
        drop(
            app.world_mut()
                .query_filtered::<SyncBreakerScaleDataReadOnly, With<Breaker>>(),
        );
    }

    #[cfg(feature = "dev")]
    #[test]
    fn telemetry_data_importable_under_dev_feature() {
        let mut app = test_app();
        drop(
            app.world_mut()
                .query_filtered::<BreakerTelemetryData, With<Breaker>>(),
        );
    }
}
