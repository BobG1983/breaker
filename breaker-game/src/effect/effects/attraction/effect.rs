use bevy::prelude::*;
use rantzsoft_physics2d::resources::CollisionQuadtree;
use rantzsoft_spatial2d::{components::GlobalPosition2D, queries::SpatialData};

use crate::{
    effect::core::AttractionType,
    prelude::*,
    shared::{BREAKER_LAYER, CELL_LAYER, WALL_LAYER},
};

/// Search radius for quadtree AABB query. Large enough to cover the entire
/// playfield so the nearest entity is always found regardless of distance.
const ATTRACTION_SEARCH_RADIUS: f32 = 500.0;

/// An individual attraction entry tracking type, force, and active state.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AttractionEntry {
    /// Which entity type to attract toward.
    pub attraction_type: AttractionType,
    /// Attraction strength.
    pub force: f32,
    /// Optional maximum force magnitude per tick.
    pub max_force: Option<f32>,
    /// Whether this attraction is currently active (deactivates on hit).
    pub active: bool,
}

/// Component holding all active attractions on an entity.
#[derive(Component, Debug, Default, Clone)]
pub(crate) struct ActiveAttractions(pub(crate) Vec<AttractionEntry>);

/// Adds an attraction entry to the entity.
///
/// Inserts `ActiveAttractions` if not already present.
pub(crate) fn fire(
    entity: Entity,
    attraction_type: AttractionType,
    force: f32,
    max_force: Option<f32>,
    _source_chip: &str,
    world: &mut World,
) {
    let entry = AttractionEntry {
        attraction_type,
        force,
        max_force,
        active: true,
    };

    if let Some(mut attractions) = world.get_mut::<ActiveAttractions>(entity) {
        attractions.0.push(entry);
    } else if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
        entity_mut.insert(ActiveAttractions(vec![entry]));
    }
}

/// Removes a matching attraction entry from the entity.
pub(crate) fn reverse(
    entity: Entity,
    attraction_type: AttractionType,
    force: f32,
    max_force: Option<f32>,
    _source_chip: &str,
    world: &mut World,
) {
    if let Some(mut attractions) = world.get_mut::<ActiveAttractions>(entity)
        && let Some(idx) = attractions.0.iter().position(|e| {
            e.attraction_type == attraction_type
                && (e.force - force).abs() < f32::EPSILON
                && e.max_force == max_force
        })
    {
        attractions.0.remove(idx);
    }
}

/// Steers entities with [`ActiveAttractions`] toward the nearest target of each
/// attracted type using the [`CollisionQuadtree`].
pub(crate) fn apply_attraction(
    time: Res<Time<Fixed>>,
    quadtree: Res<CollisionQuadtree>,
    positions: Query<&GlobalPosition2D>,
    mut attracted: Query<(SpatialData, &ActiveAttractions, Option<&ActiveSpeedBoosts>)>,
) {
    let dt = time.timestep().as_secs_f32();

    for (mut spatial, attractions, speed_boosts) in &mut attracted {
        let entity_pos = spatial.global_position.0;

        if !attractions.0.iter().any(|e| e.active) {
            continue;
        }

        // Track the nearest candidate across ALL active attraction types.
        let mut nearest_dist = f32::MAX;
        let mut nearest_pos = Vec2::ZERO;
        let mut nearest_force = 0.0_f32;
        let mut nearest_max_force: Option<f32> = None;

        for entry in attractions.0.iter().filter(|e| e.active) {
            let layer = match entry.attraction_type {
                AttractionType::Cell => CELL_LAYER,
                AttractionType::Wall => WALL_LAYER,
                AttractionType::Breaker => BREAKER_LAYER,
            };
            let search_region = Aabb2D::new(entity_pos, Vec2::splat(ATTRACTION_SEARCH_RADIUS));
            let layers = CollisionLayers::new(0, layer);
            let candidates = quadtree
                .quadtree
                .query_aabb_filtered(&search_region, layers);

            for candidate in candidates {
                if let Ok(candidate_pos) = positions.get(candidate) {
                    let dist = entity_pos.distance(candidate_pos.0);
                    if dist < nearest_dist {
                        nearest_dist = dist;
                        nearest_pos = candidate_pos.0;
                        nearest_force = entry.force;
                        nearest_max_force = entry.max_force;
                    }
                }
            }
        }

        if nearest_dist < f32::MAX {
            let direction = (nearest_pos - entity_pos).normalize_or_zero();
            let effective_force =
                nearest_max_force.map_or(nearest_force, |cap| nearest_force.min(cap));
            let steering = direction * effective_force * dt;
            spatial.velocity.0 = (spatial.velocity.0 + steering).normalize_or_zero();
            crate::bolt::queries::apply_velocity_formula(
                &mut spatial,
                speed_boosts.map_or(
                    1.0,
                    crate::effect::effects::speed_boost::ActiveSpeedBoosts::multiplier,
                ),
            );
        }
    }
}

/// Reads bolt impact messages and deactivates attraction entries on hit with an
/// attracted type, reactivates all deactivated entries on hit with a
/// non-attracted type.
pub(crate) fn manage_attraction_types(
    mut impact_cell: MessageReader<BoltImpactCell>,
    mut impact_wall: MessageReader<BoltImpactWall>,
    mut impact_breaker: MessageReader<BoltImpactBreaker>,
    mut attracted: Query<&mut ActiveAttractions>,
) {
    // Process cell impacts.
    for msg in impact_cell.read() {
        if let Ok(mut attractions) = attracted.get_mut(msg.bolt) {
            process_impact(&mut attractions, AttractionType::Cell);
        }
    }

    // Process wall impacts.
    for msg in impact_wall.read() {
        if let Ok(mut attractions) = attracted.get_mut(msg.bolt) {
            process_impact(&mut attractions, AttractionType::Wall);
        }
    }

    // Process breaker impacts.
    for msg in impact_breaker.read() {
        if let Ok(mut attractions) = attracted.get_mut(msg.bolt) {
            process_impact(&mut attractions, AttractionType::Breaker);
        }
    }
}

/// Handles an impact for a specific attraction type.
///
/// If the entity has an entry for the impact type, deactivate matching entries.
/// If it does NOT have an entry for the impact type, reactivate all entries.
fn process_impact(attractions: &mut ActiveAttractions, impact_type: AttractionType) {
    let has_type = attractions
        .0
        .iter()
        .any(|e| e.attraction_type == impact_type);

    if has_type {
        // Deactivate all entries of the matching type.
        for entry in &mut attractions.0 {
            if entry.attraction_type == impact_type {
                entry.active = false;
            }
        }
    } else {
        // Reactivate all deactivated entries.
        for entry in &mut attractions.0 {
            entry.active = true;
        }
    }
}

/// Registers attraction systems in `FixedUpdate`.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            apply_attraction.after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
            manage_attraction_types,
        )
            .run_if(in_state(NodeState::Playing)),
    );
}
