//! Attraction systems — apply attraction forces.

use bevy::prelude::*;

use super::components::ActiveAttractions;
use crate::{effect_v3::types::AttractionType, prelude::*};

/// Applies attraction forces from all entries to the bolt's velocity each frame.
pub fn apply_attraction_forces(
    mut bolt_query: Query<(&mut Velocity2D, &Position2D, &ActiveAttractions), With<Bolt>>,
    breaker_query: Query<&Position2D, With<Breaker>>,
    bolt_target_query: Query<&Position2D, With<Bolt>>,
    cell_query: Query<&Position2D, With<Cell>>,
    wall_query: Query<&Position2D, With<Wall>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut velocity, bolt_pos, attractions) in &mut bolt_query {
        let mut total_force = Vec2::ZERO;

        for entry in &attractions.0 {
            // Find nearest entity of the attraction type.
            let nearest_pos: Option<Vec2> = match entry.attraction_type {
                AttractionType::Breaker => nearest_position(&breaker_query, bolt_pos.0),
                AttractionType::Bolt => nearest_position(&bolt_target_query, bolt_pos.0),
                AttractionType::Cell => nearest_position(&cell_query, bolt_pos.0),
                AttractionType::Wall => nearest_position(&wall_query, bolt_pos.0),
            };

            if let Some(target_pos) = nearest_pos {
                let to_target = target_pos - bolt_pos.0;
                let dist = to_target.length();
                if dist > f32::EPSILON {
                    let direction = to_target / dist;
                    let mut force_magnitude = entry.force * dt;
                    if let Some(max) = entry.max_force {
                        force_magnitude = force_magnitude.min(max * dt);
                    }
                    total_force += direction * force_magnitude;
                }
            }
        }

        velocity.0 += total_force;
    }
}

/// Find the position of the nearest entity to `origin` from a query.
fn nearest_position<F: bevy::ecs::query::QueryFilter>(
    query: &Query<&Position2D, F>,
    origin: Vec2,
) -> Option<Vec2> {
    let mut best: Option<(Vec2, f32)> = None;
    for pos in query.iter() {
        let dist = origin.distance(pos.0);
        if best.is_none_or(|(_, best_dist)| dist < best_dist) {
            best = Some((pos.0, dist));
        }
    }
    best.map(|(p, _)| p)
}
