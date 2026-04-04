//! Debug overrides for bolt/breaker position, velocity, and physics freeze.

use bevy::prelude::*;
use breaker::state::run::node::resources::NodeTimer;
use rantzsoft_spatial2d::components::Position2D;

use super::types::{BoltDebugQuery, BreakerDebugQuery, ScenarioConfig};
use crate::{
    invariants::{ScenarioPhysicsFrozen, ScenarioTagBolt},
    types::DebugSetup,
};

/// Applies entity-dependent debug overrides (position, velocity, physics freeze)
/// to tagged bolt and breaker entities.
///
/// Shared logic between [`apply_debug_setup`] and [`deferred_debug_setup`].
pub fn apply_entity_debug_overrides(
    setup: &DebugSetup,
    bolt_query: &mut BoltDebugQuery,
    breaker_query: &mut BreakerDebugQuery,
    commands: &mut Commands,
) {
    for (entity, mut position, mut velocity) in bolt_query.iter_mut() {
        if let Some((x, y)) = setup.bolt_position {
            position.0.x = x;
            position.0.y = y;
        }

        if let Some((vx, vy)) = setup.bolt_velocity {
            velocity.0 = Vec2::new(vx, vy);
        }

        if setup.disable_physics {
            commands
                .entity(entity)
                .insert(ScenarioPhysicsFrozen { target: position.0 });
        }
    }

    for (entity, mut position) in breaker_query.iter_mut() {
        if let Some((x, y)) = setup.breaker_position {
            position.0.x = x;
            position.0.y = y;
        }

        if setup.disable_physics {
            commands
                .entity(entity)
                .insert(ScenarioPhysicsFrozen { target: position.0 });
        }
    }
}

/// Applies debug overrides from [`ScenarioConfig`] to tagged bolt and breaker entities.
///
/// For each entity tagged with [`ScenarioTagBolt`] or [`ScenarioTagBreaker`],
/// applies position teleports from [`crate::types::DebugSetup`]. When
/// `disable_physics` is true, inserts [`ScenarioPhysicsFrozen`] on both bolts
/// and breakers with the post-teleport position as the frozen target.
///
/// Also handles `bolt_velocity`, `extra_tagged_bolts`, and `node_timer_remaining`
/// overrides.
pub fn apply_debug_setup(
    config: Res<ScenarioConfig>,
    mut bolt_query: BoltDebugQuery,
    mut breaker_query: BreakerDebugQuery,
    mut commands: Commands,
    node_timer: Option<ResMut<NodeTimer>>,
) {
    let Some(setup) = config.definition.debug_setup.as_ref() else {
        return;
    };

    apply_entity_debug_overrides(setup, &mut bolt_query, &mut breaker_query, &mut commands);

    if let Some(count) = setup.extra_tagged_bolts {
        for _ in 0..count {
            commands.spawn(ScenarioTagBolt);
        }
    }

    if let Some(remaining) = setup.node_timer_remaining
        && let Some(mut timer) = node_timer
    {
        timer.remaining = remaining;
    }
}

/// Deferred fallback for [`apply_debug_setup`].
///
/// Runs once in `FixedUpdate` after
/// [`super::entity_tagging::tag_game_entities`] to catch entities that were not yet spawned when the
/// `OnEnter(NodeState::Loading)` version of `apply_debug_setup` ran.
///
/// Under heavy parallel I/O contention (45+ scenarios loading simultaneously),
/// the `OnEnter` schedule can execute `apply_debug_setup` before spawn systems
/// have flushed their deferred commands, leaving 0 tagged entities to process.
/// This system re-applies the entity-dependent parts of debug setup (position
/// overrides, velocity overrides, and physics freeze) on the first `FixedUpdate`
/// tick where tagged entities exist.
///
/// Uses a [`Local<bool>`] guard so it fires at most once per app lifetime.
/// Non-entity parts (extra tagged bolts, timer override, forced previous state)
/// are handled by the `OnEnter` version and are not repeated here.
pub fn deferred_debug_setup(
    mut done: Local<bool>,
    config: Res<ScenarioConfig>,
    mut bolt_query: BoltDebugQuery,
    mut breaker_query: BreakerDebugQuery,
    mut commands: Commands,
) {
    if *done {
        return;
    }

    let Some(setup) = config.definition.debug_setup.as_ref() else {
        *done = true;
        return;
    };

    // Wait until at least one tagged entity exists before applying.
    if bolt_query.is_empty() && breaker_query.is_empty() {
        return;
    }

    apply_entity_debug_overrides(setup, &mut bolt_query, &mut breaker_query, &mut commands);

    *done = true;
}

/// Resets each entity with [`ScenarioPhysicsFrozen`] back to its pinned `target` every tick.
///
/// Prevents physics systems from moving entities that should be stationary during
/// a self-test scenario. Runs after physics in `FixedUpdate`.
pub fn enforce_frozen_positions(mut frozen: Query<(&ScenarioPhysicsFrozen, &mut Position2D)>) {
    for (pinned, mut position) in &mut frozen {
        position.0 = pinned.target;
    }
}
