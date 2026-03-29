//! Arc damage jumping between nearby cells — chains between random targets in range.
//!
//! Reworked from instant batch damage (`ChainLightningRequest` + `process_chain_lightning`)
//! to sequential arc-based chaining (`ChainLightningChain` + `tick_chain_lightning`).

use std::collections::HashSet;

use bevy::{ecs::system::SystemParam, prelude::*};
use rand::prelude::IndexedRandom;
use rantzsoft_physics2d::{
    collision_layers::CollisionLayers, plugin::PhysicsSystems, resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D};

use crate::{
    bolt::BASE_BOLT_DAMAGE,
    cells::{components::Cell, messages::DamageCell},
    effect::{
        EffectiveDamageMultiplier,
        core::{EffectSourceChip, chip_attribution},
    },
    shared::{CELL_LAYER, CleanupOnNodeExit, playing_state::PlayingState, rng::GameRng},
};

/// Stateful chain entity that tracks the chain lightning's progression
/// through multiple targets over successive ticks.
#[derive(Component)]
pub struct ChainLightningChain {
    /// Position of the last-hit target (origin for next arc).
    pub source: Vec2,
    /// Number of jumps remaining.
    pub remaining_jumps: u32,
    /// Pre-computed damage per arc hit.
    pub damage: f32,
    /// Set of already-hit cell entities (excluded from future targeting).
    pub hit_set: HashSet<Entity>,
    /// Current chain state machine.
    pub state: ChainState,
    /// Maximum jump distance for target selection.
    pub range: f32,
    /// Arc travel speed in world units per second.
    pub arc_speed: f32,
}

/// State machine for the chain lightning chain entity.
#[derive(Debug)]
pub enum ChainState {
    /// Waiting to select the next target and spawn an arc.
    Idle,
    /// An arc is traveling toward a target cell.
    ArcTraveling {
        /// The cell entity being targeted.
        target: Entity,
        /// The target cell's position at arc-spawn time.
        target_pos: Vec2,
        /// The arc marker entity.
        arc_entity: Entity,
        /// The arc's current position (updated each tick).
        arc_pos: Vec2,
    },
}

/// Bare marker component on arc entities. All arc state (target, position)
/// lives in [`ChainState::ArcTraveling`] on the parent chain entity.
#[derive(Component)]
pub struct ChainLightningArc;

pub(crate) fn fire(
    entity: Entity,
    arcs: u32,
    range: f32,
    damage_mult: f32,
    arc_speed: f32,
    source_chip: &str,
    world: &mut World,
) {
    // Early return: no arcs requested
    if arcs == 0 {
        return;
    }

    // Early return: cannot find targets with non-positive range
    if range <= 0.0 {
        return;
    }

    // Early return: arcs cannot travel with non-positive speed
    if arc_speed <= 0.0 {
        return;
    }

    // Read entity position from Position2D
    let position = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);

    // Read effective damage multiplier
    let edm = world
        .get::<EffectiveDamageMultiplier>(entity)
        .map_or(1.0, |e| e.0);

    // Pre-compute damage
    let damage = BASE_BOLT_DAMAGE * damage_mult * edm;

    // Query quadtree for cells in range
    let query_layers = CollisionLayers::new(0, CELL_LAYER);
    let candidates = world
        .resource::<CollisionQuadtree>()
        .quadtree
        .query_circle_filtered(position, range, query_layers);

    if candidates.is_empty() {
        return;
    }

    // RNG-pick one target from candidates
    let target = {
        let mut rng = world.resource_mut::<GameRng>();
        let Some(&picked) = candidates.choose(&mut rng.0) else {
            return;
        };
        picked
    };

    // Send DamageCell immediately for the first target
    let chip = chip_attribution(source_chip);
    world
        .resource_mut::<Messages<DamageCell>>()
        .write(DamageCell {
            cell: target,
            damage,
            source_chip: chip.clone(),
        });

    // If arcs == 1, no chain entity needed (first target was the only arc)
    if arcs == 1 {
        return;
    }

    // Read target's GlobalPosition2D for the new source position
    let target_pos = world
        .get::<GlobalPosition2D>(target)
        .map_or(Vec2::ZERO, |gp| gp.0);

    // Build hit_set with the first target
    let mut hit_set = HashSet::new();
    hit_set.insert(target);

    // Spawn ChainLightningChain entity for remaining jumps
    world.spawn((
        ChainLightningChain {
            source: target_pos,
            remaining_jumps: arcs - 1,
            damage,
            hit_set,
            state: ChainState::Idle,
            range,
            arc_speed,
        },
        EffectSourceChip(chip),
        CleanupOnNodeExit,
    ));
}

pub(crate) fn reverse(_entity: Entity, _source_chip: &str, world: &mut World) {
    let _ = world;
}

/// Bundled world queries for chain lightning tick — reduces system parameter count.
#[derive(SystemParam)]
pub struct ChainLightningWorld<'w, 's> {
    quadtree: Res<'w, CollisionQuadtree>,
    cell_positions: Query<'w, 's, &'static GlobalPosition2D, With<Cell>>,
    rng: ResMut<'w, GameRng>,
    arc_transforms: Query<'w, 's, &'static mut Transform, With<ChainLightningArc>>,
}

/// Tick system for chain lightning progression.
///
/// - In `Idle` state: select next target, spawn arc, transition to `ArcTraveling`
/// - In `ArcTraveling` state: advance arc toward target, damage on arrival
pub fn tick_chain_lightning(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    mut chains: Query<(Entity, &mut ChainLightningChain, Option<&EffectSourceChip>)>,
    mut world: ChainLightningWorld,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let dt = time.delta_secs();
    let query_layers = CollisionLayers::new(0, CELL_LAYER);

    for (chain_entity, mut chain, esc) in &mut chains {
        // Take ownership of the state to work with it
        let current_state = std::mem::replace(&mut chain.state, ChainState::Idle);

        match current_state {
            ChainState::Idle => {
                // Guard: if no remaining jumps, despawn chain
                if chain.remaining_jumps == 0 {
                    commands.entity(chain_entity).despawn();
                    continue;
                }

                // Query quadtree for cells in range of chain.source
                let candidates = world.quadtree.quadtree.query_circle_filtered(
                    chain.source,
                    chain.range,
                    query_layers,
                );

                // Filter out entities in hit_set
                let valid: Vec<Entity> = candidates
                    .into_iter()
                    .filter(|e| !chain.hit_set.contains(e))
                    .collect();

                if valid.is_empty() {
                    commands.entity(chain_entity).despawn();
                    continue;
                }

                // RNG-pick one target from valid candidates
                let Some(&target) = valid.choose(&mut world.rng.0) else {
                    commands.entity(chain_entity).despawn();
                    continue;
                };

                // Read target's GlobalPosition2D
                let target_pos = world
                    .cell_positions
                    .get(target)
                    .map_or(Vec2::ZERO, |gp| gp.0);

                // Spawn ChainLightningArc visual entity at source position
                let arc_entity = commands
                    .spawn((
                        ChainLightningArc,
                        Transform::from_translation(chain.source.extend(0.0)),
                        CleanupOnNodeExit,
                    ))
                    .id();

                // Transition to ArcTraveling
                chain.state = ChainState::ArcTraveling {
                    target,
                    target_pos,
                    arc_entity,
                    arc_pos: chain.source,
                };
            }
            ChainState::ArcTraveling {
                target,
                target_pos,
                arc_entity,
                arc_pos,
            } => {
                let diff = target_pos - arc_pos;
                let distance = diff.length();

                // Check if arc arrives this tick
                let step = chain.arc_speed * dt;

                if distance <= step || distance < f32::EPSILON {
                    // Arc has arrived at target

                    // Single fetch: check if target still exists and grab its position
                    let target_gp = world.cell_positions.get(target);
                    let target_exists = target_gp.is_ok();

                    if target_exists {
                        // Send DamageCell for the target
                        let source_chip = esc.and_then(|e| e.0.clone());
                        damage_writer.write(DamageCell {
                            cell: target,
                            damage: chain.damage,
                            source_chip,
                        });
                    }

                    // Add target to hit_set
                    chain.hit_set.insert(target);

                    // Update source to target position
                    // Use GlobalPosition2D if available, otherwise target_pos
                    chain.source = target_gp.map_or(target_pos, |gp| gp.0);

                    // Decrement remaining jumps
                    chain.remaining_jumps -= 1;

                    // Despawn arc entity
                    commands.entity(arc_entity).despawn();

                    if chain.remaining_jumps == 0 {
                        // No more jumps: despawn chain
                        commands.entity(chain_entity).despawn();
                    } else {
                        // Set state back to Idle
                        chain.state = ChainState::Idle;
                    }
                } else {
                    // Advance arc toward target
                    let direction = diff / distance;
                    let new_arc_pos = arc_pos + direction * step;

                    // Update arc entity Transform
                    if let Ok(mut transform) = world.arc_transforms.get_mut(arc_entity) {
                        transform.translation = new_arc_pos.extend(0.0);
                    }

                    // Update chain state with new arc position
                    chain.state = ChainState::ArcTraveling {
                        target,
                        target_pos,
                        arc_entity,
                        arc_pos: new_arc_pos,
                    };
                }
            }
        }
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        tick_chain_lightning
            .after(PhysicsSystems::MaintainQuadtree)
            .run_if(in_state(PlayingState::Active)),
    );
}
