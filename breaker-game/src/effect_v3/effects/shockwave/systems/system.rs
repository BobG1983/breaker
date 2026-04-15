//! Shockwave systems — tick expansion, damage application, despawn.

use bevy::prelude::*;

use super::super::components::*;
use crate::{effect_v3::components::EffectSourceChip, prelude::*};

/// Alive cell lookup — entity + position, excludes dead cells.
type AliveCellQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static Position2D), (With<Cell>, Without<Dead>)>;

/// Shockwave tick + damage query.
type ShockwaveQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Position2D,
        &'static ShockwaveRadius,
        &'static ShockwaveBaseDamage,
        &'static ShockwaveDamageMultiplier,
        &'static mut ShockwaveDamaged,
        Option<&'static EffectSourceChip>,
    ),
>;

/// Expands shockwave radius each frame based on speed.
pub fn tick_shockwave(mut query: Query<(&mut ShockwaveRadius, &ShockwaveSpeed)>, time: Res<Time>) {
    let dt = time.delta_secs();
    for (mut radius, speed) in &mut query {
        radius.0 = speed.0.mul_add(dt, radius.0);
    }
}

/// Applies damage to cells within the expanding shockwave radius.
pub(crate) fn apply_shockwave_damage(
    mut shockwave_query: ShockwaveQuery,
    cell_query: AliveCellQuery,
    mut damage_writer: MessageWriter<DamageDealt<Cell>>,
) {
    for (sw_entity, sw_pos, sw_radius, base_dmg, dmg_mult, mut damaged, chip) in
        &mut shockwave_query
    {
        for (cell_entity, cell_pos) in &cell_query {
            if damaged.0.contains(&cell_entity) {
                continue;
            }
            let distance = sw_pos.0.distance(cell_pos.0);
            if distance <= sw_radius.0 {
                damaged.0.insert(cell_entity);
                let damage = base_dmg.0 * dmg_mult.0;
                damage_writer.write(DamageDealt {
                    dealer:      Some(sw_entity),
                    target:      cell_entity,
                    amount:      damage,
                    source_chip: chip.and_then(|c| c.0.clone()),
                    _marker:     std::marker::PhantomData,
                });
            }
        }
    }
}

/// Despawns shockwaves that have reached their maximum radius.
pub fn despawn_finished_shockwave(
    query: Query<(Entity, &ShockwaveRadius, &ShockwaveMaxRadius)>,
    mut commands: Commands,
) {
    for (entity, radius, max_radius) in &query {
        if radius.0 >= max_radius.0 {
            commands.entity(entity).despawn();
        }
    }
}
