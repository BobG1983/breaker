//! Chain lightning systems — tick arc propagation.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::components::*;
use crate::{
    cells::components::Cell,
    shared::death_pipeline::{DamageDealt, Dead},
};

/// Alive cell lookup — entity + position, excludes dead cells.
type AliveCellQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static Position2D), (With<Cell>, Without<Dead>)>;

/// Advances chain lightning state machine each frame — finds targets, moves arcs, deals damage.
pub(crate) fn tick_chain_lightning(
    mut chain_query: Query<(Entity, &mut ChainLightningChain)>,
    cell_query: AliveCellQuery,
    mut damage_writer: MessageWriter<DamageDealt<Cell>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();

    for (chain_entity, mut chain) in &mut chain_query {
        match chain.state.clone() {
            ChainState::Idle => {
                if chain.remaining_jumps == 0 {
                    commands.entity(chain_entity).despawn();
                    continue;
                }

                // Find the nearest unhit cell within range of the current source position.
                let source_pos = chain.source_pos;
                let mut best: Option<(Entity, Vec2, f32)> = None;
                for (cell_entity, cell_pos) in &cell_query {
                    if chain.hit_set.contains(&cell_entity) {
                        continue;
                    }
                    let dist = source_pos.distance(cell_pos.0);
                    if dist <= chain.range && best.is_none_or(|(_, _, best_dist)| dist < best_dist)
                    {
                        best = Some((cell_entity, cell_pos.0, dist));
                    }
                }

                if let Some((target, target_pos, _)) = best {
                    // Spawn an arc visual entity.
                    let arc_entity = commands
                        .spawn((ChainLightningArc, Position2D(source_pos)))
                        .id();
                    chain.state = ChainState::ArcTraveling {
                        target,
                        target_pos,
                        arc_entity,
                        arc_pos: source_pos,
                    };
                } else {
                    // No valid target — chain ends.
                    commands.entity(chain_entity).despawn();
                }
            }
            ChainState::ArcTraveling {
                target,
                target_pos,
                arc_entity,
                arc_pos,
            } => {
                // Move arc toward target.
                let to_target = target_pos - arc_pos;
                let dist = to_target.length();
                let move_dist = chain.arc_speed * dt;

                if move_dist >= dist {
                    // Arc has arrived — deal damage and advance chain.
                    chain.hit_set.insert(target);
                    chain.remaining_jumps = chain.remaining_jumps.saturating_sub(1);
                    chain.source_pos = target_pos;
                    chain.state = ChainState::Idle;

                    // Despawn the arc visual.
                    commands.entity(arc_entity).despawn();

                    // Deal damage to the target cell.
                    damage_writer.write(DamageDealt {
                        dealer: Some(chain_entity),
                        target,
                        amount: chain.damage,
                        source_chip: None,
                        _marker: std::marker::PhantomData,
                    });
                } else {
                    // Move arc closer.
                    let direction = to_target.normalize_or(Vec2::ZERO);
                    let new_pos = direction.mul_add(Vec2::splat(move_dist), arc_pos);
                    chain.state = ChainState::ArcTraveling {
                        target,
                        target_pos,
                        arc_entity,
                        arc_pos: new_pos,
                    };
                }
            }
        }
    }
}
