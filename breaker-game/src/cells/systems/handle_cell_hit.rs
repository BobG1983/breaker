//! System to handle cell damage when hit by the bolt.

use bevy::prelude::*;

use crate::{
    cells::{components::Cell, messages::CellDestroyed, queries::CellDamageVisualQuery},
    physics::messages::BoltHitCell,
};

/// Handles cell damage in response to [`BoltHitCell`] messages.
///
/// Decrements cell health, updates visual feedback via material color,
/// and despawns cells that reach zero HP. Sends [`CellDestroyed`] on destruction.
///
/// Guards against the same cell appearing in multiple messages in one frame
/// (e.g., two bolts hitting the same cell simultaneously): only the first hit
/// that destroys the cell is processed; subsequent messages for an already-despawned
/// cell are skipped to prevent duplicate [`CellDestroyed`] messages.
pub(crate) fn handle_cell_hit(
    mut reader: MessageReader<BoltHitCell>,
    mut cell_query: Query<CellDamageVisualQuery, With<Cell>>,
    mut commands: Commands,
    mut destroyed_writer: MessageWriter<CellDestroyed>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Small vec suffices — MAX_BOUNCES = 4 limits hits per frame
    let mut despawned = Vec::<Entity>::new();
    for hit in reader.read() {
        if despawned.contains(&hit.cell) {
            continue;
        }
        let Ok((mut health, material_handle, visuals, is_required)) = cell_query.get_mut(hit.cell)
        else {
            continue;
        };

        let destroyed = health.take_hit();

        if destroyed {
            commands.entity(hit.cell).despawn();
            destroyed_writer.write(CellDestroyed {
                entity: hit.cell,
                was_required_to_clear: is_required,
            });
            despawned.push(hit.cell);
        } else {
            // Visual feedback — dim HDR intensity based on remaining health
            let frac = health.fraction();
            let intensity = frac * visuals.hdr_base;
            if let Some(material) = materials.get_mut(material_handle.id()) {
                material.color = Color::srgb(
                    intensity,
                    visuals.green_min * frac,
                    visuals.blue_range.mul_add(1.0 - frac, visuals.blue_base),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::components::*;

    #[derive(Resource)]
    struct TestMessage(Option<BoltHitCell>);

    #[derive(Resource, Default)]
    struct TestMessages(Vec<BoltHitCell>);

    #[derive(Resource, Default)]
    struct CapturedDestroyed(Vec<CellDestroyed>);

    fn enqueue_from_resource(msg_res: Res<TestMessage>, mut writer: MessageWriter<BoltHitCell>) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    fn enqueue_all(msg_res: Res<TestMessages>, mut writer: MessageWriter<BoltHitCell>) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    fn capture_destroyed(
        mut reader: MessageReader<CellDestroyed>,
        mut captured: ResMut<CapturedDestroyed>,
    ) {
        for msg in reader.read() {
            captured.0.push(msg.clone());
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();
        app.add_message::<BoltHitCell>();
        app.add_message::<CellDestroyed>();
        app.add_systems(FixedUpdate, handle_cell_hit);
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

    fn spawn_cell(app: &mut App, hp: u32) -> Entity {
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

    fn spawn_optional_cell(app: &mut App, hp: u32, required: bool) -> Entity {
        let material = app
            .world_mut()
            .resource_mut::<Assets<ColorMaterial>>()
            .add(ColorMaterial::from_color(Color::srgb(4.0, 0.2, 0.5)));
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(Rectangle::new(1.0, 1.0));
        let mut entity = app.world_mut().spawn((
            Cell,
            CellHealth::new(hp),
            default_damage_visuals(),
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
        if required {
            entity.insert(RequiredToClear);
        }
        entity.id()
    }

    #[test]
    fn standard_cell_destroyed_on_hit() {
        let mut app = test_app();
        let cell = spawn_cell(&mut app, 1);

        app.insert_resource(TestMessage(Some(BoltHitCell { cell })));

        app.add_systems(FixedUpdate, enqueue_from_resource.before(handle_cell_hit));
        tick(&mut app);

        assert!(
            app.world().get_entity(cell).is_err(),
            "standard cell should be despawned"
        );
    }

    #[test]
    fn tough_cell_survives_one_hit() {
        let mut app = test_app();
        let cell = spawn_cell(&mut app, 3);

        app.insert_resource(TestMessage(Some(BoltHitCell { cell })));

        app.add_systems(FixedUpdate, enqueue_from_resource.before(handle_cell_hit));
        tick(&mut app);

        assert!(
            app.world().get_entity(cell).is_ok(),
            "tough cell should survive one hit"
        );
        let health = app.world().get::<CellHealth>(cell).unwrap();
        assert_eq!(health.current, 2);
    }

    #[test]
    fn destroyed_message_includes_required_to_clear() {
        let mut app = test_app();
        let cell = spawn_optional_cell(&mut app, 1, true);

        app.init_resource::<CapturedDestroyed>();
        app.insert_resource(TestMessage(Some(BoltHitCell { cell })));
        app.add_systems(
            FixedUpdate,
            (
                enqueue_from_resource.before(handle_cell_hit),
                capture_destroyed.after(handle_cell_hit),
            ),
        );
        tick(&mut app);

        let captured = app.world().resource::<CapturedDestroyed>();
        assert_eq!(
            captured.0.len(),
            1,
            "exactly one CellDestroyed should be sent"
        );
        assert!(
            captured.0[0].was_required_to_clear,
            "RequiredToClear cell should set was_required_to_clear = true"
        );
    }

    #[test]
    fn destroyed_message_false_for_non_required_cell() {
        let mut app = test_app();
        let cell = spawn_optional_cell(&mut app, 1, false);

        app.init_resource::<CapturedDestroyed>();
        app.insert_resource(TestMessage(Some(BoltHitCell { cell })));
        app.add_systems(
            FixedUpdate,
            (
                enqueue_from_resource.before(handle_cell_hit),
                capture_destroyed.after(handle_cell_hit),
            ),
        );
        tick(&mut app);

        let captured = app.world().resource::<CapturedDestroyed>();
        assert_eq!(
            captured.0.len(),
            1,
            "exactly one CellDestroyed should be sent"
        );
        assert!(
            !captured.0[0].was_required_to_clear,
            "non-required cell should set was_required_to_clear = false"
        );
    }

    #[test]
    fn double_hit_same_cell_only_destroys_once() {
        let mut app = test_app();
        let cell = spawn_optional_cell(&mut app, 1, true);

        app.init_resource::<CapturedDestroyed>();
        app.init_resource::<TestMessages>();
        app.world_mut().resource_mut::<TestMessages>().0 =
            vec![BoltHitCell { cell }, BoltHitCell { cell }];
        app.add_systems(
            FixedUpdate,
            (
                enqueue_all.before(handle_cell_hit),
                capture_destroyed.after(handle_cell_hit),
            ),
        );
        tick(&mut app);

        let captured = app.world().resource::<CapturedDestroyed>();
        assert_eq!(
            captured.0.len(),
            1,
            "two hits on the same 1-HP cell should produce exactly one CellDestroyed"
        );
    }
}
