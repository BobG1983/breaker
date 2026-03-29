use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::{
    collision_layers::CollisionLayers, plugin::PhysicsSystems, resources::CollisionQuadtree,
};

use crate::{
    bolt::BASE_BOLT_DAMAGE,
    cells::messages::DamageCell,
    shared::{CELL_LAYER, CleanupOnNodeExit, playing_state::PlayingState},
};

/// Emitter component attached to a bolt entity. Drives periodic ring emission.
#[derive(Component)]
pub struct PulseEmitter {
    /// Base range of emitted rings.
    pub base_range: f32,
    /// Additional range per stack level.
    pub range_per_level: f32,
    /// Current stack count.
    pub stacks: u32,
    /// Expansion speed of emitted rings in world units per second.
    pub speed: f32,
    /// Seconds between ring emissions.
    pub interval: f32,
    /// Accumulated time since last emission.
    pub timer: f32,
}

impl PulseEmitter {
    /// Effective maximum radius: `base_range + (stacks - 1) * range_per_level`.
    #[must_use]
    pub fn effective_max_radius(&self) -> f32 {
        let extra = u16::try_from(self.stacks.saturating_sub(1)).unwrap_or(u16::MAX);
        self.base_range + f32::from(extra) * self.range_per_level
    }
}

/// Marker component on pulse ring entities.
#[derive(Component)]
pub struct PulseRing;

/// The entity that spawned this ring (typically a bolt).
#[derive(Component)]
pub struct PulseSource(pub Entity);

/// Current expanding radius of the ring.
#[derive(Component)]
pub struct PulseRadius(pub f32);

/// Maximum radius before the ring despawns.
#[derive(Component)]
pub struct PulseMaxRadius(pub f32);

/// Expansion speed in world units per second.
#[derive(Component)]
pub struct PulseSpeed(pub f32);

/// Tracks which cells have been damaged by this specific ring.
#[derive(Component, Default)]
pub struct PulseDamaged(pub HashSet<Entity>);

pub(crate) fn fire(
    entity: Entity,
    base_range: f32,
    range_per_level: f32,
    stacks: u32,
    speed: f32,
    world: &mut World,
) {
    let emitter = PulseEmitter {
        base_range,
        range_per_level,
        stacks,
        speed,
        interval: 0.5,
        timer: 0.0,
    };
    if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
        entity_mut.insert(emitter);
    }
}

pub(crate) fn reverse(entity: Entity, world: &mut World) {
    if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
        entity_mut.remove::<PulseEmitter>();
    }
}

/// Tick emitter timers and spawn pulse rings when interval elapses.
///
/// Uses manual `f32` timer accumulation. When the timer reaches the interval,
/// spawns a [`PulseRing`] entity at the emitter's current position.
fn tick_pulse_emitter(
    time: Res<Time<Fixed>>,
    mut commands: Commands,
    mut emitters: Query<(Entity, &mut PulseEmitter, &Transform)>,
) {
    let dt = time.timestep().as_secs_f32();
    for (entity, mut emitter, transform) in &mut emitters {
        emitter.timer += dt;
        if emitter.timer >= emitter.interval {
            emitter.timer -= emitter.interval;
            let effective_range = emitter.effective_max_radius();
            let speed = emitter.speed;
            commands.spawn((
                PulseRing,
                PulseSource(entity),
                PulseRadius(0.0),
                PulseMaxRadius(effective_range),
                PulseSpeed(speed),
                PulseDamaged(HashSet::new()),
                Transform::from_translation(transform.translation),
                CleanupOnNodeExit,
            ));
        }
    }
}

/// Expand pulse ring radius by speed * dt each tick.
fn tick_pulse_ring(
    time: Res<Time>,
    mut rings: Query<(&mut PulseRadius, &PulseSpeed), With<PulseRing>>,
) {
    let dt = time.delta_secs();
    for (mut radius, speed) in &mut rings {
        radius.0 += speed.0 * dt;
    }
}

/// Damage cells within each pulse ring radius.
///
/// For each ring, queries the quadtree for cells within the current radius
/// and sends [`DamageCell`] for any cell not already in the [`PulseDamaged`] set.
fn apply_pulse_damage(
    quadtree: Res<CollisionQuadtree>,
    mut rings: Query<(&Transform, &PulseRadius, &mut PulseDamaged), With<PulseRing>>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let query_layers = CollisionLayers::new(0, CELL_LAYER);
    for (transform, radius, mut damaged) in &mut rings {
        if radius.0 <= 0.0 {
            continue;
        }
        let center = transform.translation.truncate();
        let candidates = quadtree
            .quadtree
            .query_circle_filtered(center, radius.0, query_layers);
        for cell in candidates {
            if damaged.0.insert(cell) {
                damage_writer.write(DamageCell {
                    cell,
                    damage: BASE_BOLT_DAMAGE,
                    source_chip: None,
                });
            }
        }
    }
}

/// Despawn pulse rings that have reached their maximum radius.
fn despawn_finished_pulse_ring(
    mut commands: Commands,
    rings: Query<(Entity, &PulseRadius, &PulseMaxRadius), With<PulseRing>>,
) {
    for (entity, radius, max_radius) in &rings {
        if radius.0 >= max_radius.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            tick_pulse_emitter,
            tick_pulse_ring,
            apply_pulse_damage,
            despawn_finished_pulse_ring,
        )
            .chain()
            .after(PhysicsSystems::MaintainQuadtree)
            .run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use rantzsoft_physics2d::{
        aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
    };
    use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D};

    use super::*;
    use crate::{
        bolt::BASE_BOLT_DAMAGE,
        cells::{components::Cell, messages::DamageCell},
        shared::{BOLT_LAYER, CELL_LAYER, WALL_LAYER},
    };

    // ── Behavior 8: fire() adds PulseEmitter to the target entity ──

    #[test]
    fn fire_adds_pulse_emitter_to_entity() {
        let mut world = World::new();
        let entity = world.spawn(Transform::from_xyz(100.0, 200.0, 0.0)).id();

        fire(entity, 32.0, 8.0, 1, 50.0, &mut world);

        let emitter = world
            .get::<PulseEmitter>(entity)
            .expect("entity should have PulseEmitter after fire()");

        assert!(
            (emitter.base_range - 32.0).abs() < f32::EPSILON,
            "expected base_range 32.0, got {}",
            emitter.base_range
        );
        assert!(
            (emitter.range_per_level - 8.0).abs() < f32::EPSILON,
            "expected range_per_level 8.0, got {}",
            emitter.range_per_level
        );
        assert_eq!(emitter.stacks, 1, "expected stacks 1");
        assert!(
            (emitter.speed - 50.0).abs() < f32::EPSILON,
            "expected speed 50.0, got {}",
            emitter.speed
        );
        assert!(
            (emitter.interval - 0.5).abs() < f32::EPSILON,
            "expected interval 0.5 (default), got {}",
            emitter.interval
        );
        assert!(
            (emitter.timer - 0.0).abs() < f32::EPSILON,
            "expected timer 0.0, got {}",
            emitter.timer
        );
    }

    #[test]
    fn fire_overwrites_existing_pulse_emitter() {
        let mut world = World::new();
        let entity = world.spawn(Transform::from_xyz(100.0, 200.0, 0.0)).id();

        fire(entity, 32.0, 8.0, 1, 50.0, &mut world);
        fire(entity, 64.0, 16.0, 2, 100.0, &mut world);

        let emitter = world
            .get::<PulseEmitter>(entity)
            .expect("entity should have PulseEmitter after second fire()");

        assert!(
            (emitter.base_range - 64.0).abs() < f32::EPSILON,
            "expected overwritten base_range 64.0, got {}",
            emitter.base_range
        );
        assert_eq!(emitter.stacks, 2, "expected overwritten stacks 2");
    }

    // ── Behavior 9: effective_range scales with stacks ──

    #[test]
    fn effective_max_radius_scales_with_stacks() {
        let emitter = PulseEmitter {
            base_range: 32.0,
            range_per_level: 8.0,
            stacks: 3,
            speed: 50.0,
            interval: 0.5,
            timer: 0.0,
        };

        // 32.0 + (3-1)*8.0 = 48.0
        let radius = emitter.effective_max_radius();
        assert!(
            (radius - 48.0).abs() < f32::EPSILON,
            "expected effective_max_radius 48.0, got {radius}"
        );
    }

    #[test]
    fn effective_max_radius_with_zero_stacks_uses_base_range() {
        let emitter = PulseEmitter {
            base_range: 32.0,
            range_per_level: 8.0,
            stacks: 0,
            speed: 50.0,
            interval: 0.5,
            timer: 0.0,
        };

        // saturating_sub(1) on 0 = 0, so effective = base_range = 32.0
        let radius = emitter.effective_max_radius();
        assert!(
            (radius - 32.0).abs() < f32::EPSILON,
            "expected effective_max_radius 32.0 for stacks=0, got {radius}"
        );
    }

    // ── Behavior 10: reverse() removes PulseEmitter ──

    #[test]
    fn reverse_removes_pulse_emitter() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Transform::from_xyz(100.0, 200.0, 0.0),
                PulseEmitter {
                    base_range: 32.0,
                    range_per_level: 8.0,
                    stacks: 1,
                    speed: 50.0,
                    interval: 0.5,
                    timer: 0.0,
                },
            ))
            .id();

        reverse(entity, &mut world);

        assert!(
            world.get::<PulseEmitter>(entity).is_none(),
            "PulseEmitter should be removed after reverse()"
        );
    }

    #[test]
    fn reverse_on_entity_without_emitter_does_not_panic() {
        let mut world = World::new();
        let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

        // Should not panic
        reverse(entity, &mut world);

        assert!(
            world.get_entity(entity).is_ok(),
            "entity should still exist after no-op reverse"
        );
    }

    // ── system tests ────────────────────────────────────────────────

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<crate::shared::game_state::GameState>();
        app.add_sub_state::<crate::shared::playing_state::PlayingState>();
        app.add_systems(Update, tick_pulse_emitter);
        app.add_systems(Update, tick_pulse_ring);
        app.add_systems(Update, despawn_finished_pulse_ring);
        app
    }

    fn enter_playing(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<crate::shared::game_state::GameState>>()
            .set(crate::shared::game_state::GameState::Playing);
        app.update();
    }

    /// Collects [`DamageCell`] messages into a resource for test assertions.
    #[derive(Resource, Default)]
    struct DamageCellCollector(Vec<DamageCell>);

    fn collect_damage_cells(
        mut reader: MessageReader<DamageCell>,
        mut collector: ResMut<DamageCellCollector>,
    ) {
        for msg in reader.read() {
            collector.0.push(msg.clone());
        }
    }

    fn damage_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzPhysics2dPlugin);
        app.add_message::<DamageCell>();
        app.insert_resource(DamageCellCollector::default());
        app.add_systems(Update, apply_pulse_damage);
        app.add_systems(Update, collect_damage_cells.after(apply_pulse_damage));
        app
    }

    /// Accumulates one fixed timestep then runs one update (ensures quadtree maintenance runs).
    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn spawn_test_cell(app: &mut App, x: f32, y: f32) -> Entity {
        let pos = Vec2::new(x, y);
        app.world_mut()
            .spawn((
                Cell,
                Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
                CollisionLayers::new(CELL_LAYER, 0),
                Position2D(pos),
                GlobalPosition2D(pos),
                Spatial2D,
            ))
            .id()
    }

    // ── Behavior 11: tick_pulse_emitter spawns PulseRing when timer fires ──

    #[test]
    fn tick_pulse_emitter_spawns_ring_when_timer_reaches_interval() {
        let mut app = test_app();
        enter_playing(&mut app);

        let bolt = app
            .world_mut()
            .spawn((
                Transform::from_xyz(80.0, 120.0, 0.0),
                PulseEmitter {
                    base_range: 32.0,
                    range_per_level: 0.0,
                    stacks: 1,
                    speed: 50.0,
                    interval: 0.5,
                    timer: 0.49,
                },
            ))
            .id();

        // Advance time by injecting a delta that will push timer over interval.
        // MinimalPlugins provides Time, one update tick advances by a small dt.
        // We pre-set timer to 0.49 so even a tiny dt (e.g., 0.016) pushes it past 0.5.
        app.update();

        // Check that a PulseRing entity was spawned
        let mut ring_query = app.world_mut().query::<(
            &PulseRing,
            &PulseSource,
            &PulseRadius,
            &PulseMaxRadius,
            &PulseSpeed,
            &PulseDamaged,
            &Transform,
        )>();
        let rings: Vec<_> = ring_query.iter(app.world()).collect();
        assert_eq!(
            rings.len(),
            1,
            "expected one PulseRing spawned, got {}",
            rings.len()
        );

        let (_ring, source, radius, max_radius, speed, damaged, transform) = rings[0];
        assert_eq!(
            source.0, bolt,
            "PulseSource should reference the bolt entity"
        );
        assert!(
            (radius.0 - 0.0).abs() < f32::EPSILON,
            "new ring radius should be 0.0"
        );
        assert!(
            (max_radius.0 - 32.0).abs() < f32::EPSILON,
            "max radius should match effective_max_radius (32.0)"
        );
        assert!(
            (speed.0 - 50.0).abs() < f32::EPSILON,
            "ring speed should be 50.0"
        );
        assert!(damaged.0.is_empty(), "ring PulseDamaged should be empty");
        assert!(
            (transform.translation.x - 80.0).abs() < f32::EPSILON,
            "ring should spawn at bolt x position"
        );
        assert!(
            (transform.translation.y - 120.0).abs() < f32::EPSILON,
            "ring should spawn at bolt y position"
        );

        // Emitter timer should have been reset (timer - interval preserves fractional part)
        let emitter = app.world().get::<PulseEmitter>(bolt).unwrap();
        assert!(
            emitter.timer < emitter.interval,
            "emitter timer should be reset after emission, got {}",
            emitter.timer
        );
    }

    // ── Behavior 12: tick_pulse_emitter does NOT spawn before interval ──

    #[test]
    fn tick_pulse_emitter_does_not_spawn_before_interval() {
        let mut app = test_app();
        enter_playing(&mut app);

        let bolt = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                PulseEmitter {
                    base_range: 32.0,
                    range_per_level: 0.0,
                    stacks: 1,
                    speed: 50.0,
                    interval: 100.0, // Very large interval -- will never fire
                    timer: 0.0,
                },
            ))
            .id();

        app.update();

        let mut ring_query = app.world_mut().query::<&PulseRing>();
        let count = ring_query.iter(app.world()).count();
        assert_eq!(
            count, 0,
            "no PulseRing should spawn before interval elapses"
        );

        // Timer should have advanced
        let emitter = app.world().get::<PulseEmitter>(bolt).unwrap();
        assert!(
            emitter.timer > 0.0,
            "emitter timer should advance, got {}",
            emitter.timer
        );
    }

    // ── Behavior 13: tick_pulse_emitter reads bolt's current position ──

    #[test]
    fn tick_pulse_emitter_reads_current_bolt_position() {
        let mut app = test_app();
        enter_playing(&mut app);

        // Bolt starts at (200, 300) -- this is its "current" position
        let _bolt = app
            .world_mut()
            .spawn((
                Transform::from_xyz(200.0, 300.0, 0.0),
                PulseEmitter {
                    base_range: 32.0,
                    range_per_level: 0.0,
                    stacks: 1,
                    speed: 50.0,
                    interval: 0.5,
                    timer: 0.49, // About to fire
                },
            ))
            .id();

        app.update();

        let mut ring_query = app.world_mut().query::<(&PulseRing, &Transform)>();
        let rings: Vec<_> = ring_query.iter(app.world()).collect();
        assert_eq!(rings.len(), 1, "expected one ring spawned");

        let (_ring, transform) = rings[0];
        assert!(
            (transform.translation.x - 200.0).abs() < f32::EPSILON,
            "ring should spawn at bolt's current x (200.0), got {}",
            transform.translation.x
        );
        assert!(
            (transform.translation.y - 300.0).abs() < f32::EPSILON,
            "ring should spawn at bolt's current y (300.0), got {}",
            transform.translation.y
        );
    }

    // ── Behavior 14: tick_pulse_ring expands radius ──

    #[test]
    fn tick_pulse_ring_expands_radius_by_speed_times_dt() {
        let mut app = test_app();
        enter_playing(&mut app);

        let ring = app
            .world_mut()
            .spawn((
                PulseRing,
                PulseRadius(10.0),
                PulseMaxRadius(50.0),
                PulseSpeed(100.0),
                PulseDamaged::default(),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        app.update();

        let radius = app.world().get::<PulseRadius>(ring).unwrap();
        // After one tick, radius should have increased (speed * dt > 0)
        assert!(
            radius.0 > 10.0,
            "pulse radius should expand, got {}",
            radius.0
        );
    }

    #[test]
    fn tick_pulse_ring_zero_speed_no_expansion() {
        let mut app = test_app();
        enter_playing(&mut app);

        let ring = app
            .world_mut()
            .spawn((
                PulseRing,
                PulseRadius(10.0),
                PulseMaxRadius(50.0),
                PulseSpeed(0.0),
                PulseDamaged::default(),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        app.update();

        let radius = app.world().get::<PulseRadius>(ring).unwrap();
        assert!(
            (radius.0 - 10.0).abs() < f32::EPSILON,
            "pulse radius should not change with zero speed, got {}",
            radius.0
        );
    }

    // ── Behavior 15: despawn_finished_pulse_ring ──

    #[test]
    fn despawn_finished_pulse_ring_when_radius_equals_max() {
        let mut app = test_app();
        enter_playing(&mut app);

        let ring = app
            .world_mut()
            .spawn((
                PulseRing,
                PulseRadius(50.0),
                PulseMaxRadius(50.0),
                PulseSpeed(0.0),
                PulseDamaged::default(),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        app.update();

        assert!(
            app.world().get_entity(ring).is_err(),
            "pulse ring should be despawned when radius >= max_radius"
        );
    }

    #[test]
    fn despawn_finished_pulse_ring_when_radius_exceeds_max() {
        let mut app = test_app();
        enter_playing(&mut app);

        let ring = app
            .world_mut()
            .spawn((
                PulseRing,
                PulseRadius(50.1),
                PulseMaxRadius(50.0),
                PulseSpeed(0.0),
                PulseDamaged::default(),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        app.update();

        assert!(
            app.world().get_entity(ring).is_err(),
            "pulse ring should be despawned when radius > max_radius"
        );
    }

    // ── Behavior 16: apply_pulse_damage damages cells within radius ──

    #[test]
    fn pulse_ring_damages_cell_within_radius() {
        let mut app = damage_test_app();

        let cell = spawn_test_cell(&mut app, 20.0, 0.0);

        app.world_mut().spawn((
            PulseRing,
            PulseRadius(25.0),
            PulseMaxRadius(50.0),
            PulseSpeed(0.0),
            PulseDamaged::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert_eq!(
            collector.0.len(),
            1,
            "expected one DamageCell message, got {}",
            collector.0.len()
        );
        assert_eq!(collector.0[0].cell, cell);
        assert!(
            (collector.0[0].damage - BASE_BOLT_DAMAGE).abs() < f32::EPSILON,
            "expected damage {}, got {}",
            BASE_BOLT_DAMAGE,
            collector.0[0].damage
        );
        assert!(
            collector.0[0].source_chip.is_none(),
            "source_chip should be None for pulse damage"
        );
    }

    #[test]
    fn pulse_ring_does_not_damage_already_damaged_cell() {
        let mut app = damage_test_app();

        let cell = spawn_test_cell(&mut app, 20.0, 0.0);

        let mut already_damaged = HashSet::new();
        already_damaged.insert(cell);

        app.world_mut().spawn((
            PulseRing,
            PulseRadius(25.0),
            PulseMaxRadius(50.0),
            PulseSpeed(0.0),
            PulseDamaged(already_damaged),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert!(
            collector.0.is_empty(),
            "already-damaged cell should not receive DamageCell again"
        );
    }

    // ── Behavior 17: Each PulseRing damages cells independently ──

    #[test]
    fn each_pulse_ring_damages_cells_independently() {
        let mut app = damage_test_app();

        let cell = spawn_test_cell(&mut app, 15.0, 0.0);

        // Two rings at the same position, each with empty PulseDamaged
        app.world_mut().spawn((
            PulseRing,
            PulseRadius(25.0),
            PulseMaxRadius(50.0),
            PulseSpeed(0.0),
            PulseDamaged::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
        app.world_mut().spawn((
            PulseRing,
            PulseRadius(25.0),
            PulseMaxRadius(50.0),
            PulseSpeed(0.0),
            PulseDamaged::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert_eq!(
            collector.0.len(),
            2,
            "each ring should send its own DamageCell, expected 2, got {}",
            collector.0.len()
        );

        // Both messages should reference the same cell
        assert_eq!(collector.0[0].cell, cell);
        assert_eq!(collector.0[1].cell, cell);
    }

    // ── Behavior 18: Pulse ring does not damage non-CELL_LAYER entities ──

    #[test]
    fn pulse_ring_does_not_damage_non_cell_layer_entities() {
        let mut app = damage_test_app();

        // Spawn a bolt-layer entity (not a cell)
        let bolt_pos = Vec2::new(10.0, 0.0);
        app.world_mut().spawn((
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
            CollisionLayers::new(BOLT_LAYER, 0),
            Position2D(bolt_pos),
            GlobalPosition2D(bolt_pos),
            Spatial2D,
        ));

        // Spawn a wall-layer entity (not a cell)
        let wall_pos = Vec2::new(5.0, 0.0);
        app.world_mut().spawn((
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
            CollisionLayers::new(WALL_LAYER, 0),
            Position2D(wall_pos),
            GlobalPosition2D(wall_pos),
            Spatial2D,
        ));

        app.world_mut().spawn((
            PulseRing,
            PulseRadius(25.0),
            PulseMaxRadius(50.0),
            PulseSpeed(0.0),
            PulseDamaged::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert!(
            collector.0.is_empty(),
            "non-CELL_LAYER entities should not receive damage"
        );
    }

    #[test]
    fn pulse_ring_damages_entity_with_cell_layer_in_combined_mask() {
        let mut app = damage_test_app();

        // Entity with CELL_LAYER | WALL_LAYER -- should be damaged since it IS on CELL_LAYER
        let pos = Vec2::new(10.0, 0.0);
        let cell = app
            .world_mut()
            .spawn((
                Cell,
                Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
                CollisionLayers::new(CELL_LAYER | WALL_LAYER, 0),
                Position2D(pos),
                GlobalPosition2D(pos),
                Spatial2D,
            ))
            .id();

        app.world_mut().spawn((
            PulseRing,
            PulseRadius(25.0),
            PulseMaxRadius(50.0),
            PulseSpeed(0.0),
            PulseDamaged::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert_eq!(
            collector.0.len(),
            1,
            "entity with CELL_LAYER in combined mask should be damaged"
        );
        assert_eq!(collector.0[0].cell, cell);
    }
}
