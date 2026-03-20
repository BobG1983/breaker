//! System to release locked cells when all adjacent cells are destroyed.

use bevy::{ecs::entity::Entities, prelude::*};

use crate::cells::{
    components::{LockAdjacents, Locked},
    messages::CellDestroyed,
};

/// Removes [`Locked`] marker when all adjacent cells are destroyed.
///
/// Listens for [`CellDestroyed`] messages and checks each locked cell's
/// [`LockAdjacents`] list. If every entity in the adjacents list has been
/// destroyed (no longer exists in the world), the [`Locked`] component is
/// removed, allowing the cell to take damage.
pub(crate) fn check_lock_release(
    mut reader: MessageReader<CellDestroyed>,
    query: Query<(Entity, &LockAdjacents), With<Locked>>,
    mut commands: Commands,
    all_entities: &Entities,
) {
    // Drain destroyed messages and count them so they don't accumulate.
    let destroyed_count = reader.read().count();

    for (entity, adjacents) in &query {
        if adjacents.0.is_empty() {
            // Empty adjacents list means the cell should always unlock.
            commands.entity(entity).remove::<Locked>();
        } else if destroyed_count > 0 {
            // Only scan entity existence when something was actually destroyed.
            let all_gone = adjacents.0.iter().all(|adj| !all_entities.contains(*adj));
            if all_gone {
                commands.entity(entity).remove::<Locked>();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cells::components::*, physics::messages::BoltHitCell};

    // ---------------------------------------------------------------
    // Test helpers — message injection for CellDestroyed
    // ---------------------------------------------------------------

    #[derive(Resource, Default)]
    struct TestDestroyedMessages(Vec<CellDestroyed>);

    fn enqueue_destroyed(
        msg_res: Res<TestDestroyedMessages>,
        mut writer: MessageWriter<CellDestroyed>,
    ) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    // ---------------------------------------------------------------
    // Test helpers — message injection for BoltHitCell
    // ---------------------------------------------------------------

    #[derive(Resource)]
    struct TestBoltHitMessage(Option<BoltHitCell>);

    fn enqueue_bolt_hit(msg_res: Res<TestBoltHitMessage>, mut writer: MessageWriter<BoltHitCell>) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    // ---------------------------------------------------------------
    // Test app factories
    // ---------------------------------------------------------------

    /// App for testing `check_lock_release` (behaviors 2, 3, 5).
    fn lock_release_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<CellDestroyed>()
            .init_resource::<TestDestroyedMessages>()
            .add_systems(
                FixedUpdate,
                (
                    enqueue_destroyed.before(check_lock_release),
                    check_lock_release,
                ),
            );
        app
    }

    /// App for testing `handle_cell_hit` with lock interaction (behaviors 1, 4).
    fn hit_app() -> App {
        use crate::cells::systems::handle_cell_hit;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .add_message::<BoltHitCell>()
            .add_message::<CellDestroyed>()
            .add_systems(
                FixedUpdate,
                (enqueue_bolt_hit.before(handle_cell_hit), handle_cell_hit),
            );
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn default_damage_visuals() -> CellDamageVisuals {
        CellDamageVisuals {
            hdr_base: 4.0,
            green_min: 0.2,
            blue_range: 0.4,
            blue_base: 0.2,
        }
    }

    /// Spawns a cell with `Locked` marker and visuals for the hit-system tests.
    fn spawn_locked_cell(app: &mut App, hp: f32) -> Entity {
        let material = app
            .world_mut()
            .resource_mut::<Assets<ColorMaterial>>()
            .add(ColorMaterial::from_color(Color::srgb(4.0, 0.2, 0.5)));
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(Rectangle::new(1.0, 1.0));
        app.world_mut()
            .spawn((
                Cell,
                Locked,
                CellHealth::new(hp),
                default_damage_visuals(),
                RequiredToClear,
                Mesh2d(mesh),
                MeshMaterial2d(material),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id()
    }

    /// Spawns an unlocked cell with visuals for the hit-system tests.
    fn spawn_unlocked_cell(app: &mut App, hp: f32) -> Entity {
        let material = app
            .world_mut()
            .resource_mut::<Assets<ColorMaterial>>()
            .add(ColorMaterial::from_color(Color::srgb(4.0, 0.2, 0.5)));
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(Rectangle::new(1.0, 1.0));
        app.world_mut()
            .spawn((
                Cell,
                CellHealth::new(hp),
                default_damage_visuals(),
                RequiredToClear,
                Mesh2d(mesh),
                MeshMaterial2d(material),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id()
    }

    // ---------------------------------------------------------------
    // Behavior 1: Locked cell immune to bolt damage
    // ---------------------------------------------------------------

    #[test]
    fn locked_cell_hp_unchanged_after_bolt_hit() {
        let mut app = hit_app();
        let cell = spawn_locked_cell(&mut app, 10.0);

        app.insert_resource(TestBoltHitMessage(Some(BoltHitCell {
            cell,
            bolt: Entity::PLACEHOLDER,
        })));
        tick(&mut app);

        // Locked cell should still exist (not destroyed)
        assert!(
            app.world().get_entity(cell).is_ok(),
            "locked cell should not be despawned by bolt hit"
        );
        // HP should be untouched
        let health = app.world().get::<CellHealth>(cell).unwrap();
        assert!(
            (health.current - 10.0).abs() < f32::EPSILON,
            "locked cell HP should remain 10.0, got {}",
            health.current
        );
    }

    // ---------------------------------------------------------------
    // Behavior 2: Lock releases when all adjacents destroyed
    // ---------------------------------------------------------------

    #[test]
    fn lock_releases_when_all_adjacents_destroyed() {
        let mut app = lock_release_app();

        // Spawn two adjacent cell entities (they just need to exist, then be destroyed).
        let adj_a = app.world_mut().spawn_empty().id();
        let adj_b = app.world_mut().spawn_empty().id();

        // Spawn the lock cell with Locked + LockAdjacents pointing at the two adjacents.
        let lock_cell = app
            .world_mut()
            .spawn((
                Cell,
                Locked,
                LockAdjacents(vec![adj_a, adj_b]),
                CellHealth::new(10.0),
            ))
            .id();

        // Despawn the adjacent entities (simulating their destruction).
        app.world_mut().despawn(adj_a);
        app.world_mut().despawn(adj_b);

        // Send CellDestroyed messages for both adjacents.
        app.world_mut().resource_mut::<TestDestroyedMessages>().0 = vec![
            CellDestroyed {
                was_required_to_clear: true,
            },
            CellDestroyed {
                was_required_to_clear: true,
            },
        ];

        tick(&mut app);

        // Locked component should be removed from the lock cell.
        assert!(
            app.world().get::<Locked>(lock_cell).is_none(),
            "Locked should be removed when all adjacents are destroyed"
        );
    }

    // ---------------------------------------------------------------
    // Behavior 3: Lock stays locked when only some adjacents destroyed
    // ---------------------------------------------------------------

    #[test]
    fn lock_stays_locked_when_only_some_adjacents_destroyed() {
        let mut app = lock_release_app();

        // Two adjacent cells; only one will be destroyed.
        let adj_a = app.world_mut().spawn_empty().id();
        let adj_b = app.world_mut().spawn_empty().id();

        let lock_cell = app
            .world_mut()
            .spawn((
                Cell,
                Locked,
                LockAdjacents(vec![adj_a, adj_b]),
                CellHealth::new(10.0),
            ))
            .id();

        // Despawn only adj_a.
        app.world_mut().despawn(adj_a);

        // Send CellDestroyed only for adj_a.
        app.world_mut().resource_mut::<TestDestroyedMessages>().0 = vec![CellDestroyed {
            was_required_to_clear: true,
        }];

        tick(&mut app);

        // adj_b still alive => Locked should remain.
        assert!(
            app.world().get::<Locked>(lock_cell).is_some(),
            "Locked should remain when adj_b is still alive"
        );
    }

    // ---------------------------------------------------------------
    // Behavior 4: Unlocked cell takes normal damage
    // ---------------------------------------------------------------

    #[test]
    fn unlocked_cell_takes_normal_damage_and_is_destroyed() {
        let mut app = hit_app();
        let cell = spawn_unlocked_cell(&mut app, 10.0);

        app.insert_resource(TestBoltHitMessage(Some(BoltHitCell {
            cell,
            bolt: Entity::PLACEHOLDER,
        })));
        tick(&mut app);

        // 10.0 HP cell hit with BASE_BOLT_DAMAGE (10.0) should be destroyed.
        assert!(
            app.world().get_entity(cell).is_err(),
            "unlocked 10-HP cell should be destroyed by 10 base damage"
        );
    }

    // ---------------------------------------------------------------
    // Behavior 5: Lock cell with empty adjacents unlocks immediately
    // ---------------------------------------------------------------

    #[test]
    fn lock_cell_with_empty_adjacents_unlocks_immediately() {
        let mut app = lock_release_app();

        // Lock cell with empty adjacents list — edge case.
        let lock_cell = app
            .world_mut()
            .spawn((Cell, Locked, LockAdjacents(vec![]), CellHealth::new(10.0)))
            .id();

        // No CellDestroyed messages needed — the adjacents list is empty.
        tick(&mut app);

        // Empty adjacents vec => all adjacents are "destroyed" => Locked removed.
        assert!(
            app.world().get::<Locked>(lock_cell).is_none(),
            "Locked should be removed immediately when adjacents list is empty"
        );
    }
}
