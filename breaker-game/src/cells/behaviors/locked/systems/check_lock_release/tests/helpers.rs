//! Shared test helpers for `check_lock_release` tests.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::super::system::check_lock_release;
use crate::cells::{
    components::*,
    messages::{CellDestroyedAt, DamageCell},
    systems::handle_cell_hit,
};

// ---------------------------------------------------------------
// Test helpers -- message injection for CellDestroyedAt
// ---------------------------------------------------------------

#[derive(Resource, Default)]
pub(super) struct TestDestroyedMessages(pub(super) Vec<CellDestroyedAt>);

pub(super) fn enqueue_destroyed(
    msg_res: Res<TestDestroyedMessages>,
    mut writer: MessageWriter<CellDestroyedAt>,
) {
    for msg in &msg_res.0 {
        writer.write(msg.clone());
    }
}

// ---------------------------------------------------------------
// Test helpers -- message injection for DamageCell
// ---------------------------------------------------------------

#[derive(Resource)]
pub(super) struct TestDamageCellMessage(pub(super) Option<DamageCell>);

pub(super) fn enqueue_damage_cell(
    msg_res: Res<TestDamageCellMessage>,
    mut writer: MessageWriter<DamageCell>,
) {
    if let Some(msg) = msg_res.0.clone() {
        writer.write(msg);
    }
}

// ---------------------------------------------------------------
// Test app factories
// ---------------------------------------------------------------

/// App for testing `check_lock_release` (behaviors 2, 3, 5).
pub(super) fn lock_release_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<CellDestroyedAt>()
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
pub(super) fn hit_app() -> App {
    use crate::cells::messages::RequestCellDestroyed;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_message::<DamageCell>()
        .add_message::<RequestCellDestroyed>()
        .add_systems(
            FixedUpdate,
            (enqueue_damage_cell.before(handle_cell_hit), handle_cell_hit),
        );
    app
}

pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

pub(super) fn default_damage_visuals() -> CellDamageVisuals {
    CellDamageVisuals {
        hdr_base: 4.0,
        green_min: 0.2,
        blue_range: 0.4,
        blue_base: 0.2,
    }
}

/// Spawns a cell with `Locked` marker and visuals for the hit-system tests.
pub(super) fn spawn_locked_cell(app: &mut App, hp: f32) -> Entity {
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
            LockCell,
            Locked,
            CellHealth::new(hp),
            default_damage_visuals(),
            RequiredToClear,
            Position2D(Vec2::ZERO),
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id()
}

/// Spawns an unlocked cell with visuals for the hit-system tests.
pub(super) fn spawn_unlocked_cell(app: &mut App, hp: f32) -> Entity {
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
            Position2D(Vec2::ZERO),
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id()
}
