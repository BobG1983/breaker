//! Invariant checker systems and `ViolationLog` resource.
//!
//! Invariant systems run in `FixedUpdate` after gameplay. They query game state
//! and append to a [`ViolationLog`] resource. They never panic — they collect
//! all violations for end-of-run reporting.

use bevy::{platform::collections::HashMap, prelude::*};
use breaker::{
    bolt::components::{BoltMaxSpeed, BoltMinSpeed, BoltVelocity},
    breaker::components::{BreakerState, BreakerWidth},
    run::node::resources::NodeTimer,
    shared::{GameState, PlayfieldConfig, PlayingState},
};

use crate::{lifecycle::ScenarioConfig, types::InvariantKind};

/// Statistics collected during a scenario run.
///
/// Inserted by [`ScenarioLifecycle`] at run start. Updated by various systems.
#[derive(Resource, Default, Clone)]
pub struct ScenarioStats {
    /// Total number of actions injected via [`inject_scenario_input`].
    pub actions_injected: u32,
    /// Total number of invariant check evaluations performed.
    pub invariant_checks: u32,
    /// Highest [`ScenarioFrame`] value reached.
    pub max_frame: u32,
    /// Whether [`GameState::Playing`] was entered at least once.
    pub entered_playing: bool,
    /// Number of bolt entities that were tagged with [`ScenarioTagBolt`].
    pub bolts_tagged: u32,
    /// Number of breaker entities that were tagged with [`ScenarioTagBreaker`].
    pub breakers_tagged: u32,
}

/// Checks that [`BreakerState`] transitions on the tagged breaker follow the legal path.
///
/// Legal transitions: `Idle → Dashing`, `Dashing → Braking`, `Braking → Settling`,
/// `Settling → Idle`. Any other change fires a [`ViolationEntry`] with
/// [`InvariantKind::ValidBreakerState`].
///
/// Skips the first frame per entity (no previous state stored yet for that entity).
pub fn check_valid_breaker_state(
    breakers: Query<(Entity, &BreakerState), With<ScenarioTagBreaker>>,
    mut previous: Local<HashMap<Entity, BreakerState>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    for (entity, &current) in &breakers {
        if let Some(&prev) = previous.get(&entity)
            && prev != current
        {
            let legal = matches!(
                (prev, current),
                (
                    BreakerState::Idle | BreakerState::Settling,
                    BreakerState::Dashing
                ) | (BreakerState::Dashing, BreakerState::Braking)
                    | (BreakerState::Braking, BreakerState::Settling)
                    | (BreakerState::Settling, BreakerState::Idle)
            );
            if !legal {
                log.0.push(ViolationEntry {
                    frame: frame.0,
                    invariant: InvariantKind::ValidBreakerState,
                    entity: None,
                    message: format!(
                        "ValidBreakerState FAIL frame={} {prev:?} → {current:?}",
                        frame.0,
                    ),
                });
            }
        }
        previous.insert(entity, current);
    }
}

/// Checks that [`NodeTimer::remaining`] never increases between ticks.
///
/// Stores the previous `remaining` value in a `Local`. If the current value is
/// greater than the previous (by more than floating-point noise), appends a
/// [`ViolationEntry`] with [`InvariantKind::TimerMonotonicallyDecreasing`].
///
/// Skips and resets when [`NodeTimer`] is absent.
pub fn check_timer_monotonically_decreasing(
    timer: Option<Res<NodeTimer>>,
    mut previous: Local<Option<f32>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let Some(timer) = timer else {
        *previous = None;
        return;
    };
    let current = timer.remaining;
    // When prev is 0.0 (NodeTimer default before init), an increase to the real
    // timer value is initialization, not a violation.
    if let Some(prev) = *previous
        && prev > 0.0
        && current > prev
    {
        log.0.push(ViolationEntry {
            frame: frame.0,
            invariant: InvariantKind::TimerMonotonicallyDecreasing,
            entity: None,
            message: format!(
                "TimerMonotonicallyDecreasing FAIL frame={} remaining increased {prev:.3} → {current:.3}",
                frame.0,
            ),
        });
    }
    *previous = Some(current);
}

/// Checks that the tagged breaker's x position stays within `playfield.right() - half_width`.
///
/// Appends a [`ViolationEntry`] with [`InvariantKind::BreakerPositionClamped`] when the
/// breaker is outside the tight clamping bounds (with 1px tolerance).
pub fn check_breaker_position_clamped(
    breakers: Query<(Entity, &Transform, &BreakerWidth), With<ScenarioTagBreaker>>,
    playfield: Res<PlayfieldConfig>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let tolerance = 1.0_f32;
    for (entity, transform, width) in &breakers {
        let half_width = width.half_width();
        let max_x = playfield.right() - half_width;
        let min_x = playfield.left() + half_width;
        let x = transform.translation.x;
        if x > max_x + tolerance || x < min_x - tolerance {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::BreakerPositionClamped,
                entity: Some(entity),
                message: format!(
                    "BreakerPositionClamped FAIL frame={} entity={entity:?} x={x:.1} bounds=[{min_x:.1}, {max_x:.1}]",
                    frame.0,
                ),
            });
        }
    }
}

/// Per-entity previous position map for pause-freeze checking.
///
/// Stored in a [`Local`] to track bolt positions between fixed-update ticks.
type PreviousBoltPositions = HashMap<Entity, Vec3>;

/// Checks that physics entities do not move while the game is paused.
///
/// Stores the previous `Transform` for each tagged bolt each tick. When
/// [`PlayingState`] is [`PlayingState::Paused`] and a bolt has moved since
/// last tick, appends a [`ViolationEntry`] with
/// [`InvariantKind::PhysicsFrozenDuringPause`].
///
/// Clears local state when [`PlayingState`] is absent (game is not in `Playing`).
pub fn check_physics_frozen_during_pause(
    bolts: Query<(Entity, &Transform), With<ScenarioTagBolt>>,
    playing_state: Option<Res<State<PlayingState>>>,
    mut previous_positions: Local<PreviousBoltPositions>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let Some(state) = playing_state else {
        previous_positions.clear();
        return;
    };

    let is_paused = **state == PlayingState::Paused;

    for (entity, transform) in &bolts {
        let current_pos = transform.translation;
        if is_paused
            && let Some(&prev_pos) = previous_positions.get(&entity)
            && current_pos != prev_pos
        {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::PhysicsFrozenDuringPause,
                entity: Some(entity),
                message: format!(
                    "PhysicsFrozenDuringPause FAIL frame={} entity={entity:?} moved from {prev_pos:?} to {current_pos:?}",
                    frame.0,
                ),
            });
        }
        previous_positions.insert(entity, current_pos);
    }
}

/// Query filter that matches entities tagged for invariant checking.
type TaggedTransformQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Transform),
    Or<(With<ScenarioTagBolt>, With<ScenarioTagBreaker>)>,
>;

/// Marker — attached by the lifecycle plugin to the bolt entity for invariant checking.
#[derive(Component)]
pub struct ScenarioTagBolt;

/// Marker — attached by the lifecycle plugin to the breaker entity for invariant checking.
#[derive(Component)]
pub struct ScenarioTagBreaker;

/// Tracks the current fixed-update frame number for violation logging.
#[derive(Resource, Default)]
pub struct ScenarioFrame(pub u32);

/// A single invariant violation recorded during a scenario run.
#[derive(Debug, Clone)]
pub struct ViolationEntry {
    /// Fixed-update frame when the violation was detected.
    pub frame: u32,
    /// Which invariant was violated.
    pub invariant: InvariantKind,
    /// Entity involved, if applicable.
    pub entity: Option<Entity>,
    /// Human-readable description with concrete values.
    pub message: String,
}

/// Accumulated violations from all invariant checks.
#[derive(Resource, Default)]
pub struct ViolationLog(pub Vec<ViolationEntry>);

/// Stores the frozen world-space position for an entity with `disable_physics: true`.
///
/// When `ScenarioPhysicsFrozen` is present on an entity, `enforce_frozen_positions`
/// resets the entity's `Transform` to `target` every tick, preventing physics from
/// moving it.
#[derive(Component)]
pub struct ScenarioPhysicsFrozen {
    /// The world-space position this entity is pinned to each tick.
    pub target: Vec3,
}

/// Checks that all [`ScenarioTagBolt`] entities remain within playfield bounds.
///
/// Appends a [`ViolationEntry`] to [`ViolationLog`] for every bolt whose
/// `Transform` translation is outside any of the four playfield boundaries
/// (`bottom`, `top`, `left`, `right`). All checks use strict inequality.
///
/// Increments [`ScenarioStats::invariant_checks`] by the number of bolts checked.
pub fn check_bolt_in_bounds(
    bolts: Query<(Entity, &Transform), With<ScenarioTagBolt>>,
    playfield: Res<PlayfieldConfig>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    let bottom = playfield.bottom();
    let top = playfield.top();
    let left = playfield.left();
    let right = playfield.right();
    let mut checks = 0u32;
    for (entity, transform) in &bolts {
        checks += 1;
        let x = transform.translation.x;
        let y = transform.translation.y;
        if y < bottom {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::BoltInBounds,
                entity: Some(entity),
                message: format!(
                    "BoltInBounds FAIL frame={} entity={entity:?} position=(_, {y:.1}) bottom_bound={bottom:.1}",
                    frame.0,
                ),
            });
        }
        if y > top {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::BoltInBounds,
                entity: Some(entity),
                message: format!(
                    "BoltInBounds FAIL frame={} entity={entity:?} position=(_, {y:.1}) top_bound={top:.1}",
                    frame.0,
                ),
            });
        }
        if x < left {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::BoltInBounds,
                entity: Some(entity),
                message: format!(
                    "BoltInBounds FAIL frame={} entity={entity:?} position=({x:.1}, _) left_bound={left:.1}",
                    frame.0,
                ),
            });
        }
        if x > right {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::BoltInBounds,
                entity: Some(entity),
                message: format!(
                    "BoltInBounds FAIL frame={} entity={entity:?} position=({x:.1}, _) right_bound={right:.1}",
                    frame.0,
                ),
            });
        }
    }
    if let Some(ref mut s) = stats {
        s.invariant_checks += checks;
    }
}

/// Checks that all tagged entities have finite `Transform` values (no NaN or Inf).
///
/// Appends a [`ViolationEntry`] to [`ViolationLog`] for every entity whose
/// translation or rotation contains a non-finite value.
pub fn check_no_nan(
    tagged: TaggedTransformQuery,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    for (entity, transform) in &tagged {
        let t = transform.translation;
        let r = transform.rotation;
        if !t.is_finite() || !r.is_finite() {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::NoNaN,
                entity: Some(entity),
                message: format!(
                    "NoNaN FAIL frame={} entity={entity:?} translation={t:?} rotation={r:?}",
                    frame.0,
                ),
            });
        }
    }
}

/// Checks that all [`ScenarioTagBreaker`] entities remain within playfield bounds.
///
/// Appends a [`ViolationEntry`] for every breaker whose `Transform` translation x
/// is outside `PlayfieldConfig::left()` or `PlayfieldConfig::right()` (with 50.0 margin).
pub fn check_breaker_in_bounds(
    breakers: Query<(Entity, &Transform), With<ScenarioTagBreaker>>,
    playfield: Res<PlayfieldConfig>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let margin = 50.0;
    let left = playfield.left() - margin;
    let right = playfield.right() + margin;
    for (entity, transform) in &breakers {
        let x = transform.translation.x;
        if x < left || x > right {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::BreakerInBounds,
                entity: Some(entity),
                message: format!(
                    "BreakerInBounds FAIL frame={} entity={entity:?} x={x:.1} bounds=[{left:.1}, {right:.1}]",
                    frame.0,
                ),
            });
        }
    }
}

/// Tracks the previous [`GameState`] for transition validation.
#[derive(Resource, Default)]
pub struct PreviousGameState(pub Option<GameState>);

/// Checks that [`GameState`] transitions follow valid paths.
///
/// Forbidden transitions:
/// - `Loading → Playing` (must go through `MainMenu`)
/// - `Loading → RunEnd`
/// - `Playing → Loading`
/// - `RunEnd → Playing` (must go through `MainMenu`)
pub fn check_valid_state_transitions(
    state: Res<State<GameState>>,
    mut previous: ResMut<PreviousGameState>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let current = **state;
    if let Some(prev) = previous.0
        && prev != current
    {
        let forbidden = matches!(
            (prev, current),
            (GameState::Loading | GameState::RunEnd, GameState::Playing)
                | (GameState::Loading, GameState::RunEnd)
                | (GameState::Playing, GameState::Loading)
        );
        if forbidden {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::ValidStateTransitions,
                entity: None,
                message: format!(
                    "ValidStateTransitions FAIL frame={} {prev:?} → {current:?}",
                    frame.0,
                ),
            });
        }
    }
    previous.0 = Some(current);
}

/// Baseline entity count for leak detection.
#[derive(Resource, Default)]
pub struct EntityLeakBaseline {
    /// Entity count sampled at frame 60.
    pub baseline: Option<usize>,
}

/// Checks for unexpected entity accumulation over time.
///
/// Samples entity count at frame 60 as baseline. Every 120 frames after that,
/// checks if count exceeds 2× baseline.
pub fn check_no_entity_leaks(
    all_entities: Query<Entity>,
    frame: Res<ScenarioFrame>,
    mut baseline: ResMut<EntityLeakBaseline>,
    mut log: ResMut<ViolationLog>,
) {
    let count = all_entities.iter().count();

    if frame.0 == 60 {
        baseline.baseline = Some(count);
        return;
    }

    let Some(base) = baseline.baseline else {
        return;
    };

    if frame.0 > 60 && frame.0.is_multiple_of(120) && count > base * 2 {
        log.0.push(ViolationEntry {
            frame: frame.0,
            invariant: InvariantKind::NoEntityLeaks,
            entity: None,
            message: format!(
                "NoEntityLeaks FAIL frame={} count={count} baseline={base} (>{} threshold)",
                frame.0,
                base * 2,
            ),
        });
    }
}

/// Checks that bolt speed stays within configured min/max bounds.
///
/// Skips bolts with zero speed (serving or dead bolts).
pub fn check_bolt_speed_in_range(
    bolts: Query<(Entity, &BoltVelocity, &BoltMinSpeed, &BoltMaxSpeed), With<ScenarioTagBolt>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    for (entity, velocity, min_speed, max_speed) in &bolts {
        let speed = velocity.speed();
        if speed < f32::EPSILON {
            continue;
        }
        if speed < min_speed.0 || speed > max_speed.0 {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::BoltSpeedInRange,
                entity: Some(entity),
                message: format!(
                    "BoltSpeedInRange FAIL frame={} entity={entity:?} speed={speed:.1} bounds=[{:.1}, {:.1}]",
                    frame.0, min_speed.0, max_speed.0,
                ),
            });
        }
    }
}

/// Checks that [`NodeTimer::remaining`] never goes negative.
///
/// Only runs when the `NodeTimer` resource exists (Chrono archetype).
pub fn check_timer_non_negative(
    timer: Option<Res<NodeTimer>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let Some(timer) = timer else { return };
    if timer.remaining < 0.0 {
        log.0.push(ViolationEntry {
            frame: frame.0,
            invariant: InvariantKind::TimerNonNegative,
            entity: None,
            message: format!(
                "TimerNonNegative FAIL frame={} remaining={:.3}",
                frame.0, timer.remaining,
            ),
        });
    }
}

/// Checks that the bolt count stays within `invariant_params.max_bolt_count`.
///
/// Catches bolt accumulation leaks (e.g. Prism bolts not despawned on loss).
pub fn check_bolt_count_reasonable(
    bolts: Query<Entity, With<ScenarioTagBolt>>,
    config: Res<ScenarioConfig>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let max = config.definition.invariant_params.max_bolt_count;
    let count = bolts.iter().count();
    if count > max {
        log.0.push(ViolationEntry {
            frame: frame.0,
            invariant: InvariantKind::BoltCountReasonable,
            entity: None,
            message: format!("BoltCountReasonable FAIL frame={} count={count}", frame.0),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    /// Build a minimal test app with `check_bolt_in_bounds` registered plus
    /// required resources pre-inserted.
    fn test_app_bolt_in_bounds() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(PlayfieldConfig::default());
        app.insert_resource(ViolationLog::default());
        app.insert_resource(ScenarioFrame::default());
        app.add_systems(FixedUpdate, check_bolt_in_bounds);
        app
    }

    /// Build a minimal test app with `check_no_nan` registered plus
    /// required resources pre-inserted.
    fn test_app_no_nan() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ViolationLog::default());
        app.insert_resource(ScenarioFrame::default());
        app.add_systems(FixedUpdate, check_no_nan);
        app
    }

    /// Advance one fixed-update timestep and run one update.
    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // -------------------------------------------------------------------------
    // BoltInBounds — violation fires when bolt is below bottom bound
    // -------------------------------------------------------------------------

    /// A bolt at y = -500.0 is below the bottom bound of a playfield with
    /// height 700.0 (bottom = -350.0). The system must append one
    /// [`ViolationEntry`] with [`InvariantKind::BoltInBounds`], frame 1842,
    /// the entity id, and a message containing the actual position and the bound.
    #[test]
    fn bolt_in_bounds_appends_violation_when_bolt_is_below_bottom_bound() {
        let mut app = test_app_bolt_in_bounds();

        // height 700.0 → bottom() = -350.0
        app.world_mut().insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
        });
        app.world_mut().insert_resource(ScenarioFrame(1842));

        let bolt_entity = app
            .world_mut()
            .spawn((
                ScenarioTagBolt,
                Transform::from_translation(Vec3::new(0.0, -500.0, 0.0)),
            ))
            .id();

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly one violation, got {}",
            log.0.len()
        );

        let entry = &log.0[0];
        assert_eq!(entry.invariant, InvariantKind::BoltInBounds);
        assert_eq!(entry.frame, 1842);
        assert_eq!(entry.entity, Some(bolt_entity));
        assert!(
            entry.message.contains("1842"),
            "message should contain frame '1842', got: {}",
            entry.message
        );
        assert!(
            entry.message.contains("-500"),
            "message should contain bolt y '-500', got: {}",
            entry.message
        );
        assert!(
            entry.message.contains("-350"),
            "message should contain bound '-350', got: {}",
            entry.message
        );
    }

    // -------------------------------------------------------------------------
    // BoltInBounds — no violation when bolt is within bounds
    // -------------------------------------------------------------------------

    /// A bolt at y = -100.0 is within a playfield with height 700.0
    /// (bottom = -350.0). No violations should be recorded.
    #[test]
    fn bolt_in_bounds_does_not_fire_when_bolt_is_within_bounds() {
        let mut app = test_app_bolt_in_bounds();

        app.world_mut().insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
        });
        app.world_mut().insert_resource(ScenarioFrame(10));

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(0.0, -100.0, 0.0)),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations for in-bounds bolt at y = -100.0, got: {:?}",
            log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
    }

    // -------------------------------------------------------------------------
    // BoltInBounds — edge case: bolt exactly at the bottom bound
    // -------------------------------------------------------------------------

    /// A bolt exactly at y = -350.0 (the bottom boundary of a 700.0-height
    /// playfield) should NOT trigger a violation — it is at the edge, not below.
    #[test]
    fn bolt_in_bounds_does_not_fire_when_bolt_is_exactly_at_bottom_bound() {
        let mut app = test_app_bolt_in_bounds();

        app.world_mut().insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
        });
        app.world_mut().insert_resource(ScenarioFrame(0));

        // `PlayfieldConfig::bottom()` returns -350.0 for height 700.0
        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(0.0, -350.0, 0.0)),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when bolt is exactly at bottom bound (-350.0)"
        );
    }

    // -------------------------------------------------------------------------
    // ViolationEntry — fields populated correctly
    // -------------------------------------------------------------------------

    /// Verifies that the entry from the out-of-bounds case has all required
    /// fields set: frame, invariant, entity (`Some`), and a message with values.
    #[test]
    fn violation_entry_contains_frame_invariant_entity_and_message_with_values() {
        let mut app = test_app_bolt_in_bounds();

        app.world_mut().insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
        });
        app.world_mut().insert_resource(ScenarioFrame(1842));

        let bolt_entity = app
            .world_mut()
            .spawn((
                ScenarioTagBolt,
                Transform::from_translation(Vec3::new(0.0, -500.0, 0.0)),
            ))
            .id();

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        let entry = &log.0[0];

        assert_eq!(entry.frame, 1842, "frame must be 1842");
        assert_eq!(
            entry.invariant,
            InvariantKind::BoltInBounds,
            "invariant must be BoltInBounds"
        );
        assert_eq!(
            entry.entity,
            Some(bolt_entity),
            "entity must be Some(bolt_entity)"
        );
        assert!(!entry.message.is_empty(), "message must not be empty");
        assert!(
            entry.message.contains("-500"),
            "message must contain the bolt y position '-500', got: {}",
            entry.message
        );
        assert!(
            entry.message.contains("-350"),
            "message must contain the bound value '-350', got: {}",
            entry.message
        );
    }

    // -------------------------------------------------------------------------
    // NoNaN — violation fires when Transform translation has NaN
    // -------------------------------------------------------------------------

    /// A bolt entity with `f32::NAN` in the x component of translation triggers a
    /// [`ViolationEntry`] with [`InvariantKind::NoNaN`], frame 5, and a message
    /// containing "NaN".
    #[test]
    fn no_nan_appends_violation_when_transform_translation_has_nan() {
        let mut app = test_app_no_nan();

        app.world_mut().insert_resource(ScenarioFrame(5));

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(f32::NAN, 0.0, 0.0)),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly one NaN violation, got {}",
            log.0.len()
        );

        let entry = &log.0[0];
        assert_eq!(entry.invariant, InvariantKind::NoNaN);
        assert_eq!(entry.frame, 5);
        assert!(
            entry.message.contains("NaN"),
            "message must contain 'NaN', got: {}",
            entry.message
        );
    }

    // -------------------------------------------------------------------------
    // NoNaN — no violation for finite transforms
    // -------------------------------------------------------------------------

    /// A bolt at (1.0, 2.0, 0.0) with `Quat::IDENTITY` rotation is fully finite.
    /// No violations should be recorded.
    #[test]
    fn no_nan_does_not_fire_for_finite_transforms() {
        let mut app = test_app_no_nan();

        app.world_mut().insert_resource(ScenarioFrame(0));

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform {
                translation: Vec3::new(1.0, 2.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations for finite transform, got: {:?}",
            log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
    }

    // -------------------------------------------------------------------------
    // NoNaN — ScenarioTagBreaker entities are also checked
    // -------------------------------------------------------------------------

    /// The `check_no_nan` system covers both [`ScenarioTagBolt`] and
    /// [`ScenarioTagBreaker`] entities. A breaker with `f32::NAN` in its
    /// y translation should also trigger a violation.
    #[test]
    fn no_nan_fires_for_breaker_tagged_entity_with_nan_translation() {
        let mut app = test_app_no_nan();

        app.world_mut().insert_resource(ScenarioFrame(99));

        app.world_mut().spawn((
            ScenarioTagBreaker,
            Transform::from_translation(Vec3::new(0.0, f32::NAN, 0.0)),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            !log.0.is_empty(),
            "expected a NaN violation for ScenarioTagBreaker entity"
        );
        assert_eq!(log.0[0].invariant, InvariantKind::NoNaN);
        assert!(
            log.0[0].message.contains("NaN"),
            "message must contain 'NaN', got: {}",
            log.0[0].message
        );
    }

    // -------------------------------------------------------------------------
    // NoNaN — NaN in rotation triggers violation
    // -------------------------------------------------------------------------

    /// A transform with a NaN quaternion component should also trigger
    /// [`InvariantKind::NoNaN`]. This covers the rotation field, not just
    /// translation.
    #[test]
    fn no_nan_fires_when_rotation_contains_nan() {
        let mut app = test_app_no_nan();

        app.world_mut().insert_resource(ScenarioFrame(7));

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                rotation: Quat::from_xyzw(0.0, 0.0, 0.0, f32::NAN),
                scale: Vec3::ONE,
            },
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            !log.0.is_empty(),
            "expected NoNaN violation for transform with NaN rotation"
        );
        assert_eq!(log.0[0].invariant, InvariantKind::NoNaN);
    }

    // -------------------------------------------------------------------------
    // BreakerInBounds — violation when breaker is outside playfield
    // -------------------------------------------------------------------------

    fn test_app_breaker_in_bounds() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(PlayfieldConfig::default());
        app.insert_resource(ViolationLog::default());
        app.insert_resource(ScenarioFrame::default());
        app.add_systems(FixedUpdate, check_breaker_in_bounds);
        app
    }

    #[test]
    fn breaker_in_bounds_fires_when_breaker_far_outside_right() {
        let mut app = test_app_breaker_in_bounds();

        app.world_mut().spawn((
            ScenarioTagBreaker,
            Transform::from_translation(Vec3::new(1000.0, 0.0, 0.0)),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert_eq!(log.0[0].invariant, InvariantKind::BreakerInBounds);
    }

    #[test]
    fn breaker_in_bounds_does_not_fire_when_breaker_centered() {
        let mut app = test_app_breaker_in_bounds();

        app.world_mut().spawn((
            ScenarioTagBreaker,
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }

    // -------------------------------------------------------------------------
    // ValidStateTransitions — forbidden transition fires violation
    // -------------------------------------------------------------------------

    fn test_app_valid_transitions() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.insert_resource(ViolationLog::default());
        app.insert_resource(ScenarioFrame::default());
        app.init_resource::<PreviousGameState>();
        app.add_systems(FixedUpdate, check_valid_state_transitions);
        app
    }

    #[test]
    fn valid_state_transitions_fires_on_loading_to_playing() {
        let mut app = test_app_valid_transitions();
        // Set previous to Loading (the default initial state)
        app.world_mut()
            .insert_resource(PreviousGameState(Some(GameState::Loading)));
        // Transition to Playing (forbidden: skips MainMenu)
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update(); // process state transition
        tick(&mut app); // run checker in FixedUpdate

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0
                .iter()
                .any(|v| v.invariant == InvariantKind::ValidStateTransitions),
            "expected ValidStateTransitions violation for Loading→Playing"
        );
    }

    #[test]
    fn valid_state_transitions_does_not_fire_on_loading_to_main_menu() {
        let mut app = test_app_valid_transitions();
        app.world_mut()
            .insert_resource(PreviousGameState(Some(GameState::Loading)));
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::MainMenu);
        app.update();
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        let violations: Vec<_> = log
            .0
            .iter()
            .filter(|v| v.invariant == InvariantKind::ValidStateTransitions)
            .collect();
        assert!(
            violations.is_empty(),
            "Loading→MainMenu should be valid, got: {:?}",
            violations.iter().map(|v| &v.message).collect::<Vec<_>>()
        );
    }

    // -------------------------------------------------------------------------
    // NoEntityLeaks — violation when entity count explodes
    // -------------------------------------------------------------------------

    #[test]
    fn no_entity_leaks_fires_when_count_exceeds_double_baseline() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ViolationLog::default());
        app.insert_resource(ScenarioFrame(120));
        app.insert_resource(EntityLeakBaseline { baseline: Some(5) });
        app.add_systems(FixedUpdate, check_no_entity_leaks);

        // Spawn enough entities to exceed 2×5 = 10
        for _ in 0..15 {
            app.world_mut().spawn(Transform::default());
        }

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0
                .iter()
                .any(|v| v.invariant == InvariantKind::NoEntityLeaks),
            "expected NoEntityLeaks violation when count >> baseline"
        );
    }

    #[test]
    fn no_entity_leaks_does_not_fire_when_count_is_normal() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ViolationLog::default());
        app.insert_resource(ScenarioFrame(120));
        app.insert_resource(EntityLeakBaseline {
            baseline: Some(100),
        });
        app.add_systems(FixedUpdate, check_no_entity_leaks);

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no NoEntityLeaks violation when count <= baseline"
        );
    }

    // -------------------------------------------------------------------------
    // BoltSpeedInRange
    // -------------------------------------------------------------------------

    fn test_app_bolt_speed() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ViolationLog::default());
        app.insert_resource(ScenarioFrame::default());
        app.add_systems(FixedUpdate, check_bolt_speed_in_range);
        app
    }

    #[test]
    fn bolt_speed_in_range_fires_when_above_max() {
        let mut app = test_app_bolt_speed();

        app.world_mut().spawn((
            ScenarioTagBolt,
            BoltVelocity::new(0.0, 1000.0),
            BoltMinSpeed(200.0),
            BoltMaxSpeed(800.0),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert_eq!(log.0[0].invariant, InvariantKind::BoltSpeedInRange);
    }

    #[test]
    fn bolt_speed_in_range_does_not_fire_when_within_bounds() {
        let mut app = test_app_bolt_speed();

        app.world_mut().spawn((
            ScenarioTagBolt,
            BoltVelocity::new(0.0, 400.0),
            BoltMinSpeed(200.0),
            BoltMaxSpeed(800.0),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }

    #[test]
    fn bolt_speed_in_range_skips_zero_speed() {
        let mut app = test_app_bolt_speed();

        app.world_mut().spawn((
            ScenarioTagBolt,
            BoltVelocity::new(0.0, 0.0),
            BoltMinSpeed(200.0),
            BoltMaxSpeed(800.0),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty(), "zero speed should be skipped");
    }

    // -------------------------------------------------------------------------
    // TimerNonNegative
    // -------------------------------------------------------------------------

    #[test]
    fn timer_non_negative_fires_when_remaining_is_negative() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ViolationLog::default());
        app.insert_resource(ScenarioFrame::default());
        app.insert_resource(NodeTimer {
            remaining: -1.0,
            total: 60.0,
        });
        app.add_systems(FixedUpdate, check_timer_non_negative);

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert_eq!(log.0[0].invariant, InvariantKind::TimerNonNegative);
    }

    #[test]
    fn timer_non_negative_does_not_fire_when_remaining_is_zero() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ViolationLog::default());
        app.insert_resource(ScenarioFrame::default());
        app.insert_resource(NodeTimer {
            remaining: 0.0,
            total: 60.0,
        });
        app.add_systems(FixedUpdate, check_timer_non_negative);

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }

    #[test]
    fn timer_non_negative_skips_when_no_resource() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ViolationLog::default());
        app.insert_resource(ScenarioFrame::default());
        // NodeTimer not inserted
        app.add_systems(FixedUpdate, check_timer_non_negative);

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }

    // -------------------------------------------------------------------------
    // BoltCountReasonable
    // -------------------------------------------------------------------------

    fn bolt_count_test_app(max_bolt_count: usize) -> App {
        use crate::types::{InputStrategy, InvariantParams, ScenarioDefinition, ScriptedParams};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ViolationLog::default());
        app.insert_resource(ScenarioFrame::default());
        app.insert_resource(ScenarioConfig {
            definition: ScenarioDefinition {
                breaker: "Aegis".to_owned(),
                layout: "Corridor".to_owned(),
                input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
                max_frames: 1000,
                invariants: vec![],
                expected_violations: None,
                debug_setup: None,
                invariant_params: InvariantParams { max_bolt_count },
                allow_early_end: true,
            },
        });
        app.add_systems(FixedUpdate, check_bolt_count_reasonable);
        app
    }

    #[test]
    fn bolt_count_reasonable_fires_when_count_exceeds_max() {
        let mut app = bolt_count_test_app(8);

        for _ in 0..9 {
            app.world_mut().spawn(ScenarioTagBolt);
        }

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert_eq!(log.0[0].invariant, InvariantKind::BoltCountReasonable);
    }

    #[test]
    fn bolt_count_reasonable_does_not_fire_at_max() {
        let mut app = bolt_count_test_app(8);

        for _ in 0..8 {
            app.world_mut().spawn(ScenarioTagBolt);
        }

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }

    #[test]
    fn bolt_count_reasonable_uses_scenario_params() {
        let mut app = bolt_count_test_app(12);

        for _ in 0..10 {
            app.world_mut().spawn(ScenarioTagBolt);
        }

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "10 bolts should be OK with max_bolt_count=12"
        );
    }

    // -------------------------------------------------------------------------
    // check_valid_breaker_state helpers
    // -------------------------------------------------------------------------

    fn test_app_valid_breaker_state() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ViolationLog::default());
        app.insert_resource(ScenarioFrame::default());
        app.add_systems(FixedUpdate, check_valid_breaker_state);
        app
    }

    // -------------------------------------------------------------------------
    // ValidBreakerState — illegal transition fires violation
    // -------------------------------------------------------------------------

    /// `Idle → Braking` is illegal (must go through `Dashing`). The system must
    /// append a [`ViolationEntry`] with [`InvariantKind::ValidBreakerState`].
    ///
    /// Tick 1 seeds `Local` with `Idle`. Tick 2 sees `Braking` → violation.
    #[test]
    fn valid_breaker_state_fires_on_idle_to_braking() {
        let mut app = test_app_valid_breaker_state();

        let entity = app
            .world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Idle))
            .id();

        // Tick 1: system stores Idle in Local, no previous to compare → no violation
        tick(&mut app);

        let log_after_tick1 = app.world().resource::<ViolationLog>();
        assert!(
            log_after_tick1.0.is_empty(),
            "no violation expected on first tick (no previous state)"
        );

        // Mutate to Braking (illegal: Idle → Braking)
        *app.world_mut()
            .entity_mut(entity)
            .get_mut::<BreakerState>()
            .unwrap() = BreakerState::Braking;

        // Tick 2: system compares Braking vs previous Idle → should fire
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly one ValidBreakerState violation, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::ValidBreakerState);
    }

    // -------------------------------------------------------------------------
    // ValidBreakerState — legal transition does not fire
    // -------------------------------------------------------------------------

    /// `Idle → Dashing` is a legal transition. No violation should be recorded.
    #[test]
    fn valid_breaker_state_does_not_fire_on_idle_to_dashing() {
        let mut app = test_app_valid_breaker_state();

        let entity = app
            .world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Idle))
            .id();

        // Tick 1: seeds Local with Idle
        tick(&mut app);

        // Change to Dashing (legal: Idle → Dashing)
        *app.world_mut()
            .entity_mut(entity)
            .get_mut::<BreakerState>()
            .unwrap() = BreakerState::Dashing;

        // Tick 2: should NOT fire
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation for Idle→Dashing (legal), got: {:?}",
            log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
    }

    /// `Idle → Dashing` is a legal transition per the state machine.
    ///
    /// Tick 1 seeds `Local` with `Idle`. Tick 2 sees `Dashing`. No
    /// [`ViolationEntry`] with [`InvariantKind::ValidBreakerState`] should fire.
    #[test]
    fn check_valid_breaker_state_legal_idle_to_dashing_produces_no_violation() {
        let mut app = test_app_valid_breaker_state();

        let entity = app
            .world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Idle))
            .id();

        // Tick 1: seeds Local with Idle — no previous, no violation
        tick(&mut app);

        // Transition to Dashing (legal)
        *app.world_mut()
            .entity_mut(entity)
            .get_mut::<BreakerState>()
            .unwrap() = BreakerState::Dashing;

        // Tick 2: Idle → Dashing is legal, log must remain empty
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no ValidBreakerState violation for Idle→Dashing (legal), got: {:?}",
            log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
    }

    /// `Idle → Braking` skips the required `Dashing` intermediate state.
    ///
    /// Tick 1 seeds `Local` with `Idle`. Tick 2 sees `Braking`. The
    /// [`ViolationLog`] must contain exactly 1 entry with
    /// [`InvariantKind::ValidBreakerState`].
    #[test]
    fn check_valid_breaker_state_illegal_idle_to_braking_produces_violation() {
        let mut app = test_app_valid_breaker_state();

        let entity = app
            .world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Idle))
            .id();

        // Tick 1: seeds Local with Idle
        tick(&mut app);

        assert!(
            app.world().resource::<ViolationLog>().0.is_empty(),
            "no violation expected on first tick (no previous state)"
        );

        // Transition to Braking (illegal: skips Dashing)
        *app.world_mut()
            .entity_mut(entity)
            .get_mut::<BreakerState>()
            .unwrap() = BreakerState::Braking;

        // Tick 2: must fire ValidBreakerState violation
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly 1 ValidBreakerState violation for Idle→Braking, got {}",
            log.0.len()
        );
        assert_eq!(
            log.0[0].invariant,
            InvariantKind::ValidBreakerState,
            "expected invariant kind ValidBreakerState"
        );
    }

    // -------------------------------------------------------------------------
    // ValidBreakerState — same state does not fire
    // -------------------------------------------------------------------------

    /// When the state does not change (`Idle → Idle`), no violation should fire.
    #[test]
    fn valid_breaker_state_does_not_fire_on_no_state_change() {
        let mut app = test_app_valid_breaker_state();

        app.world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Idle));

        // Tick 1: seeds Local
        tick(&mut app);
        // Tick 2: same state
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when state does not change"
        );
    }

    // -------------------------------------------------------------------------
    // ValidBreakerState — first frame (no previous) skipped
    // -------------------------------------------------------------------------

    /// On the very first tick (no previous state stored in `Local`), the system must
    /// not fire even for `Dashing` — there is no prior state to compare.
    #[test]
    fn valid_breaker_state_skips_first_frame_with_no_previous() {
        let mut app = test_app_valid_breaker_state();

        // Start directly in Dashing (would be illegal from Idle, but first frame only)
        app.world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Dashing));

        // Only one tick — Local starts empty, no comparison possible
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation on first frame (Local has no previous)"
        );
    }

    // -------------------------------------------------------------------------
    // check_timer_monotonically_decreasing helpers
    // -------------------------------------------------------------------------

    fn test_app_timer_monotonic() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ViolationLog::default());
        app.insert_resource(ScenarioFrame::default());
        app.add_systems(FixedUpdate, check_timer_monotonically_decreasing);
        app
    }

    // -------------------------------------------------------------------------
    // TimerMonotonicallyDecreasing — fires when timer increases
    // -------------------------------------------------------------------------

    /// Timer increases from 50.0 to 55.0 — a violation must be recorded.
    ///
    /// Tick 1: insert `NodeTimer { remaining: 50.0, total: 60.0 }` → seeds `Local(50.0)`.
    /// Tick 2: update to `remaining: 55.0` → fires violation.
    #[test]
    fn timer_monotonically_decreasing_fires_when_timer_increases() {
        let mut app = test_app_timer_monotonic();

        app.insert_resource(NodeTimer {
            remaining: 50.0,
            total: 60.0,
        });

        // Tick 1: seeds Local with 50.0
        tick(&mut app);

        assert!(
            app.world().resource::<ViolationLog>().0.is_empty(),
            "no violation expected after seeding tick"
        );

        // Update timer to a higher value (illegal increase)
        app.world_mut().resource_mut::<NodeTimer>().remaining = 55.0;

        // Tick 2: 55.0 > 50.0 → violation
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly one TimerMonotonicallyDecreasing violation, got {}",
            log.0.len()
        );
        assert_eq!(
            log.0[0].invariant,
            InvariantKind::TimerMonotonicallyDecreasing
        );
    }

    // -------------------------------------------------------------------------
    // TimerMonotonicallyDecreasing — allows decrease
    // -------------------------------------------------------------------------

    /// Timer decreasing from 50.0 to 49.0 is correct. No violation should fire.
    #[test]
    fn timer_monotonically_decreasing_does_not_fire_when_timer_decreases() {
        let mut app = test_app_timer_monotonic();

        app.insert_resource(NodeTimer {
            remaining: 50.0,
            total: 60.0,
        });

        // Tick 1: seeds Local with 50.0
        tick(&mut app);

        // Decrease timer (correct behavior)
        app.world_mut().resource_mut::<NodeTimer>().remaining = 49.0;

        // Tick 2: 49.0 < 50.0 → no violation
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when timer decreases from 50.0 to 49.0"
        );
    }

    // -------------------------------------------------------------------------
    // TimerMonotonicallyDecreasing — skips when no NodeTimer resource
    // -------------------------------------------------------------------------

    /// When [`NodeTimer`] is not present, the system must do nothing.
    #[test]
    fn timer_monotonically_decreasing_skips_when_no_node_timer() {
        let mut app = test_app_timer_monotonic();
        // No NodeTimer inserted

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when NodeTimer is absent"
        );
    }

    // -------------------------------------------------------------------------
    // TimerMonotonicallyDecreasing — resets Local when NodeTimer removed
    // -------------------------------------------------------------------------

    /// When [`NodeTimer`] disappears and then reappears at 60.0, Local must have
    /// been reset so no spurious violation fires.
    #[test]
    fn timer_monotonically_decreasing_resets_local_when_timer_removed() {
        let mut app = test_app_timer_monotonic();

        // Start with NodeTimer present
        app.insert_resource(NodeTimer {
            remaining: 50.0,
            total: 60.0,
        });

        // Tick 1: seeds Local with 50.0
        tick(&mut app);

        // Remove NodeTimer → system should reset Local
        app.world_mut().remove_resource::<NodeTimer>();

        // Tick 2: NodeTimer absent → no violation, Local reset
        tick(&mut app);

        let log_after_removal = app.world().resource::<ViolationLog>();
        assert!(
            log_after_removal.0.is_empty(),
            "expected no violation when NodeTimer is absent"
        );

        // Reinsert NodeTimer at 60.0 (higher than old 50.0, but Local was reset)
        app.insert_resource(NodeTimer {
            remaining: 60.0,
            total: 60.0,
        });

        // Tick 3: 60.0 appears fresh — no previous value → no violation
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when NodeTimer reappears after reset (Local was cleared)"
        );
    }

    // -------------------------------------------------------------------------
    // check_breaker_position_clamped helpers
    // -------------------------------------------------------------------------

    fn test_app_breaker_position_clamped() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ViolationLog::default());
        app.insert_resource(ScenarioFrame::default());
        app.insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
        });
        app.add_systems(FixedUpdate, check_breaker_position_clamped);
        app
    }

    // -------------------------------------------------------------------------
    // BreakerPositionClamped — fires when outside tight bounds
    // -------------------------------------------------------------------------

    /// Breaker at x=1000.0 is well outside `right() - half_width` (400.0 - 60.0 = 340.0).
    /// A [`ViolationEntry`] with [`InvariantKind::BreakerPositionClamped`] must fire.
    #[test]
    fn breaker_position_clamped_fires_when_outside_bounds() {
        let mut app = test_app_breaker_position_clamped();

        // BreakerWidth(120.0) → half_width = 60.0; right() = 400.0 → clamped max = 340.0
        app.world_mut().spawn((
            ScenarioTagBreaker,
            Transform::from_translation(Vec3::new(1000.0, -250.0, 0.0)),
            BreakerWidth(120.0),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly one BreakerPositionClamped violation, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BreakerPositionClamped);
    }

    // -------------------------------------------------------------------------
    // BreakerPositionClamped — allows within tight bounds
    // -------------------------------------------------------------------------

    /// Breaker at x=0.0 is well within bounds. No violation should fire.
    #[test]
    fn breaker_position_clamped_does_not_fire_when_within_bounds() {
        let mut app = test_app_breaker_position_clamped();

        app.world_mut().spawn((
            ScenarioTagBreaker,
            Transform::from_translation(Vec3::new(0.0, -250.0, 0.0)),
            BreakerWidth(120.0),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation for breaker at x=0.0"
        );
    }

    // -------------------------------------------------------------------------
    // BreakerPositionClamped — allows at exact boundary (within 1px tolerance)
    // -------------------------------------------------------------------------

    /// Breaker at x = 340.0 (exactly `right() - half_width = 400.0 - 60.0`)
    /// is within tolerance. No violation should fire.
    #[test]
    fn breaker_position_clamped_does_not_fire_at_exact_boundary() {
        let mut app = test_app_breaker_position_clamped();

        // Exact boundary: right() - half_width = 400.0 - 60.0 = 340.0
        app.world_mut().spawn((
            ScenarioTagBreaker,
            Transform::from_translation(Vec3::new(340.0, -250.0, 0.0)),
            BreakerWidth(120.0),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when breaker is exactly at clamped boundary (340.0)"
        );
    }

    // -------------------------------------------------------------------------
    // check_physics_frozen_during_pause helpers
    // -------------------------------------------------------------------------

    fn test_app_physics_frozen() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.add_sub_state::<PlayingState>();
        app.insert_resource(ViolationLog::default());
        app.insert_resource(ScenarioFrame::default());
        app.add_systems(FixedUpdate, check_physics_frozen_during_pause);
        app
    }

    // -------------------------------------------------------------------------
    // PhysicsFrozenDuringPause — fires when position changes during pause
    // -------------------------------------------------------------------------

    /// When [`PlayingState`] is `Paused` and a tagged bolt moves between ticks,
    /// a [`ViolationEntry`] with [`InvariantKind::PhysicsFrozenDuringPause`] fires.
    ///
    /// Tick 1 (Active): seeds Local with position (100.0, 200.0, 0.0).
    /// Then transition to Paused.
    /// Tick 2 (Paused): bolt moved to (105.0, 200.0, 0.0) → violation.
    #[test]
    fn physics_frozen_during_pause_fires_when_bolt_moves_during_pause() {
        let mut app = test_app_physics_frozen();

        // Enter Playing (needed for PlayingState to be active)
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update(); // process state transition

        let entity = app
            .world_mut()
            .spawn((
                ScenarioTagBolt,
                Transform::from_translation(Vec3::new(100.0, 200.0, 0.0)),
            ))
            .id();

        // Tick 1 in Active: system stores (100.0, 200.0, 0.0) in Local
        tick(&mut app);

        // Transition to Paused
        app.world_mut()
            .resource_mut::<NextState<PlayingState>>()
            .set(PlayingState::Paused);
        app.update(); // process sub-state transition

        // Move the bolt while paused
        app.world_mut()
            .entity_mut(entity)
            .get_mut::<Transform>()
            .unwrap()
            .translation = Vec3::new(105.0, 200.0, 0.0);

        // Tick 2: game is paused and bolt moved → violation
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly one PhysicsFrozenDuringPause violation, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::PhysicsFrozenDuringPause);
    }

    // -------------------------------------------------------------------------
    // PhysicsFrozenDuringPause — allows movement during Active
    // -------------------------------------------------------------------------

    /// When [`PlayingState`] is `Active`, bolt movement is expected. No violation should fire.
    #[test]
    fn physics_frozen_during_pause_does_not_fire_when_active() {
        let mut app = test_app_physics_frozen();

        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();

        let entity = app
            .world_mut()
            .spawn((
                ScenarioTagBolt,
                Transform::from_translation(Vec3::new(100.0, 200.0, 0.0)),
            ))
            .id();

        // Tick 1: seeds Local with position
        tick(&mut app);

        // Move bolt (game is Active — movement is legal)
        app.world_mut()
            .entity_mut(entity)
            .get_mut::<Transform>()
            .unwrap()
            .translation = Vec3::new(200.0, 200.0, 0.0);

        // Tick 2: Active state → no violation
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when bolt moves during Active state"
        );
    }

    // -------------------------------------------------------------------------
    // PhysicsFrozenDuringPause — clears when PlayingState absent
    // -------------------------------------------------------------------------

    /// When [`PlayingState`] is absent (game not in `Playing`), the system must
    /// do nothing and not panic.
    #[test]
    fn physics_frozen_during_pause_clears_when_playing_state_absent() {
        let mut app = test_app_physics_frozen();

        // Do NOT enter Playing — PlayingState is absent

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(100.0, 200.0, 0.0)),
        ));

        // Tick with no PlayingState in world → should not panic, no violation
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when PlayingState is absent"
        );
    }

    // =========================================================================
    // Fix 7: check_bolt_in_bounds — all 4 boundaries
    // =========================================================================

    // -------------------------------------------------------------------------
    // BoltInBounds — violation fires when bolt is above top bound
    // -------------------------------------------------------------------------

    /// A bolt at y = 1000.0 exceeds the top bound of a playfield with height=700.0
    /// (top = 350.0). The system must append one [`ViolationEntry`] with
    /// [`InvariantKind::BoltInBounds`].
    ///
    /// Current production code only checks `y < bottom` — this test MUST FAIL
    /// until the top-bound check is added.
    #[test]
    fn bolt_in_bounds_fires_when_bolt_is_above_top_bound() {
        let mut app = test_app_bolt_in_bounds();

        // width=800.0, height=700.0 → top() = 350.0
        app.world_mut().insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
        });
        app.world_mut().insert_resource(ScenarioFrame(1));

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(0.0, 1000.0, 0.0)),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly 1 BoltInBounds violation for bolt above top bound (y=1000.0 > top=350.0), got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BoltInBounds);
    }

    // -------------------------------------------------------------------------
    // BoltInBounds — no violation when bolt is exactly at top bound (strict >)
    // -------------------------------------------------------------------------

    /// A bolt exactly at y = 350.0 (the top boundary of a 700.0-height playfield)
    /// must NOT trigger a violation — the check is strict `>`.
    #[test]
    fn bolt_in_bounds_does_not_fire_when_bolt_is_exactly_at_top_bound() {
        let mut app = test_app_bolt_in_bounds();

        // top() = 700.0 / 2.0 = 350.0
        app.world_mut().insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
        });
        app.world_mut().insert_resource(ScenarioFrame(1));

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(0.0, 350.0, 0.0)),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when bolt is exactly at top bound (350.0) — check is strict >"
        );
    }

    // -------------------------------------------------------------------------
    // BoltInBounds — violation fires when bolt is left of left bound
    // -------------------------------------------------------------------------

    /// A bolt at x = -2000.0 exceeds the left bound of a playfield with
    /// width=800.0 (left = -400.0). The system must append one
    /// [`ViolationEntry`] with [`InvariantKind::BoltInBounds`].
    ///
    /// Current production code only checks `y < bottom` — this test MUST FAIL
    /// until the left-bound check is added.
    #[test]
    fn bolt_in_bounds_fires_when_bolt_is_left_of_left_bound() {
        let mut app = test_app_bolt_in_bounds();

        // width=800.0 → left() = -400.0
        app.world_mut().insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
        });
        app.world_mut().insert_resource(ScenarioFrame(1));

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(-2000.0, 0.0, 0.0)),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly 1 BoltInBounds violation for bolt left of left bound (x=-2000.0 < left=-400.0), got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BoltInBounds);
    }

    // -------------------------------------------------------------------------
    // BoltInBounds — violation fires when bolt is right of right bound
    // -------------------------------------------------------------------------

    /// A bolt at x = 2000.0 exceeds the right bound of a playfield with
    /// width=800.0 (right = 400.0). The system must append one
    /// [`ViolationEntry`] with [`InvariantKind::BoltInBounds`].
    ///
    /// Current production code only checks `y < bottom` — this test MUST FAIL
    /// until the right-bound check is added.
    #[test]
    fn bolt_in_bounds_fires_when_bolt_is_right_of_right_bound() {
        let mut app = test_app_bolt_in_bounds();

        // width=800.0 → right() = 400.0
        app.world_mut().insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
        });
        app.world_mut().insert_resource(ScenarioFrame(1));

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(2000.0, 0.0, 0.0)),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly 1 BoltInBounds violation for bolt right of right bound (x=2000.0 > right=400.0), got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BoltInBounds);
    }

    // =========================================================================
    // Fix 8: check_valid_breaker_state — per-entity tracking via HashMap
    // =========================================================================

    // -------------------------------------------------------------------------
    // ValidBreakerState — two breakers tracked independently
    // -------------------------------------------------------------------------

    /// Two [`ScenarioTagBreaker`] entities are tracked independently. When entity A
    /// makes a legal transition (`Idle → Dashing`) and entity B makes an illegal
    /// transition (`Idle → Braking`), exactly one violation fires — for entity B.
    ///
    /// The current production code uses `Local<Option<BreakerState>>` which mixes
    /// state across all entities (last entity wins). This test MUST FAIL until
    /// `Local<HashMap<Entity, BreakerState>>` is used.
    #[test]
    fn valid_breaker_state_tracks_two_breakers_independently_one_illegal() {
        let mut app = test_app_valid_breaker_state();

        // Spawn entity A and entity B, both starting Idle
        let entity_a = app
            .world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Idle))
            .id();
        let entity_b = app
            .world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Idle))
            .id();

        // Tick 1: seeds Local for both A (Idle) and B (Idle)
        tick(&mut app);

        assert!(
            app.world().resource::<ViolationLog>().0.is_empty(),
            "no violation expected after seeding tick (no previous state to compare)"
        );

        // Entity A: Idle → Dashing (legal)
        *app.world_mut()
            .entity_mut(entity_a)
            .get_mut::<BreakerState>()
            .unwrap() = BreakerState::Dashing;

        // Entity B: Idle → Braking (illegal — skips Dashing)
        *app.world_mut()
            .entity_mut(entity_b)
            .get_mut::<BreakerState>()
            .unwrap() = BreakerState::Braking;

        // Tick 2: A is legal, B is illegal → exactly 1 violation
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly 1 ValidBreakerState violation (entity B's Idle→Braking is illegal), got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::ValidBreakerState);
    }

    // -------------------------------------------------------------------------
    // ValidBreakerState — two breakers both transitioning legally produce no violation
    // -------------------------------------------------------------------------

    /// When both [`ScenarioTagBreaker`] entities make legal transitions
    /// (`Idle → Dashing`), no [`ViolationEntry`] should be recorded.
    ///
    /// Current production code with `Local<Option<BreakerState>>` may incorrectly
    /// fire a violation because only the last entity's state survives in `Local`.
    /// This test MUST FAIL until per-entity `HashMap` tracking is implemented.
    #[test]
    fn valid_breaker_state_produces_no_violation_when_both_breakers_transition_legally() {
        let mut app = test_app_valid_breaker_state();

        // Spawn two breakers, both Idle
        let entity_a = app
            .world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Idle))
            .id();
        let entity_b = app
            .world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Idle))
            .id();

        // Tick 1: seeds Local for A=Idle, B=Idle
        tick(&mut app);

        assert!(
            app.world().resource::<ViolationLog>().0.is_empty(),
            "no violation expected on seeding tick"
        );

        // Both transition Idle → Dashing (legal)
        *app.world_mut()
            .entity_mut(entity_a)
            .get_mut::<BreakerState>()
            .unwrap() = BreakerState::Dashing;
        *app.world_mut()
            .entity_mut(entity_b)
            .get_mut::<BreakerState>()
            .unwrap() = BreakerState::Dashing;

        // Tick 2: both legal → no violation
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no ValidBreakerState violation when both breakers transition Idle→Dashing (legal), got: {:?}",
            log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
    }
}
