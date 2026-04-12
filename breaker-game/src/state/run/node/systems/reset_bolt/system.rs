//! System to reset the bolt's position and velocity at the start of each node.

use bevy::prelude::*;
use rand::Rng;

use crate::{
    bolt::{
        components::ExtraBolt,
        messages::BoltSpawned,
        queries::{ResetBoltData, apply_velocity_formula},
        resources::DEFAULT_BOLT_ANGLE_SPREAD,
    },
    prelude::*,
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
    mut bolt_spawned: MessageWriter<BoltSpawned>,
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
            apply_velocity_formula(
                &mut bolt.spatial,
                bolt.active_speed_boosts
                    .map_or(1.0, crate::effect_v3::stacking::EffectStack::aggregate),
            );

            commands.entity(bolt.entity).remove::<BoltServing>();
        }

        if let (Some(ref mut remaining), Some(ap)) =
            (bolt.piercing_remaining, bolt.active_piercings)
        {
            #[allow(
                clippy::cast_sign_loss,
                clippy::cast_possible_truncation,
                reason = "piercing aggregate is always non-negative small integer"
            )]
            {
                remaining.0 = ap.aggregate().round() as u32;
            }
        }

        bolt_spawned.write(BoltSpawned);
    }
}
