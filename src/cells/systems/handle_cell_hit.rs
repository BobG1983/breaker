//! System to handle cell damage when hit by the bolt.

use bevy::prelude::*;

use crate::{
    cells::{
        components::{Cell, CellHealth},
        messages::CellDestroyed,
        resources::CellConfig,
    },
    physics::messages::BoltHitCell,
};

/// Handles cell damage in response to [`BoltHitCell`] messages.
///
/// Decrements cell health, updates visual feedback via material color,
/// and despawns cells that reach zero HP. Sends [`CellDestroyed`] on destruction.
pub fn handle_cell_hit(
    mut reader: MessageReader<BoltHitCell>,
    mut cell_query: Query<(&mut CellHealth, &MeshMaterial2d<ColorMaterial>), With<Cell>>,
    mut commands: Commands,
    mut destroyed_writer: MessageWriter<CellDestroyed>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    cell_config: Res<CellConfig>,
) {
    for hit in reader.read() {
        let Ok((mut health, material_handle)) = cell_query.get_mut(hit.cell) else {
            continue;
        };

        let destroyed = health.take_hit();

        if destroyed {
            commands.entity(hit.cell).despawn();
            destroyed_writer.write(CellDestroyed { entity: hit.cell });
        } else {
            // Visual feedback — dim HDR intensity based on remaining health
            let frac = health.fraction();
            let intensity = frac * cell_config.damage_hdr_base;
            if let Some(material) = materials.get_mut(material_handle.id()) {
                material.color = Color::srgb(
                    intensity,
                    cell_config.damage_green_min * frac,
                    cell_config
                        .damage_green_range
                        .mul_add(1.0 - frac, cell_config.damage_blue_base),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::components::{Cell, CellHealth};

    #[derive(Resource)]
    struct TestMessage(Option<BoltHitCell>);

    /// Helper system to queue a message from a test resource.
    fn enqueue_from_resource(msg_res: Res<TestMessage>, mut writer: MessageWriter<BoltHitCell>) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();
        app.init_resource::<CellConfig>();
        app.add_message::<BoltHitCell>();
        app.add_message::<CellDestroyed>();
        app.add_systems(Update, handle_cell_hit);
        app
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
                Mesh2d(mesh),
                MeshMaterial2d(material),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id()
    }

    #[test]
    fn standard_cell_destroyed_on_hit() {
        let mut app = test_app();
        let cell = spawn_cell(&mut app, 1);

        app.insert_resource(TestMessage(Some(BoltHitCell {
            bolt: Entity::PLACEHOLDER,
            cell,
        })));

        app.add_systems(Update, enqueue_from_resource.before(handle_cell_hit));
        app.update();

        assert!(
            app.world().get_entity(cell).is_err(),
            "standard cell should be despawned"
        );
    }

    #[test]
    fn tough_cell_survives_one_hit() {
        let mut app = test_app();
        let cell = spawn_cell(&mut app, 3);

        app.insert_resource(TestMessage(Some(BoltHitCell {
            bolt: Entity::PLACEHOLDER,
            cell,
        })));

        app.add_systems(Update, enqueue_from_resource.before(handle_cell_hit));
        app.update();

        assert!(
            app.world().get_entity(cell).is_ok(),
            "tough cell should survive one hit"
        );
        let health = app.world().get::<CellHealth>(cell).unwrap();
        assert_eq!(health.current, 2);
    }
}
