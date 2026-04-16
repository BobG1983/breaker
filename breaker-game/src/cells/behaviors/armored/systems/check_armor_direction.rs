//! Filters bolt-on-armor damage before the damage pipeline applies it.
//!
//! Reads `BoltImpactCell` messages for the current tick, looks up each hit's
//! target cell, and — for cells that are armored with a facing that matches
//! the impact normal — either drops the corresponding `DamageDealt<Cell>`
//! entry from `Messages<DamageDealt<Cell>>` (block) or decrements the bolt's
//! `PiercingRemaining` by `armor_value` (breakthrough). Non-armored hits,
//! and hits on armored cells from weak-point faces, pass through unchanged.
//!
//! Scheduling: `FixedUpdate`,
//! `.after(BoltSystems::CellCollision).before(DeathPipelineSystems::ApplyDamage)`,
//! `.run_if(in_state(NodeState::Playing))`. Ordering is the entire
//! correctness argument — the system mutates the `DamageDealt<Cell>` queue
//! after `bolt_cell_collision` has populated it and before
//! `apply_damage::<Cell>` reads from it.

use bevy::{ecs::message::Messages, prelude::*};

use crate::{
    bolt::components::PiercingRemaining,
    cells::behaviors::armored::components::{ArmorDirection, ArmorFacing, ArmorValue, ArmoredCell},
    prelude::*,
};

/// Query for reading armor state on candidate cells. Read-only — the system
/// never mutates `ArmorValue` or `ArmorFacing`.
type ArmorQuery<'w, 's> =
    Query<'w, 's, (&'static ArmorValue, &'static ArmorFacing), With<ArmoredCell>>;

/// Query for mutating bolt piercing charges on breakthrough.
type BoltPiercingQuery<'w, 's> = Query<'w, 's, &'static mut PiercingRemaining>;

pub(crate) fn check_armor_direction(
    mut impacts: MessageReader<BoltImpactCell>,
    mut damage: ResMut<Messages<DamageDealt<Cell>>>,
    armor_query: ArmorQuery,
    mut bolt_query: BoltPiercingQuery,
) {
    // 1. Build the blocklist and decrement piercing on breakthrough hits.
    //    Collect (bolt, cell) pairs whose DamageDealt<Cell> entries must be
    //    dropped. Non-armored hits and weak-face hits contribute nothing to
    //    the blocklist.
    let mut blocklist: Vec<(Entity, Entity)> = Vec::new();

    for impact in impacts.read() {
        let Ok((armor_value, armor_facing)) = armor_query.get(impact.cell) else {
            // Not an armored cell — nothing to do for this hit.
            continue;
        };
        if !normal_hits_armored_face(impact.impact_normal, armor_facing.0) {
            // Hit landed on a non-armored (weak-point or side) face — pass
            // through unchanged.
            continue;
        }
        if impact.piercing_remaining >= u32::from(armor_value.0) {
            // Breakthrough — damage applies normally, but the bolt pays
            // `armor_value` piercing charges on top of any pierce-through
            // the collision system already consumed.
            if let Ok(mut pr) = bolt_query.get_mut(impact.bolt) {
                pr.0 = pr.0.saturating_sub(u32::from(armor_value.0));
            }
        } else {
            // Block — flag this (bolt, cell) pair for removal from the
            // damage queue.
            blocklist.push((impact.bolt, impact.cell));
        }
    }

    // 2. Fast path — nothing to block, the queue is unchanged.
    if blocklist.is_empty() {
        return;
    }

    // 3. Drain the damage queue, filter out the blocked pairs, and re-extend.
    //    `Messages<T>::drain()` consumes BOTH internal buffers, so all
    //    pending entries are visible. After re-extending, only unblocked
    //    entries remain for `apply_damage::<Cell>` to process.
    //
    //    First-match-wins on the blocklist: if two impacts on the same tick
    //    both target the same (bolt, cell) pair, each one "consumes" one
    //    matching `DamageDealt<Cell>` entry. Writer-code implements this by
    //    removing one blocklist entry per matched damage message.
    let drained: Vec<DamageDealt<Cell>> = damage.drain().collect();
    for msg in drained {
        // Entity::PLACEHOLDER never matches a real bolt entity, so dealerless
        // damage messages (e.g. from effects) cannot false-match the blocklist.
        let key = (msg.dealer.unwrap_or(Entity::PLACEHOLDER), msg.target);
        if let Some(idx) = blocklist
            .iter()
            .position(|&(bolt, cell)| bolt == key.0 && cell == key.1)
        {
            blocklist.swap_remove(idx);
            // Drop this message — do not re-write it.
            continue;
        }
        damage.write(msg);
    }
}

/// Returns `true` when `impact_normal` points away from the armored face of
/// the cell.
///
/// `impact_normal` is the surface normal **at the contact point** as
/// reported by `rantzsoft_physics2d::SweepHit::normal` — it points from the
/// cell surface outward at the point of contact, which means the normal
/// encodes which face of the cell was hit. A bolt coming from below that
/// strikes the bottom face of a cell receives `normal == (0, -1)`.
///
/// Mapping: if the armor plates the `ArmorDirection::Bottom` face, an
/// `impact_normal` whose `y` component is strictly less than 0 hits the
/// armored face. `ArmorDirection::Top` → `normal.y > 0`.
/// `ArmorDirection::Left` → `normal.x < 0`. `ArmorDirection::Right`
/// → `normal.x > 0`.
#[inline]
fn normal_hits_armored_face(impact_normal: Vec2, facing: ArmorDirection) -> bool {
    match facing {
        ArmorDirection::Bottom => impact_normal.y < 0.0,
        ArmorDirection::Top => impact_normal.y > 0.0,
        ArmorDirection::Left => impact_normal.x < 0.0,
        ArmorDirection::Right => impact_normal.x > 0.0,
    }
}
