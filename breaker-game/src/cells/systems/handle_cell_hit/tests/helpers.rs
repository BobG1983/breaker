use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

// Re-export the system under test so sub-modules can reference it in ordering constraints.
pub(super) use super::super::system::handle_cell_hit;
use crate::{
    cells::{
        components::*,
        messages::{DamageCell, RequestCellDestroyed},
    },
    effect::effects::shield::ShieldActive,
};

#[derive(Resource)]
pub(super) struct TestMessage(pub(super) Option<DamageCell>);

#[derive(Resource, Default)]
pub(super) struct TestMessages(pub(super) Vec<DamageCell>);

#[derive(Resource, Default)]
pub(super) struct CapturedDestroyed(pub(super) Vec<RequestCellDestroyed>);

#[derive(Resource, Default)]
pub(super) struct CapturedRequestCellDestroyed(pub(super) Vec<RequestCellDestroyed>);

pub(super) fn enqueue_from_resource(
    msg_res: Res<TestMessage>,
    mut writer: MessageWriter<DamageCell>,
) {
    if let Some(msg) = msg_res.0.clone() {
        writer.write(msg);
    }
}

pub(super) fn enqueue_all(msg_res: Res<TestMessages>, mut writer: MessageWriter<DamageCell>) {
    for msg in &msg_res.0 {
        writer.write(msg.clone());
    }
}

pub(super) fn capture_destroyed(
    mut reader: MessageReader<RequestCellDestroyed>,
    mut captured: ResMut<CapturedDestroyed>,
) {
    for msg in reader.read() {
        captured.0.push(msg.clone());
    }
}

pub(super) fn capture_request_cell_destroyed(
    mut reader: MessageReader<RequestCellDestroyed>,
    mut captured: ResMut<CapturedRequestCellDestroyed>,
) {
    for msg in reader.read() {
        captured.0.push(msg.clone());
    }
}

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_message::<DamageCell>()
        .add_message::<RequestCellDestroyed>()
        .add_systems(FixedUpdate, handle_cell_hit);
    app
}

pub(super) fn test_app_two_phase() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_message::<DamageCell>()
        .add_message::<RequestCellDestroyed>()
        .add_systems(FixedUpdate, handle_cell_hit);
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

pub(super) fn spawn_cell(app: &mut App, hp: f32) -> Entity {
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

pub(super) fn spawn_optional_cell(app: &mut App, hp: f32, required: bool) -> Entity {
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
        Position2D(Vec2::ZERO),
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    if required {
        entity.insert(RequiredToClear);
    }
    entity.id()
}

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

pub(super) fn spawn_cell_at(app: &mut App, hp: f32, pos: Vec2, required: bool) -> Entity {
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
        Position2D(pos),
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_xyz(pos.x, pos.y, 0.0),
    ));
    if required {
        entity.insert(RequiredToClear);
    }
    entity.id()
}

pub(super) fn spawn_shielded_cell(app: &mut App, hp: f32) -> Entity {
    let entity = spawn_cell(app, hp);
    app.world_mut()
        .entity_mut(entity)
        .insert(ShieldActive { charges: 3 });
    entity
}
