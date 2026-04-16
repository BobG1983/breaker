//! Filters bolt damage on cells with the `BoltImmune` marker.
//!
//! Reads `BoltImpactCell` messages and drops any `DamageDealt<Cell>`
//! whose target has `BoltImmune` and whose dealer is a bolt that hit
//! that cell. Effect-sourced damage passes through.

use bevy::{ecs::message::Messages, prelude::*};

use crate::{cells::behaviors::survival::components::BoltImmune, prelude::*};

/// Drains bolt-sourced damage for `BoltImmune` cells. Each blocklist
/// entry is consumed on first match (one suppression per impact).
pub(crate) fn suppress_bolt_immune_damage(
    mut impacts: MessageReader<BoltImpactCell>,
    mut damage: ResMut<Messages<DamageDealt<Cell>>>,
    immune_query: Query<(), With<BoltImmune>>,
) {
    // 1. Build the blocklist: (bolt, cell) pairs whose DamageDealt<Cell>
    //    entries must be dropped.
    let mut blocklist: Vec<(Entity, Entity)> = Vec::new();

    for impact in impacts.read() {
        if immune_query.get(impact.cell).is_ok() {
            blocklist.push((impact.bolt, impact.cell));
        }
    }

    // 2. Fast path — nothing to block, the queue is unchanged.
    if blocklist.is_empty() {
        return;
    }

    // 3. Drain the damage queue, filter out the blocked pairs, and re-extend.
    //    First-match-wins on the blocklist: each matching DamageDealt<Cell>
    //    "consumes" one blocklist entry via swap_remove.
    let drained: Vec<DamageDealt<Cell>> = damage.drain().collect();
    for msg in drained {
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
