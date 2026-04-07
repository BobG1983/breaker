use bevy::prelude::*;
use breaker::{bolt::components::BoltRadius, shared::PlayfieldConfig};
use rantzsoft_spatial2d::components::Position2D;

use crate::{invariants::*, types::InvariantKind};

/// Checks that all [`ScenarioTagBolt`] entities remain within playfield bounds.
///
/// Appends a [`ViolationEntry`] to [`ViolationLog`] for every bolt whose
/// `Position2D` is outside the top, left, or right playfield boundaries,
/// expanded by `BoltRadius + 1.0` when [`BoltRadius`] is present (zero margin when
/// absent). The bottom is intentionally open (no floor wall) — bolts exit through
/// the bottom during life-loss, so no bottom check is performed.
///
/// Gated on [`ScenarioStats::entered_playing`]: when [`ScenarioStats`] is present
/// and `entered_playing` is `false`, the checker early-returns without producing
/// violations. This prevents false positives during `GameState::Loading` before
/// entities are fully initialized.
///
/// Increments [`ScenarioStats::invariant_checks`] by the number of bolts checked.
pub fn check_bolt_in_bounds(
    bolts: Query<(Entity, &Position2D, Option<&BoltRadius>), With<ScenarioTagBolt>>,
    playfield: Res<PlayfieldConfig>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    // Gate: do not check invariants until the game has entered Playing.
    // When ScenarioStats is present but entered_playing is false, we are
    // still in Loading/MainMenu — entities may not be fully initialized.
    if let Some(ref stats) = stats
        && !stats.entered_playing
    {
        return;
    }
    let top = playfield.top();
    let left = playfield.left();
    let right = playfield.right();
    for (entity, position, bolt_radius) in &bolts {
        let x = position.0.x;
        let y = position.0.y;
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
        s.invariant_checks += 1;
    }
}
