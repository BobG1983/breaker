use bevy::prelude::*;

pub(super) use crate::bolt::test_utils::spawn_bolt;
use crate::bolt::{
    components::PiercingRemaining, messages::BoltImpactWall, systems::bolt_wall_collision::*,
    test_utils::piercing_stack,
};

pub(super) fn test_app() -> App {
    use crate::shared::test_utils::TestAppBuilder;

    TestAppBuilder::new()
        .with_physics()
        .with_message::<BoltImpactWall>()
        .insert_resource(WallHitMessages::default())
        .with_system(
            FixedUpdate,
            bolt_wall_collision
                .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
        )
        .with_system(FixedUpdate, collect_wall_hits.after(bolt_wall_collision))
        .build()
}

pub(super) use crate::shared::test_utils::tick;

/// Spawns a bolt with `EffectStack<PiercingConfig>` and `PiercingRemaining` components.
pub(super) fn spawn_piercing_bolt(
    app: &mut App,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    active_piercings: Vec<u32>,
    piercing_remaining: u32,
) -> Entity {
    let entity = spawn_bolt(app, x, y, vx, vy);
    app.world_mut().entity_mut(entity).insert((
        piercing_stack(&active_piercings),
        PiercingRemaining(piercing_remaining),
    ));
    entity
}

pub(super) use crate::walls::test_utils::spawn_wall;

/// Collects `BoltImpactWall` messages into a resource for test assertions.
#[derive(Resource, Default)]
pub(super) struct WallHitMessages(pub(super) Vec<BoltImpactWall>);

pub(super) fn collect_wall_hits(
    mut reader: MessageReader<BoltImpactWall>,
    mut msgs: ResMut<WallHitMessages>,
) {
    for msg in reader.read() {
        msgs.0.push(msg.clone());
    }
}
