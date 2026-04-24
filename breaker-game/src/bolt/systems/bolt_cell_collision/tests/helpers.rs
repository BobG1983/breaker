use bevy::prelude::*;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

// Re-export constants used by test modules
pub(super) use crate::bolt::systems::bolt_cell_collision::system::MAX_BOUNCES;
pub(super) use crate::bolt::test_utils::{
    default_bolt_definition as test_bolt_definition, spawn_bolt,
};
use crate::{
    bolt::{BoltPlugin, systems::bolt_cell_collision::system::bolt_cell_collision},
    cells::{
        components::{CellHeight, CellWidth},
        test_utils as cell_test_utils,
    },
    prelude::*,
    shared::GameDrawLayer,
};

/// Real grid vertical spacing: `cell_height` (24) + padding (4) = 28
pub(super) const GRID_STEP_Y: f32 = 28.0;
/// Real grid horizontal spacing: `cell_width` (70) + padding (4) = 74
pub(super) const GRID_STEP_X: f32 = 74.0;

pub(super) fn test_app() -> App {
    TestAppBuilder::new()
        .with_physics()
        .with_message::<BoltImpactCell>()
        .with_message::<DamageDealt<Cell>>()
        .with_message::<BoltImpactWall>()
        .with_system(
            FixedUpdate,
            bolt_cell_collision
                .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
        )
        .build()
}

pub(super) fn default_cell_dims() -> (CellWidth, CellHeight) {
    cell_test_utils::default_cell_dims()
}

pub(super) use crate::prelude::tick;

/// Cell entities use `Position2D` as canonical position.
pub(super) fn spawn_cell(app: &mut App, x: f32, y: f32) -> Entity {
    cell_test_utils::spawn_cell(app, x, y)
}

/// Spawns a cell with explicit [`Hp`] for piercing lookahead tests.
pub(super) fn spawn_cell_with_health(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
    let (cw, ch) = default_cell_dims();
    let half_extents = Vec2::new(cw.half_width(), ch.half_height());
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Cell,
            cw,
            ch,
            Hp::new(hp),
            KilledBy { killer: None },
            Aabb2D::new(Vec2::ZERO, half_extents),
            CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            GameDrawLayer::Cell,
        ))
        .id()
}

pub(super) use crate::walls::test_utils::spawn_right_wall;

/// Spawns a cell with explicit `Aabb2D` `half_extents` that differ from the
/// legacy `CellWidth`/`CellHeight` dimensions. Used to test which source
/// the collision system reads for Minkowski expansion.
pub(super) fn spawn_cell_with_custom_aabb(
    app: &mut App,
    x: f32,
    y: f32,
    aabb_half_extents: Vec2,
) -> Entity {
    let (cw, ch) = default_cell_dims();
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Cell,
            cw,
            ch,
            Aabb2D::new(Vec2::ZERO, aabb_half_extents),
            CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            GameDrawLayer::Cell,
        ))
        .id()
}

/// Spawns a cell with explicit [`Hp`] and a [`VulnerableStack`] carrying one
/// persistent entry tagged `"test"`.
pub(super) fn spawn_vulnerable_cell(
    app: &mut App,
    x: f32,
    y: f32,
    hp: f32,
    vulnerability: f32,
) -> Entity {
    let (cw, ch) = default_cell_dims();
    let half_extents = Vec2::new(cw.half_width(), ch.half_height());
    let pos = Vec2::new(x, y);
    let mut vuln_stack = VulnerableStack::default();
    vuln_stack.add(SourceId::from("test"), vulnerability);
    app.world_mut()
        .spawn((
            Cell,
            cw,
            ch,
            Hp::new(hp),
            KilledBy { killer: None },
            vuln_stack,
            Aabb2D::new(Vec2::ZERO, half_extents),
            CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            GameDrawLayer::Cell,
        ))
        .id()
}

/// Collects `BoltImpactCell` messages into a resource for test assertions.
#[derive(Resource, Default)]
pub(super) struct HitCells(pub(super) Vec<Entity>);

pub(super) fn collect_cell_hits(
    mut reader: MessageReader<BoltImpactCell>,
    mut hits: ResMut<HitCells>,
) {
    for msg in reader.read() {
        hits.0.push(msg.cell);
    }
}

/// Collects full `BoltImpactCell` messages (including the bolt field) for assertion.
#[derive(Resource, Default)]
pub(super) struct FullHitMessages(pub(super) Vec<BoltImpactCell>);

pub(super) fn collect_full_hits(
    mut reader: MessageReader<BoltImpactCell>,
    mut hits: ResMut<FullHitMessages>,
) {
    for msg in reader.read() {
        hits.0.push(msg.clone());
    }
}

/// Collects [`DamageDealt<Cell>`] messages into a resource for test assertions.
#[derive(Resource, Default)]
pub(super) struct DamageDealtCellMessages(pub(super) Vec<DamageDealt<Cell>>);

pub(super) fn collect_damage_cells(
    mut reader: MessageReader<DamageDealt<Cell>>,
    mut msgs: ResMut<DamageDealtCellMessages>,
) {
    for msg in reader.read() {
        msgs.0.push(msg.clone());
    }
}

/// Collects [`BoltImpactWall`] messages into a resource for test assertions.
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

/// Creates a test app with `DamageDealt<Cell>` and `BoltImpactWall` message capture
/// in addition to the standard `BoltImpactCell`.
///
/// Harness A (emission-only): does NOT install `RantzDmgPlugin`, so the
/// captured `DamageDealt<Cell>.amount` is the RAW value `bolt_cell_collision`
/// emitted — no boost/vulnerability multiplication has been applied. Use this
/// helper when the test asserts on the producer's emission amount directly.
pub(super) fn test_app_with_damage_and_wall_messages() -> App {
    TestAppBuilder::new()
        .with_physics()
        .with_message::<BoltImpactCell>()
        .with_message::<DamageDealt<Cell>>()
        .with_message::<BoltImpactWall>()
        .insert_resource(DamageDealtCellMessages::default())
        .insert_resource(WallHitMessages::default())
        .insert_resource(FullHitMessages::default())
        .with_system(
            FixedUpdate,
            bolt_cell_collision
                .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
        )
        .with_system(
            FixedUpdate,
            (collect_damage_cells, collect_wall_hits, collect_full_hits).after(bolt_cell_collision),
        )
        .build()
}

/// Harness B (full pipeline): installs the production damage pipeline
/// (`RantzDmgPlugin` + `register_dmgable::<Cell>()` via
/// `with_effects_pipeline()`) and the real `BoltPlugin`, so
/// `bolt_cell_collision` is scheduled with its production ordering
/// (`BoltSystems::CellCollision.before(EffectV3Systems::Bridge)`).
/// `EffectV3Plugin` configures `EffectV3Systems::Tick.before(DmgSystems::EmitDamage)`
/// which transitively orders the bolt producer before the crate pipeline.
///
/// Message collectors are scheduled `.after(DmgSystems::ApplyVulnerable).before(DmgSystems::ApplyDamage)`
/// so they observe `DamageDealt<Cell>.amount` AFTER `apply_damage_boosts::<Cell>`
/// and `apply_vulnerable::<Cell>` have multiplied the message, but BEFORE
/// `apply_damage::<Cell>` consumes the message. Use this helper for tests
/// that assert post-pipeline `amount` OR final `Hp.current` on the target
/// cell after one `tick(...)`.
///
/// Does NOT add any test-only scheduling tag on `bolt_cell_collision` — the
/// production wiring is preserved verbatim.
pub(super) fn test_app_with_full_pipeline() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_physics()
        .with_playfield()
        .with_bolt_registry()
        .with_breaker_registry()
        .with_cell_registry()
        .with_resource::<crate::input::resources::InputActions>()
        .with_effects_pipeline()
        .build();
    // BoltPlugin owns `bolt_cell_collision`'s production scheduling. Install
    // it UNMODIFIED — no DmgSystems::EmitDamage tag is added here.
    app.add_plugins(BoltPlugin);

    app.insert_resource(DamageDealtCellMessages::default());
    app.insert_resource(WallHitMessages::default());
    app.insert_resource(FullHitMessages::default());
    app.add_systems(
        FixedUpdate,
        (collect_damage_cells, collect_wall_hits, collect_full_hits)
            .after(DmgSystems::ApplyVulnerable)
            .before(DmgSystems::ApplyDamage),
    );
    app
}
