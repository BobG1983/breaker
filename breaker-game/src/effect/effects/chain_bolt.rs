use bevy::prelude::*;

/// Marker on a chain bolt entity, pointing to its anchor entity.
#[derive(Component)]
pub struct ChainBoltMarker(pub Entity);

/// Marker on an entity that serves as the anchor for a chain bolt.
#[derive(Component)]
pub struct ChainBoltAnchor;

/// Spawns a chain bolt tethered to the given entity.
///
/// Placeholder — needs bolt components and `DistanceConstraint` wiring.
pub(crate) fn fire(entity: Entity, _tether_distance: f32, world: &mut World) {
    let position = world
        .entity(entity)
        .get::<Transform>()
        .map_or(Vec3::ZERO, |t| t.translation);

    let chain_bolt_a = world
        .spawn((
            ChainBoltMarker(entity),
            Transform::from_translation(position), // Should not use transform, ever
        ))
        .id();

    let chain_bolt_b = world
        .spawn((
            ChainBoltMarker(entity),
            Transform::from_translation(position), // Should not use transform, ever
        ))
        .id();

    info!(
        "spawned chain bolts {:?} and {:?} anchored to {:?} at {:?}",
        chain_bolt_a, chain_bolt_b, entity, position
    );

    // TODO: add DistanceConstraint between chain_bolt_a and chain_bolt_b

    if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
        entity_mut.insert(ChainBoltAnchor);
    }
}

/// Despawns all chain bolts anchored to the given entity and removes
/// the `ChainBoltAnchor` marker.
pub(crate) fn reverse(entity: Entity, _tether_distance: f32, world: &mut World) {
    let chain_bolts: Vec<Entity> = world
        .query::<(Entity, &ChainBoltMarker)>()
        .iter(world)
        .filter(|(_, marker)| marker.0 == entity)
        .map(|(e, _)| e)
        .collect();

    for chain_bolt in chain_bolts {
        world.despawn(chain_bolt);
    }

    if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
        entity_mut.remove::<ChainBoltAnchor>();
    }
}

/// Registers systems for `ChainBolt` effect.
pub(crate) fn register(_app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_spawns_two_chain_bolts_and_marks_anchor() {
        let mut world = World::new();
        let anchor = world.spawn(Transform::from_translation(Vec3::ZERO)).id();

        fire(anchor, 50.0, &mut world);

        // Should spawn TWO chain bolt entities.
        let chain_bolts: Vec<(Entity, &ChainBoltMarker)> = world
            .query::<(Entity, &ChainBoltMarker)>()
            .iter(&world)
            .collect();
        assert_eq!(chain_bolts.len(), 2, "fire should spawn two chain bolts");

        // Both should reference the anchor.
        for (_, marker) in &chain_bolts {
            assert_eq!(marker.0, anchor);
        }

        // Anchor should have `ChainBoltAnchor`.
        assert!(
            world.get::<ChainBoltAnchor>(anchor).is_some(),
            "anchor should have ChainBoltAnchor component"
        );
    }

    #[test]
    fn reverse_despawns_chain_bolts_and_removes_anchor_marker() {
        let mut world = World::new();
        let anchor = world
            .spawn((Transform::from_translation(Vec3::ZERO), ChainBoltAnchor))
            .id();

        // Manually spawn two chain bolts referencing the anchor.
        world.spawn((
            ChainBoltMarker(anchor),
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        ));
        world.spawn((
            ChainBoltMarker(anchor),
            Transform::from_translation(Vec3::new(-10.0, 0.0, 0.0)),
        ));

        reverse(anchor, 50.0, &mut world);

        // All chain bolts should be despawned.
        let remaining: Vec<Entity> = world
            .query_filtered::<Entity, With<ChainBoltMarker>>()
            .iter(&world)
            .collect();
        assert!(remaining.is_empty(), "all chain bolts should be despawned");

        // Anchor marker should be removed.
        assert!(
            world.get::<ChainBoltAnchor>(anchor).is_none(),
            "ChainBoltAnchor should be removed from anchor"
        );
    }

    #[test]
    fn reverse_when_no_chain_bolts_is_noop() {
        let mut world = World::new();
        let anchor = world.spawn(Transform::from_translation(Vec3::ZERO)).id();

        // reverse with no chain bolts and no anchor marker should not panic.
        reverse(anchor, 50.0, &mut world);

        assert!(
            world.get::<ChainBoltAnchor>(anchor).is_none(),
            "anchor should remain without ChainBoltAnchor"
        );
    }
}
