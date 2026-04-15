//! Pulse systems — emitter tick, ring expansion, damage application, despawn.

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;
use rantzsoft_stateflow::CleanupOnExit;

use super::super::components::{
    PulseEmitter, PulseRing, PulseRingBaseDamage, PulseRingDamageMultiplier, PulseRingDamaged,
    PulseRingMaxRadius, PulseRingRadius, PulseRingSpeed,
};
use crate::{
    bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE},
    cells::components::Cell,
    effect_v3::{components::EffectSourceChip, effects::DamageBoostConfig, stacking::EffectStack},
    shared::death_pipeline::{DamageDealt, Dead},
    state::types::NodeState,
};

/// Alive cell lookup — entity + position, excludes dead cells.
type AliveCellQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static Position2D), (With<Cell>, Without<Dead>)>;

/// Pulse ring tick + damage query.
type PulseRingQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Position2D,
        &'static PulseRingRadius,
        &'static PulseRingBaseDamage,
        &'static PulseRingDamageMultiplier,
        &'static mut PulseRingDamaged,
        Option<&'static EffectSourceChip>,
    ),
>;

/// Pulse emitter tick query — reads `BoltBaseDamage` and
/// `EffectStack<DamageBoostConfig>` from the emitter entity for per-ring
/// snapshot at spawn time.
type PulseEmitterQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut PulseEmitter,
        &'static Position2D,
        Option<&'static BoltBaseDamage>,
        Option<&'static EffectStack<DamageBoostConfig>>,
    ),
>;

/// Decrements pulse emitter timers each frame and spawns pulse rings when the
/// timer reaches zero. Each spawned ring snapshots `BoltBaseDamage` and
/// `EffectStack<DamageBoostConfig>` from the emitter entity at spawn time, and
/// inherits the emitter's `source_chip` string via `EffectSourceChip`.
pub(crate) fn tick_pulse(mut query: PulseEmitterQuery, time: Res<Time>, mut commands: Commands) {
    let dt = time.delta_secs();

    for (mut emitter, pos, bolt_base_damage_opt, damage_stack_opt) in &mut query {
        emitter.timer -= dt;
        if emitter.timer <= 0.0 {
            emitter.timer += emitter.interval;

            let stacks_f32 = emitter.stacks.saturating_sub(1) as f32;
            let max_radius = emitter
                .range_per_level
                .mul_add(stacks_f32, emitter.base_range);
            let base_damage = bolt_base_damage_opt.map_or(DEFAULT_BOLT_BASE_DAMAGE, |d| d.0);
            let damage_mult = damage_stack_opt.map_or(1.0, EffectStack::aggregate);

            commands.spawn((
                PulseRing,
                PulseRingRadius(0.0),
                PulseRingMaxRadius(max_radius),
                PulseRingSpeed(emitter.speed),
                PulseRingDamaged(HashSet::new()),
                PulseRingBaseDamage(base_damage),
                PulseRingDamageMultiplier(damage_mult),
                Position2D(pos.0),
                emitter.source_chip.clone(),
                CleanupOnExit::<NodeState>::default(),
            ));
        }
    }
}

/// Expands pulse ring radius each frame based on speed.
pub(crate) fn tick_pulse_ring(
    mut query: Query<(&mut PulseRingRadius, &PulseRingSpeed)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut radius, speed) in &mut query {
        radius.0 = speed.0.mul_add(dt, radius.0);
    }
}

/// Applies damage to cells within the expanding pulse ring radius.
pub(crate) fn apply_pulse_damage(
    mut pulse_query: PulseRingQuery,
    cell_query: AliveCellQuery,
    mut damage_writer: MessageWriter<DamageDealt<Cell>>,
) {
    for (ring_entity, ring_pos, ring_radius, base_dmg, dmg_mult, mut damaged, chip) in
        &mut pulse_query
    {
        for (cell_entity, cell_pos) in &cell_query {
            if damaged.0.contains(&cell_entity) {
                continue;
            }
            let distance = ring_pos.0.distance(cell_pos.0);
            if distance <= ring_radius.0 {
                damaged.0.insert(cell_entity);
                let damage = base_dmg.0 * dmg_mult.0;
                damage_writer.write(DamageDealt {
                    dealer:      Some(ring_entity),
                    target:      cell_entity,
                    amount:      damage,
                    source_chip: chip.and_then(|c| c.0.clone()),
                    _marker:     std::marker::PhantomData,
                });
            }
        }
    }
}

/// Despawns pulse rings that have reached their maximum radius.
pub(crate) fn despawn_finished_pulse_ring(
    query: Query<(Entity, &PulseRingRadius, &PulseRingMaxRadius)>,
    mut commands: Commands,
) {
    for (entity, radius, max_radius) in &query {
        if radius.0 >= max_radius.0 {
            commands.entity(entity).despawn();
        }
    }
}
