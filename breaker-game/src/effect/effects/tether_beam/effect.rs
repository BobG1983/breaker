//! Two free-moving bolts connected by a crackling neon beam that damages intersected cells.

use std::collections::HashSet;

use bevy::{ecs::world::CommandQueue, prelude::*};
use rand::Rng;
use rantzsoft_physics2d::{
    aabb::Aabb2D, ccd::ray_vs_aabb, collision_layers::CollisionLayers, plugin::PhysicsSystems,
    resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Velocity2D};

use crate::{
    bolt::{
        components::{Bolt, BoltBaseDamage, BoltDefinitionRef, BoltRadius},
        definition::BoltDefinition,
        registry::BoltRegistry,
        resources::DEFAULT_BOLT_BASE_DAMAGE,
    },
    cells::{components::Cell, messages::DamageCell},
    effect::{
        core::{EffectSourceChip, chip_attribution},
        effects::damage_boost::ActiveDamageBoosts,
    },
    shared::{CELL_LAYER, CleanupOnNodeExit, rng::GameRng},
    state::types::NodeState,
};

/// Marker on a tether bolt entity, indicating it belongs to a tether beam.
#[derive(Component)]
pub(crate) struct TetherBoltMarker;

/// Marker component on chain-mode beam entities to distinguish them from standard tether beams.
#[derive(Component)]
pub(crate) struct TetherChainBeam;

/// Resource inserted when chain mode is active. Stores parameters for dynamic beam rebuilding.
#[derive(Resource)]
pub(crate) struct TetherChainActive {
    pub damage_mult: f32,
    pub effective_damage_multiplier: f32,
    pub base_damage: f32,
    pub source_chip: Option<String>,
    /// Bolt count from the last rebuild — compared against the current count to detect changes.
    pub last_bolt_count: usize,
}

/// The beam entity linking two tether bolts.
#[derive(Component)]
pub(crate) struct TetherBeamComponent {
    /// First tether bolt entity.
    pub bolt_a: Entity,
    /// Second tether bolt entity.
    pub bolt_b: Entity,
    /// Damage multiplier applied to `base_damage`.
    pub damage_mult: f32,
    /// Effective damage multiplier snapshotted from the source entity's
    /// `ActiveDamageBoosts` at fire-time. Default `1.0`.
    pub effective_damage_multiplier: f32,
    /// Snapshotted base damage from the source entity's `BoltBaseDamage`.
    /// Falls back to `DEFAULT_BOLT_BASE_DAMAGE` when the source has no `BoltBaseDamage`.
    pub base_damage: f32,
}

/// Spawns two tethered bolts with a damaging beam between them (standard mode),
/// or connects all existing bolts with chain beams (chain mode).
///
/// Evolution of `ChainBolt`. The beam is a line segment between the two bolt
/// positions — cells intersecting the beam take damage each tick.
pub(crate) fn fire(
    entity: Entity,
    damage_mult: f32,
    chain: bool,
    source_chip: &str,
    world: &mut World,
) {
    if chain {
        fire_chain(entity, damage_mult, source_chip, world);
    } else {
        fire_standard(entity, damage_mult, source_chip, world);
    }
}

/// Spawns a single extra bolt with a random velocity direction at the given position.
fn spawn_tether_bolt(world: &mut World, spawn_pos: Vec2, bolt_def: &BoltDefinition) -> Entity {
    let angle = {
        let mut rng = world.resource_mut::<GameRng>();
        rng.0.random_range(0.0..std::f32::consts::TAU)
    };
    let direction = Vec2::new(angle.cos(), angle.sin());
    let velocity = Velocity2D(direction * bolt_def.base_speed);
    let mut queue = CommandQueue::default();
    let entity = {
        let mut commands = Commands::new(&mut queue, world);
        Bolt::builder()
            .at_position(spawn_pos)
            .definition(bolt_def)
            .with_velocity(velocity)
            .extra()
            .headless()
            .spawn(&mut commands)
    };
    queue.apply(world);
    entity
}

/// Standard mode: spawn two tethered bolts with a beam between them.
fn fire_standard(entity: Entity, damage_mult: f32, source_chip: &str, world: &mut World) {
    let spawn_pos = super::super::entity_position(world, entity);

    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .map_or_else(|| "Bolt".to_owned(), |r| r.0.clone());
    let Some(bolt_def) = world
        .resource::<BoltRegistry>()
        .get(&def_ref)
        .cloned()
        .or_else(|| world.resource::<BoltRegistry>().get("Bolt").cloned())
    else {
        warn!("default Bolt definition missing");
        return;
    };

    let bolt_a = spawn_tether_bolt(world, spawn_pos, &bolt_def);
    let bolt_b = spawn_tether_bolt(world, spawn_pos, &bolt_def);

    let edm = world
        .get::<ActiveDamageBoosts>(entity)
        .map_or(1.0, ActiveDamageBoosts::multiplier);

    let base_damage = world
        .get::<BoltBaseDamage>(entity)
        .map_or(DEFAULT_BOLT_BASE_DAMAGE, |d| d.0);

    // Spawn the beam entity linking both bolts
    let _beam = world
        .spawn((
            TetherBeamComponent {
                bolt_a,
                bolt_b,
                damage_mult,
                effective_damage_multiplier: edm,
                base_damage,
            },
            EffectSourceChip::new(source_chip),
            CleanupOnNodeExit,
        ))
        .id();

    // Add TetherBoltMarker to each bolt
    world.entity_mut(bolt_a).insert(TetherBoltMarker);
    world.entity_mut(bolt_b).insert(TetherBoltMarker);
}

/// Chain mode: connect all existing bolts with chain beams.
fn fire_chain(entity: Entity, damage_mult: f32, source_chip: &str, world: &mut World) {
    // Despawn all existing TetherChainBeam entities
    let existing_chain_beams: Vec<Entity> = world
        .query_filtered::<Entity, With<TetherChainBeam>>()
        .iter(world)
        .collect();
    for e in existing_chain_beams {
        world.despawn(e);
    }

    // Snapshot EDM from the fire entity
    let edm = world
        .get::<ActiveDamageBoosts>(entity)
        .map_or(1.0, ActiveDamageBoosts::multiplier);

    let base_damage = world
        .get::<BoltBaseDamage>(entity)
        .map_or(DEFAULT_BOLT_BASE_DAMAGE, |d| d.0);

    // Query all bolt entities and sort by index (ascending spawn order)
    let mut bolts: Vec<Entity> = world
        .query_filtered::<Entity, With<Bolt>>()
        .iter(world)
        .collect();
    bolts.sort_by_key(|e| e.index());

    // Spawn chain beams for each consecutive pair
    for pair in bolts.windows(2) {
        world.spawn((
            TetherBeamComponent {
                bolt_a: pair[0],
                bolt_b: pair[1],
                damage_mult,
                effective_damage_multiplier: edm,
                base_damage,
            },
            TetherChainBeam,
            EffectSourceChip::new(source_chip),
            CleanupOnNodeExit,
        ));
    }

    // Insert TetherChainActive resource
    world.insert_resource(TetherChainActive {
        damage_mult,
        effective_damage_multiplier: edm,
        base_damage,
        source_chip: chip_attribution(source_chip),
        last_bolt_count: bolts.len(),
    });
}

/// No-op for standard mode. Chain mode: removes `TetherChainActive` resource and despawns chain beams.
pub(crate) fn reverse(
    _entity: Entity,
    _damage_mult: f32,
    chain: bool,
    _source_chip: &str,
    world: &mut World,
) {
    if chain {
        world.remove_resource::<TetherChainActive>();

        let chain_beams: Vec<Entity> = world
            .query_filtered::<Entity, With<TetherChainBeam>>()
            .iter(world)
            .collect();
        for e in chain_beams {
            world.despawn(e);
        }
    }
}

type TetherBoltQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Position2D,
        Has<TetherBoltMarker>,
        Option<&'static BoltRadius>,
    ),
    With<Bolt>,
>;

/// Tick system: damages cells whose AABB intersects each tether beam segment.
///
/// For each beam, looks up the positions of `bolt_a` and `bolt_b`. If either bolt
/// is missing, despawns the beam. Otherwise, computes broadphase via quadtree
/// AABB query and narrowphase via ray-vs-AABB intersection, sending `DamageCell`
/// for each cell hit by the beam segment.
///
/// The beam has an effective half-width equal to the bolt radius (from
/// `BoltRadius`), so cells whose AABBs are within the bolt radius of the beam
/// line segment are considered intersecting.
pub(crate) fn tick_tether_beam(
    mut commands: Commands,
    beams: Query<(Entity, &TetherBeamComponent, Option<&EffectSourceChip>)>,
    bolts: TetherBoltQuery,
    quadtree: Res<CollisionQuadtree>,
    cell_aabbs: Query<(&Aabb2D, &GlobalPosition2D), With<Cell>>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let query_layers = CollisionLayers::new(0, CELL_LAYER);

    for (beam_entity, component, esc) in &beams {
        // Look up both bolt positions; despawn beam if either is missing.
        // Extract radius from bolt_a; bolt_b discards that field.
        let (pos_a, bolt_radius) = if let Ok((p, _, radius)) = bolts.get(component.bolt_a) {
            (p.0, radius.map_or(8.0, |r| r.0))
        } else {
            commands.entity(beam_entity).despawn();
            continue;
        };
        let pos_b = if let Ok((p, ..)) = bolts.get(component.bolt_b) {
            p.0
        } else {
            commands.entity(beam_entity).despawn();
            continue;
        };
        let beam_half_width = bolt_radius;

        // Broadphase: compute beam bounding box expanded by beam half-width and
        // query quadtree. The expansion ensures cells near (but not exactly on)
        // the beam line are included as candidates.
        let beam_aabb =
            Aabb2D::from_min_max(pos_a.min(pos_b), pos_a.max(pos_b)).expand_by(beam_half_width);
        let candidates = quadtree
            .quadtree
            .query_aabb_filtered(&beam_aabb, query_layers);

        // Narrowphase: test line-segment vs cell AABB intersection
        let beam_vec = pos_b - pos_a;
        let max_dist = beam_vec.length();
        let direction = beam_vec.normalize_or_zero();
        let damage =
            component.base_damage * component.damage_mult * component.effective_damage_multiplier;

        let mut damaged_this_tick: HashSet<Entity> = HashSet::new();

        for cell in candidates {
            if damaged_this_tick.contains(&cell) {
                continue;
            }

            let Ok((local_aabb, global_pos)) = cell_aabbs.get(cell) else {
                continue;
            };

            // Compute world-space AABB for the cell, expanded by the beam
            // half-width (Minkowski sum) so a point-ray test is equivalent to a
            // thick-beam-vs-AABB test.
            let world_aabb = Aabb2D::new(global_pos.0 + local_aabb.center, local_aabb.half_extents)
                .expand_by(beam_half_width);

            // Check ray intersection OR origin-inside-AABB
            let ray_hit = ray_vs_aabb(pos_a, direction, max_dist, &world_aabb);
            let origin_inside = world_aabb.contains_point(pos_a);

            if ray_hit.is_some() || origin_inside {
                damaged_this_tick.insert(cell);
                damage_writer.write(DamageCell {
                    cell,
                    damage,
                    source_chip: esc.and_then(EffectSourceChip::source_chip),
                });
            }
        }
    }
}

/// Maintains chain beams when the bolt count changes (bolts spawned or despawned).
///
/// Rebuilds the chain when bolts are added or removed, creating N-1 beams for
/// N sorted bolts. No-ops when bolt count is unchanged.
pub(crate) fn maintain_tether_chain(
    mut commands: Commands,
    mut chain_active: ResMut<TetherChainActive>,
    bolts: Query<Entity, With<Bolt>>,
    chain_beams: Query<Entity, With<TetherChainBeam>>,
) {
    let bolt_count = bolts.iter().count();
    if bolt_count == chain_active.last_bolt_count {
        return;
    }

    // Despawn all existing chain beam entities
    for beam_entity in &chain_beams {
        commands.entity(beam_entity).despawn();
    }

    // Collect and sort bolts by index (ascending spawn order)
    let mut sorted_bolts: Vec<Entity> = bolts.iter().collect();
    sorted_bolts.sort_by_key(|e| e.index());

    // Spawn N-1 beams for consecutive bolt pairs
    let esc = EffectSourceChip(chain_active.source_chip.clone());
    for pair in sorted_bolts.windows(2) {
        commands.spawn((
            TetherBeamComponent {
                bolt_a: pair[0],
                bolt_b: pair[1],
                damage_mult: chain_active.damage_mult,
                effective_damage_multiplier: chain_active.effective_damage_multiplier,
                base_damage: chain_active.base_damage,
            },
            TetherChainBeam,
            esc.clone(),
            CleanupOnNodeExit,
        ));
    }

    chain_active.last_bolt_count = bolt_count;
}

/// Registers systems for `TetherBeam` effect.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            maintain_tether_chain
                .run_if(resource_exists::<TetherChainActive>.and(in_state(NodeState::Playing)))
                .before(tick_tether_beam),
            tick_tether_beam.run_if(in_state(NodeState::Playing)),
        )
            .after(PhysicsSystems::MaintainQuadtree),
    );
    app.add_systems(
        OnEnter(crate::state::types::NodeState::Teardown),
        cleanup_tether_chain_resource.run_if(resource_exists::<TetherChainActive>),
    );
}

/// Remove `TetherChainActive` resource on node exit to prevent stale chain
/// state from leaking into the next node.
fn cleanup_tether_chain_resource(mut commands: Commands) {
    commands.remove_resource::<TetherChainActive>();
}
