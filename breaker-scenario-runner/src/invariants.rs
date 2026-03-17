//! Invariant checker systems and `ViolationLog` resource.
//!
//! Invariant systems run in `FixedUpdate` after gameplay. They query game state
//! and append to a [`ViolationLog`] resource. They never panic — they collect
//! all violations for end-of-run reporting.

use bevy::prelude::*;
use breaker::{
    bolt::components::{BoltMaxSpeed, BoltMinSpeed, BoltVelocity},
    run::node::resources::NodeTimer,
    shared::{GameState, PlayfieldConfig},
};

use crate::{lifecycle::ScenarioConfig, types::InvariantKind};

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
/// `Transform` translation y is below `PlayfieldConfig::bottom()`.
pub fn check_bolt_in_bounds(
    bolts: Query<(Entity, &Transform), With<ScenarioTagBolt>>,
    playfield: Res<PlayfieldConfig>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let bottom = playfield.bottom();
    for (entity, transform) in &bolts {
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
/// - `Loading → Playing` (must go through MainMenu)
/// - `Loading → RunEnd`
/// - `Playing → Loading`
/// - `RunEnd → Playing` (must go through MainMenu)
pub fn check_valid_state_transitions(
    state: Res<State<GameState>>,
    mut previous: ResMut<PreviousGameState>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let current = **state;
    if let Some(prev) = previous.0 {
        if prev != current {
            let forbidden = matches!(
                (prev, current),
                (GameState::Loading, GameState::Playing)
                    | (GameState::Loading, GameState::RunEnd)
                    | (GameState::Playing, GameState::Loading)
                    | (GameState::RunEnd, GameState::Playing)
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

    if frame.0 > 60 && frame.0 % 120 == 0 && count > base * 2 {
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
    bolts: Query<
        (Entity, &BoltVelocity, &BoltMinSpeed, &BoltMaxSpeed),
        With<ScenarioTagBolt>,
    >,
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
            message: format!(
                "BoltCountReasonable FAIL frame={} count={count}",
                frame.0,
            ),
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
        app.insert_resource(EntityLeakBaseline {
            baseline: Some(5),
        });
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
        use crate::types::{InputStrategy, ScenarioDefinition, ScriptedParams, InvariantParams};

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
        assert!(log.0.is_empty(), "10 bolts should be OK with max_bolt_count=12");
    }
}
