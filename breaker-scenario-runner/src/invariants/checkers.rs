use bevy::{platform::collections::HashMap, prelude::*};
use breaker::{
    bolt::components::{BoltMaxSpeed, BoltMinSpeed, BoltRadius, BoltVelocity},
    breaker::components::{BreakerState, BreakerWidth},
    run::node::{messages::SpawnNodeComplete, resources::NodeTimer},
    shared::{GameState, PlayfieldConfig, PlayingState},
};

use super::types::*;
use crate::{lifecycle::ScenarioConfig, types::InvariantKind};

/// Per-entity previous position map for pause-freeze checking.
///
/// Stored in a [`Local`] to track bolt positions between fixed-update ticks.
type PreviousBoltPositions = HashMap<Entity, Vec3>;

/// Query filter that matches entities tagged for invariant checking.
type TaggedTransformQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Transform),
    Or<(With<ScenarioTagBolt>, With<ScenarioTagBreaker>)>,
>;

/// Checks that [`BreakerState`] transitions on the tagged breaker follow the legal path.
///
/// Legal transitions: `Idle → Dashing`, `Settling → Dashing` (re-dash),
/// `Dashing → Braking`, `Dashing → Settling` (dash cancel),
/// `Braking → Settling`, `Settling → Idle`. Any other change fires a [`ViolationEntry`] with
/// [`InvariantKind::ValidBreakerState`].
///
/// Clears tracking on [`GameState`] transitions (e.g., entering `Playing` after a
/// node change) so that forced `reset_breaker` resets to `Idle` are not flagged.
///
/// Skips the first frame per entity (no previous state stored yet for that entity).
pub fn check_valid_breaker_state(
    breakers: Query<(Entity, &BreakerState), With<ScenarioTagBreaker>>,
    mut previous: Local<HashMap<Entity, BreakerState>>,
    game_state: Res<State<GameState>>,
    mut prev_game_state: Local<Option<GameState>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let current_game = **game_state;
    // On game-state transition (e.g., entering Playing after a node change),
    // clear tracking — `reset_breaker` may have forcibly set any breaker to
    // `Idle`, which is not a state-machine violation.
    if let Some(prev_gs) = *prev_game_state
        && prev_gs != current_game
    {
        previous.clear();
    }
    *prev_game_state = Some(current_game);

    for (entity, &current) in &breakers {
        if let Some(&prev) = previous.get(&entity)
            && prev != current
        {
            let legal = matches!(
                (prev, current),
                (
                    BreakerState::Idle | BreakerState::Settling,
                    BreakerState::Dashing
                ) | (
                    BreakerState::Dashing,
                    BreakerState::Braking | BreakerState::Settling
                ) | (BreakerState::Braking, BreakerState::Settling)
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
    previous.retain(|e, _| breakers.contains(*e));
}

/// Checks that [`NodeTimer::remaining`] never increases between ticks.
///
/// Stores `(remaining, total)` from the previous tick in a `Local`. Resets when
/// `total` changes (node transition) or when `remaining` jumps back near `total`
/// (same-duration node transition). If `remaining` increases otherwise, appends a
/// [`ViolationEntry`] with [`InvariantKind::TimerMonotonicallyDecreasing`].
///
/// Skips and resets when [`NodeTimer`] is absent.
pub fn check_timer_monotonically_decreasing(
    timer: Option<Res<NodeTimer>>,
    mut previous: Local<Option<(f32, f32)>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let Some(timer) = timer else {
        *previous = None;
        return;
    };
    let current = timer.remaining;
    let current_total = timer.total;
    if let Some((prev_remaining, prev_total)) = *previous {
        if (current_total - prev_total).abs() > f32::EPSILON {
            // Node transition — total changed, reset tracking
            *previous = Some((current, current_total));
            return;
        }
        if prev_remaining > 0.0 && current > prev_remaining {
            // Check if this looks like a freshly initialized timer (new node
            // with the same duration). On the first tick of a new node,
            // remaining ≈ total. A real intra-node bug would have remaining
            // somewhere in the middle, not near total.
            let near_total = (current - current_total).abs() < 1.0;
            if near_total {
                // Same-duration node transition — reset tracking
                *previous = Some((current, current_total));
                return;
            }
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::TimerMonotonicallyDecreasing,
                entity: None,
                message: format!(
                    "TimerMonotonicallyDecreasing FAIL frame={} remaining increased {prev_remaining:.3} → {current:.3}",
                    frame.0,
                ),
            });
        }
    }
    *previous = Some((current, current_total));
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

/// Checks that all [`ScenarioTagBolt`] entities remain within playfield bounds.
///
/// Appends a [`ViolationEntry`] to [`ViolationLog`] for every bolt whose
/// `Transform` translation is outside the top, left, or right playfield boundaries,
/// expanded by `BoltRadius + 1.0` when [`BoltRadius`] is present (zero margin when
/// absent). The bottom is intentionally open (no floor wall) — bolts exit through
/// the bottom during life-loss, so no bottom check is performed.
///
/// Increments [`ScenarioStats::invariant_checks`] by the number of bolts checked.
pub fn check_bolt_in_bounds(
    bolts: Query<(Entity, &Transform, Option<&BoltRadius>), With<ScenarioTagBolt>>,
    playfield: Res<PlayfieldConfig>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    let top = playfield.top();
    let left = playfield.left();
    let right = playfield.right();
    let mut checks = 0u32;
    for (entity, transform, bolt_radius) in &bolts {
        checks += 1;
        let x = transform.translation.x;
        let y = transform.translation.y;
        let margin = bolt_radius.map_or(0.0, |r| r.0 + 1.0);
        // No bottom check — the floor is intentionally open (no wall). The bolt
        // exits through the bottom during life-loss, handled by `bolt_lost`.
        if y > top + margin {
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
        if x < left - margin {
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
        if x > right + margin {
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

/// Checks for unexpected entity accumulation over time.
///
/// Waits for [`SpawnNodeComplete`] to fire (all domain spawn systems done),
/// then samples the baseline entity count immediately. Every 120 frames after
/// that, checks if count exceeds 2× baseline.
pub fn check_no_entity_leaks(
    all_entities: Query<Entity>,
    frame: Res<ScenarioFrame>,
    mut spawn_reader: MessageReader<SpawnNodeComplete>,
    mut baseline: ResMut<EntityLeakBaseline>,
    mut log: ResMut<ViolationLog>,
) {
    let count = all_entities.iter().count();

    // When SpawnNodeComplete arrives, all gameplay entities are spawned — sample now.
    for _ in spawn_reader.read() {
        baseline.baseline = Some(count);
    }

    let Some(base) = baseline.baseline else {
        return;
    };

    // Check every 120 frames (~1.9 s at 64 Hz fixed timestep)
    if frame.0.is_multiple_of(120) && count > base * 2 {
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
    const SPEED_TOLERANCE: f32 = 1.0;
    for (entity, velocity, min_speed, max_speed) in &bolts {
        let speed = velocity.speed();
        if speed < f32::EPSILON {
            continue;
        }
        if speed < min_speed.0 - SPEED_TOLERANCE || speed > max_speed.0 + SPEED_TOLERANCE {
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
