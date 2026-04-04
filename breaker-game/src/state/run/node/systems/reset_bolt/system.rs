//! System to reset the bolt's position and velocity at the start of each node.

use bevy::prelude::*;
use rand::Rng;
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    bolt::{
        components::{Bolt, BoltServing, ExtraBolt},
        queries::{ResetBoltData, apply_velocity_formula},
        resources::DEFAULT_BOLT_ANGLE_SPREAD,
    },
    breaker::components::Breaker,
    shared::GameRng,
    state::run::NodeOutcome,
};

/// Resets the bolt's position above the breaker and adjusts velocity for the
/// current node.
///
/// On the first node (`NodeOutcome.node_index == 0`), the bolt spawns with zero
/// velocity and a [`BoltServing`] marker. On subsequent nodes it launches
/// immediately at base speed with a random angle within the bolt's
/// [`BoltAngleSpread`] component.
///
/// Effect components (e.g. `ActivePiercings`, `ActiveDamageBoosts`)
/// are NOT touched -- they persist across nodes. Only positional and velocity
/// state is reset. [`PiercingRemaining`] is reset to `ActivePiercings.total()`.
pub(crate) fn reset_bolt(
    mut commands: Commands,
    run_state: Res<NodeOutcome>,
    mut rng: ResMut<GameRng>,
    breaker_query: Query<&Position2D, (With<Breaker>, Without<Bolt>)>,
    mut bolt_query: Query<ResetBoltData, (With<Bolt>, Without<ExtraBolt>)>,
) {
    let Ok(breaker_pos) = breaker_query.single() else {
        return;
    };

    let breaker_x = breaker_pos.0.x;
    let breaker_y = breaker_pos.0.y;

    let serving = run_state.node_index == 0;

    for mut bolt in &mut bolt_query {
        let new_pos = Vec2::new(breaker_x, breaker_y + bolt.spawn_offset.0);
        bolt.spatial.position.0 = new_pos;

        if let Some(ref mut prev) = bolt.previous_position {
            prev.0 = new_pos;
        }

        if serving {
            bolt.spatial.velocity.0 = Vec2::ZERO;
            commands.entity(bolt.entity).insert(BoltServing);
        } else {
            let spread = bolt.angle_spread.map_or(DEFAULT_BOLT_ANGLE_SPREAD, |a| a.0);
            let angle = rng.0.random_range(-spread..=spread);
            bolt.spatial.velocity.0 = Vec2::new(angle.sin(), angle.cos());

            // Apply the canonical velocity formula after setting launch velocity
            apply_velocity_formula(&mut bolt.spatial, bolt.active_speed_boosts);

            commands.entity(bolt.entity).remove::<BoltServing>();
        }

        if let (Some(ref mut remaining), Some(ap)) =
            (bolt.piercing_remaining, bolt.active_piercings)
        {
            remaining.0 = ap.total();
        }
    }
}
