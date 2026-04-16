//! Shared test harness for salvo integration tests.
//!
//! Provides test app builders and entity spawn helpers for each system under test.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::ApplyVelocity;

use crate::{
    cells::{
        behaviors::survival::{
            components::{BoltImmune, SurvivalPattern, SurvivalTimer, SurvivalTurret},
            salvo::{
                components::{SALVO_HALF_EXTENT, Salvo, SalvoDamage, SalvoFireTimer, SalvoSource},
                systems::{
                    fire_survival_turret::fire_survival_turret,
                    salvo_bolt_collision::salvo_bolt_collision,
                    salvo_breaker_collision::salvo_breaker_collision,
                    salvo_cell_collision::salvo_cell_collision,
                    salvo_wall_collision::salvo_wall_collision,
                    tick_salvo_fire_timer::tick_salvo_fire_timer,
                    tick_survival_timer::tick_survival_timer,
                },
            },
        },
        definition::AttackPattern,
        messages::SalvoImpactBreaker,
        test_utils::spawn_cell_in_world,
    },
    prelude::*,
    shared::death_pipeline::sets::DeathPipelineSystems,
};

// ── Test app builders ─────────────────────────────────────────────────────

/// Builds a test app wired for `tick_survival_timer` system tests.
pub(super) fn build_tick_survival_timer_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_physics()
        .with_effects_pipeline()
        .build();
    attach_message_capture::<DamageDealt<Cell>>(&mut app);
    app.add_systems(
        FixedUpdate,
        tick_survival_timer
            .before(DeathPipelineSystems::ApplyDamage)
            .run_if(in_state(NodeState::Playing)),
    );
    app
}

/// Builds a test app wired for `tick_salvo_fire_timer` system tests.
pub(super) fn build_tick_salvo_fire_timer_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_physics()
        .with_effects_pipeline()
        .build();
    app.add_systems(
        FixedUpdate,
        tick_salvo_fire_timer.run_if(in_state(NodeState::Playing)),
    );
    app
}

/// Builds a test app wired for `fire_survival_turret` system tests.
pub(super) fn build_fire_survival_turret_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_physics()
        .with_effects_pipeline()
        .build();
    app.add_systems(
        FixedUpdate,
        fire_survival_turret.run_if(in_state(NodeState::Playing)),
    );
    app
}

/// Builds a test app wired for `salvo_cell_collision` system tests.
pub(super) fn build_salvo_cell_collision_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_physics()
        .with_effects_pipeline()
        .build();
    attach_message_capture::<DamageDealt<Cell>>(&mut app);
    app.add_systems(
        FixedUpdate,
        salvo_cell_collision
            .before(DeathPipelineSystems::ApplyDamage)
            .run_if(in_state(NodeState::Playing)),
    );
    app
}

/// Builds a test app wired for `salvo_bolt_collision` system tests.
pub(super) fn build_salvo_bolt_collision_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_physics()
        .with_effects_pipeline()
        .build();
    app.add_systems(
        FixedUpdate,
        salvo_bolt_collision.run_if(in_state(NodeState::Playing)),
    );
    app
}

/// Builds a test app wired for `salvo_breaker_collision` system tests.
pub(super) fn build_salvo_breaker_collision_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_physics()
        .with_effects_pipeline()
        .build();
    attach_message_capture::<SalvoImpactBreaker>(&mut app);
    app.add_systems(
        FixedUpdate,
        salvo_breaker_collision.run_if(in_state(NodeState::Playing)),
    );
    app
}

/// Builds a test app wired for `salvo_wall_collision` system tests.
pub(super) fn build_salvo_wall_collision_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_physics()
        .with_playfield()
        .with_effects_pipeline()
        .build();
    app.add_systems(
        FixedUpdate,
        salvo_wall_collision.run_if(in_state(NodeState::Playing)),
    );
    app
}

// ── Entity spawn helpers ──────────────────────────────────────────────────

/// Spawns a turret cell with manual component insertion (bypasses builder
/// so we can set `started` and `SalvoFireTimer` directly).
pub(super) fn spawn_turret_manual(
    app: &mut App,
    pos: Vec2,
    hp: f32,
    survival_timer: Option<SurvivalTimer>,
    fire_timer: f32,
    pattern: AttackPattern,
) -> Entity {
    let entity = spawn_cell_in_world(app.world_mut(), |commands| {
        Cell::builder()
            .position(pos)
            .dimensions(40.0, 20.0)
            .hp(hp)
            .headless()
            .spawn(commands)
    });
    let mut entity_mut = app.world_mut().entity_mut(entity);
    entity_mut.insert((
        SurvivalTurret,
        SurvivalPattern(pattern),
        SalvoFireTimer(fire_timer),
        BoltImmune,
    ));
    if let Some(timer) = survival_timer {
        entity_mut.insert(timer);
    }
    entity
}

/// Spawns a salvo entity at the given position with standard components.
pub(super) fn spawn_salvo(
    app: &mut App,
    pos: Vec2,
    vel: Vec2,
    damage: f32,
    source: Entity,
) -> Entity {
    app.world_mut()
        .spawn((
            Salvo,
            SalvoDamage(damage),
            SalvoSource(source),
            Position2D(pos),
            Velocity2D(vel),
            Scale2D {
                x: SALVO_HALF_EXTENT * 2.0,
                y: SALVO_HALF_EXTENT * 2.0,
            },
            Aabb2D::new(Vec2::ZERO, Vec2::splat(SALVO_HALF_EXTENT)),
            CollisionLayers::new(
                SALVO_LAYER,
                CELL_LAYER | BREAKER_LAYER | BOLT_LAYER | WALL_LAYER,
            ),
            ApplyVelocity,
            CleanupOnExit::<NodeState>::default(),
        ))
        .id()
}

/// Spawns a plain cell at the given position for collision tests.
pub(super) fn spawn_collision_cell(app: &mut App, pos: Vec2, half_extents: Vec2) -> Entity {
    spawn_cell_in_world(app.world_mut(), |commands| {
        Cell::builder()
            .position(pos)
            .dimensions(half_extents.x * 2.0, half_extents.y * 2.0)
            .hp(10.0)
            .headless()
            .spawn(commands)
    })
}

/// Spawns a bolt entity at the given position for collision tests.
pub(super) fn spawn_collision_bolt(app: &mut App, pos: Vec2, half_extents: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Bolt,
            Position2D(pos),
            Aabb2D::new(Vec2::ZERO, half_extents),
            Hp::new(1.0),
            Velocity2D(Vec2::new(100.0, 200.0)),
            CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
        ))
        .id()
}

/// Spawns a breaker entity at the given position for collision tests.
pub(super) fn spawn_collision_breaker(app: &mut App, pos: Vec2, half_extents: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Breaker,
            Position2D(pos),
            Aabb2D::new(Vec2::ZERO, half_extents),
        ))
        .id()
}

/// Walks `AppState::Game -> GameState::Run -> RunState::Node -> NodeState::Playing`.
pub(super) fn advance_to_playing(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Game);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Run);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<RunState>>()
        .set(RunState::Node);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();
}
