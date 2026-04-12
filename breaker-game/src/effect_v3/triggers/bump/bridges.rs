//! Bump trigger bridge systems.
//!
//! Each bridge reads a bump message, builds a [`TriggerContext`], and dispatches
//! the corresponding trigger to entities with bound effects.

use bevy::prelude::*;

use crate::{
    breaker::messages::{BumpGrade, BumpPerformed, BumpWhiffed},
    effect_v3::{
        storage::BoundEffects,
        types::{Trigger, TriggerContext},
        walking::walk_effects,
    },
};

// ---------------------------------------------------------------------------
// Local bump bridges — fire on bolt + breaker involved in the bump
// ---------------------------------------------------------------------------

/// Local bridge: fires `Bumped` on the bolt and breaker entities involved in a
/// successful bump of any grade.
pub fn on_bumped(
    mut reader: MessageReader<BumpPerformed>,
    bound_query: Query<&BoundEffects>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let context = TriggerContext::Bump {
            bolt:    msg.bolt,
            breaker: msg.breaker,
        };
        walk_local_bump(
            &Trigger::Bumped,
            &context,
            msg.bolt,
            msg.breaker,
            &bound_query,
            &mut commands,
        );
    }
}

/// Local bridge: fires `PerfectBumped` on the bolt and breaker entities involved
/// in a perfect-timed bump.
pub fn on_perfect_bumped(
    mut reader: MessageReader<BumpPerformed>,
    bound_query: Query<&BoundEffects>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if msg.grade != BumpGrade::Perfect {
            continue;
        }
        let context = TriggerContext::Bump {
            bolt:    msg.bolt,
            breaker: msg.breaker,
        };
        walk_local_bump(
            &Trigger::PerfectBumped,
            &context,
            msg.bolt,
            msg.breaker,
            &bound_query,
            &mut commands,
        );
    }
}

/// Local bridge: fires `EarlyBumped` on the bolt and breaker entities involved
/// in an early-timed bump.
pub fn on_early_bumped(
    mut reader: MessageReader<BumpPerformed>,
    bound_query: Query<&BoundEffects>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if msg.grade != BumpGrade::Early {
            continue;
        }
        let context = TriggerContext::Bump {
            bolt:    msg.bolt,
            breaker: msg.breaker,
        };
        walk_local_bump(
            &Trigger::EarlyBumped,
            &context,
            msg.bolt,
            msg.breaker,
            &bound_query,
            &mut commands,
        );
    }
}

/// Local bridge: fires `LateBumped` on the bolt and breaker entities involved
/// in a late-timed bump.
pub fn on_late_bumped(
    mut reader: MessageReader<BumpPerformed>,
    bound_query: Query<&BoundEffects>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if msg.grade != BumpGrade::Late {
            continue;
        }
        let context = TriggerContext::Bump {
            bolt:    msg.bolt,
            breaker: msg.breaker,
        };
        walk_local_bump(
            &Trigger::LateBumped,
            &context,
            msg.bolt,
            msg.breaker,
            &bound_query,
            &mut commands,
        );
    }
}

// ---------------------------------------------------------------------------
// Global bump bridges — fire on ALL entities with BoundEffects
// ---------------------------------------------------------------------------

/// Global bridge: fires `BumpOccurred` on all entities with bound effects when
/// any successful bump happens.
pub fn on_bump_occurred(
    mut reader: MessageReader<BumpPerformed>,
    bound_query: Query<(Entity, &BoundEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let context = TriggerContext::Bump {
            bolt:    msg.bolt,
            breaker: msg.breaker,
        };
        walk_global(
            &Trigger::BumpOccurred,
            &context,
            &bound_query,
            &mut commands,
        );
    }
}

/// Global bridge: fires `PerfectBumpOccurred` on all entities with bound effects
/// when a perfect bump happens.
pub fn on_perfect_bump_occurred(
    mut reader: MessageReader<BumpPerformed>,
    bound_query: Query<(Entity, &BoundEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if msg.grade != BumpGrade::Perfect {
            continue;
        }
        let context = TriggerContext::Bump {
            bolt:    msg.bolt,
            breaker: msg.breaker,
        };
        walk_global(
            &Trigger::PerfectBumpOccurred,
            &context,
            &bound_query,
            &mut commands,
        );
    }
}

/// Global bridge: fires `EarlyBumpOccurred` on all entities with bound effects
/// when an early bump happens.
pub fn on_early_bump_occurred(
    mut reader: MessageReader<BumpPerformed>,
    bound_query: Query<(Entity, &BoundEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if msg.grade != BumpGrade::Early {
            continue;
        }
        let context = TriggerContext::Bump {
            bolt:    msg.bolt,
            breaker: msg.breaker,
        };
        walk_global(
            &Trigger::EarlyBumpOccurred,
            &context,
            &bound_query,
            &mut commands,
        );
    }
}

/// Global bridge: fires `LateBumpOccurred` on all entities with bound effects
/// when a late bump happens.
pub fn on_late_bump_occurred(
    mut reader: MessageReader<BumpPerformed>,
    bound_query: Query<(Entity, &BoundEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if msg.grade != BumpGrade::Late {
            continue;
        }
        let context = TriggerContext::Bump {
            bolt:    msg.bolt,
            breaker: msg.breaker,
        };
        walk_global(
            &Trigger::LateBumpOccurred,
            &context,
            &bound_query,
            &mut commands,
        );
    }
}

/// Global bridge: fires `BumpWhiffOccurred` on all entities with bound effects
/// when a bump timing window expires without contact.
pub fn on_bump_whiff_occurred(
    mut reader: MessageReader<BumpWhiffed>,
    bound_query: Query<(Entity, &BoundEffects)>,
    mut commands: Commands,
) {
    for _ in reader.read() {
        let context = TriggerContext::None;
        walk_global(
            &Trigger::BumpWhiffOccurred,
            &context,
            &bound_query,
            &mut commands,
        );
    }
}

/// Global bridge: fires `NoBumpOccurred` on all entities with bound effects
/// when a bolt hits the breaker without any bump input.
///
/// TODO: Requires a `NoBump` message from the bump grading system.
/// The bump system needs to send this when `BoltImpactBreaker` arrives
/// with no active bump input. This bridge is a no-op until that message exists.
pub const fn on_no_bump_occurred() {
    // No-op until NoBump message is created in the breaker domain.
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Walk effects on both bolt and breaker (local dispatch).
fn walk_local_bump(
    trigger: &Trigger,
    context: &TriggerContext,
    bolt: Option<Entity>,
    breaker: Entity,
    bound_query: &Query<&BoundEffects>,
    commands: &mut Commands,
) {
    // Walk breaker effects
    if let Ok(bound) = bound_query.get(breaker) {
        let trees = bound.0.clone();
        walk_effects(breaker, trigger, context, &trees, commands);
    }
    // Walk bolt effects (if bolt entity exists)
    if let Some(bolt_entity) = bolt
        && let Ok(bound) = bound_query.get(bolt_entity)
    {
        let trees = bound.0.clone();
        walk_effects(bolt_entity, trigger, context, &trees, commands);
    }
}

/// Walk effects on all entities with `BoundEffects` (global dispatch).
fn walk_global(
    trigger: &Trigger,
    context: &TriggerContext,
    bound_query: &Query<(Entity, &BoundEffects)>,
    commands: &mut Commands,
) {
    for (entity, bound) in bound_query.iter() {
        let trees = bound.0.clone();
        walk_effects(entity, trigger, context, &trees, commands);
    }
}
