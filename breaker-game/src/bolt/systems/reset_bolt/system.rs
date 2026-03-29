//! System to reset the bolt's position and velocity at the start of each node.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    bolt::{
        components::{Bolt, BoltServing, ExtraBolt},
        queries::ResetBoltQuery,
        resources::BoltConfig,
    },
    breaker::components::Breaker,
    run::RunState,
};

/// Resets the bolt's position above the breaker and adjusts velocity for the
/// current node.
///
/// On the first node (`RunState.node_index == 0`), the bolt spawns with zero
/// velocity and a [`BoltServing`] marker. On subsequent nodes it launches
/// immediately at base speed.
///
/// Effect components (e.g. [`EffectivePiercing`], [`EffectiveDamageMultiplier`])
/// are NOT touched -- they persist across nodes. Only positional and velocity
/// state is reset. [`PiercingRemaining`] is reset to [`EffectivePiercing`].
pub(crate) fn reset_bolt(
    mut commands: Commands,
    config: Res<BoltConfig>,
    run_state: Res<RunState>,
    breaker_query: Query<&Position2D, (With<Breaker>, Without<Bolt>)>,
    mut bolt_query: Query<ResetBoltQuery, (With<Bolt>, Without<ExtraBolt>)>,
) {
    let Ok(breaker_pos) = breaker_query.single() else {
        return;
    };

    let breaker_x = breaker_pos.0.x;
    let breaker_y = breaker_pos.0.y;

    let serving = run_state.node_index == 0;

    for (entity, mut position, mut velocity, piercing_remaining, effective_piercing, prev_pos) in
        &mut bolt_query
    {
        let new_pos = Vec2::new(breaker_x, breaker_y + config.spawn_offset_y);
        position.0 = new_pos;

        if let Some(mut prev) = prev_pos {
            prev.0 = new_pos;
        }

        if serving {
            velocity.0 = Vec2::ZERO;
            commands.entity(entity).insert(BoltServing);
        } else {
            let v = config.initial_velocity();
            velocity.0 = Vec2::new(v.x, v.y);
            commands.entity(entity).remove::<BoltServing>();
        }

        if let (Some(mut remaining), Some(ep)) = (piercing_remaining, effective_piercing) {
            remaining.0 = ep.0;
        }
    }
}
