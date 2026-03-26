//! Until node tracking — timed and trigger-based removal of effect buffs.
//!
//! `UntilTimers` tracks effects with `TimeExpires` removal triggers.
//! `UntilTriggers` tracks effects with event-based removal triggers.
//! Both are per-entity components on bolt entities.

use bevy::{ecs::system::SystemParamValidationError, prelude::*};
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::messages::{BoltHitBreaker, BoltHitCell, BoltHitWall},
    cells::messages::CellDestroyedAt,
    chips::components::DamageBoost,
    effect::{
        definition::{Effect, EffectNode, ImpactTarget, Trigger},
        effects::{
            bolt_size_boost::ActiveSizeBoosts, bump_force_boost::ActiveBumpForces,
            damage_boost::ActiveDamageBoosts, piercing::ActivePiercings,
            speed_boost::ActiveSpeedBoosts,
        },
    },
};

/// Query for entities with timed Until buffs, velocity, and active boost tracking.
type UntilTimerQuery = (
    Entity,
    &'static mut UntilTimers,
    Option<&'static mut Velocity2D>,
    Option<&'static mut ActiveSpeedBoosts>,
    Option<&'static mut ActiveDamageBoosts>,
    Option<&'static mut ActivePiercings>,
    Option<&'static mut ActiveSizeBoosts>,
    Option<&'static mut ActiveBumpForces>,
);

/// Query for entities with trigger-based Until buffs, velocity, and active boost tracking.
type UntilTriggerQuery = (
    Entity,
    &'static mut UntilTriggers,
    Option<&'static mut Velocity2D>,
    Option<&'static mut ActiveSpeedBoosts>,
    Option<&'static mut ActiveDamageBoosts>,
    Option<&'static mut ActivePiercings>,
    Option<&'static mut ActiveSizeBoosts>,
    Option<&'static mut ActiveBumpForces>,
);

/// A single timed Until entry — an active buff that expires after a duration.
#[derive(Debug, Clone)]
pub(crate) struct UntilTimerEntry {
    /// Seconds remaining before this buff is removed.
    pub remaining: f32,
    /// The child effect nodes applied by this Until (for reversal on expiry).
    pub children: Vec<EffectNode>,
}

/// Per-entity tracking of active `TimeExpires` Until buffs.
#[derive(Component, Debug, Default, Clone)]
pub(crate) struct UntilTimers(pub Vec<UntilTimerEntry>);

/// A single trigger-based Until entry — an active buff removed on a trigger event.
#[derive(Debug, Clone)]
pub(crate) struct UntilTriggerEntry {
    /// The trigger that removes this buff.
    pub trigger: Trigger,
    /// The child effect nodes applied by this Until (for reversal on trigger).
    pub children: Vec<EffectNode>,
}

/// Per-entity tracking of trigger-based Until buffs.
#[derive(Component, Debug, Default, Clone)]
pub(crate) struct UntilTriggers(pub Vec<UntilTriggerEntry>);

/// Bundles mutable references to all active boost tracking components for
/// reversal. Passed as a single argument to [`reverse_children`] to stay
/// within clippy's `too_many_arguments` threshold.
struct ActiveBoosts<'a> {
    velocity: Option<&'a mut Velocity2D>,
    speed: Option<&'a mut ActiveSpeedBoosts>,
    damage: Option<&'a mut ActiveDamageBoosts>,
    piercings: Option<&'a mut ActivePiercings>,
    size: Option<&'a mut ActiveSizeBoosts>,
    bump_force: Option<&'a mut ActiveBumpForces>,
}

/// Reverses applicable child effects on an entity.
///
/// - `SpeedBoost`: if [`ActiveSpeedBoosts`] is present, removes one matching
///   multiplier entry (letting [`apply_speed_boosts`] recalculate velocity).
///   Otherwise falls back to dividing `Velocity2D` by the multiplier.
/// - `DamageBoost`: if [`ActiveDamageBoosts`] is present, removes one matching
///   multiplier entry. Otherwise removes the `DamageBoost` component.
/// - `Piercing`: if [`ActivePiercings`] is present, removes one matching
///   entry (letting `apply_active_piercings` recalculate).
/// - `SizeBoost`: if [`ActiveSizeBoosts`] is present, removes one matching
///   entry (letting `apply_active_size_boosts` recalculate).
/// - `BumpForce`: if [`ActiveBumpForces`] is present, removes one matching
///   entry (letting `apply_active_bump_forces` recalculate).
/// - All other effects: no-op (fire-and-forget).
fn reverse_children(
    entity: Entity,
    children: &[EffectNode],
    boosts: &mut ActiveBoosts,
    commands: &mut Commands,
) {
    for child in children {
        match child {
            EffectNode::Do(Effect::SpeedBoost { multiplier, .. }) => {
                if let Some(ref mut speed) = boosts.speed {
                    let pos = speed
                        .0
                        .iter()
                        .position(|&m| (m - multiplier).abs() < f32::EPSILON);
                    if let Some(idx) = pos {
                        speed.0.swap_remove(idx);
                    }
                } else if let Some(ref mut vel) = boosts.velocity {
                    vel.0 /= *multiplier;
                }
            }
            EffectNode::Do(Effect::DamageBoost(multiplier)) => {
                if let Some(ref mut damage) = boosts.damage {
                    let pos = damage
                        .0
                        .iter()
                        .position(|&m| (m - multiplier).abs() < f32::EPSILON);
                    if let Some(idx) = pos {
                        damage.0.swap_remove(idx);
                    }
                } else {
                    commands.entity(entity).remove::<DamageBoost>();
                }
            }
            EffectNode::Do(Effect::Piercing(count)) => {
                if let Some(ref mut piercings) = boosts.piercings {
                    let pos = piercings.0.iter().position(|&c| c == *count);
                    if let Some(idx) = pos {
                        piercings.0.swap_remove(idx);
                    }
                }
            }
            EffectNode::Do(Effect::SizeBoost(value)) => {
                if let Some(ref mut size) = boosts.size {
                    let pos = size
                        .0
                        .iter()
                        .position(|&v| (v - value).abs() < f32::EPSILON);
                    if let Some(idx) = pos {
                        size.0.swap_remove(idx);
                    }
                }
            }
            EffectNode::Do(Effect::BumpForce(value)) => {
                if let Some(ref mut forces) = boosts.bump_force {
                    let pos = forces
                        .0
                        .iter()
                        .position(|&v| (v - value).abs() < f32::EPSILON);
                    if let Some(idx) = pos {
                        forces.0.swap_remove(idx);
                    }
                }
            }
            _ => {
                // Non-reversible effect — no-op on removal.
            }
        }
    }
}

/// Decrements `UntilTimers` each fixed tick. When a timer expires, reverses
/// applicable child effects (removes from `ActiveSpeedBoosts`/`ActiveDamageBoosts`
/// vecs, or divides velocity / removes component as fallback).
/// Removes the `UntilTimers` component when empty.
pub(crate) fn tick_until_timers(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    mut query: Query<UntilTimerQuery>,
) {
    let dt = time.delta_secs();
    for (
        entity,
        mut timers,
        mut velocity,
        mut active_speed,
        mut active_damage,
        mut active_piercings,
        mut active_size,
        mut active_bump,
    ) in &mut query
    {
        let mut expired_indices = Vec::new();
        for (i, entry) in timers.0.iter_mut().enumerate() {
            entry.remaining -= dt;
            if entry.remaining <= 0.0 {
                expired_indices.push(i);
            }
        }
        // Process expired entries in reverse order to preserve indices.
        for &i in expired_indices.iter().rev() {
            let entry = timers.0.remove(i);
            let mut active = ActiveBoosts {
                velocity: velocity.as_deref_mut(),
                speed: active_speed.as_deref_mut(),
                damage: active_damage.as_deref_mut(),
                piercings: active_piercings.as_deref_mut(),
                size: active_size.as_deref_mut(),
                bump_force: active_bump.as_deref_mut(),
            };
            reverse_children(entity, &entry.children, &mut active, &mut commands);
        }
        if timers.0.is_empty() {
            commands.entity(entity).remove::<UntilTimers>();
        }
    }
}

/// Checks collision/destruction messages against `UntilTriggers` entries.
/// When a matching trigger fires, reverses applicable child effects and
/// removes the entry.
///
/// Message readers are wrapped in `Result` so the system works in tests that
/// register only a subset of the message types.
pub(crate) fn check_until_triggers(
    mut commands: Commands,
    bolt_hit_cell_reader: Result<MessageReader<BoltHitCell>, SystemParamValidationError>,
    bolt_hit_wall_reader: Result<MessageReader<BoltHitWall>, SystemParamValidationError>,
    bolt_hit_breaker_reader: Result<MessageReader<BoltHitBreaker>, SystemParamValidationError>,
    cell_destroyed_reader: Result<MessageReader<CellDestroyedAt>, SystemParamValidationError>,
    mut query: Query<UntilTriggerQuery>,
) {
    // Collect bolt entities that hit each impact target type.
    let cell_hit_bolts: Vec<Entity> = bolt_hit_cell_reader
        .map(|mut r| r.read().map(|m| m.bolt).collect())
        .unwrap_or_default();
    let wall_hit_bolts: Vec<Entity> = bolt_hit_wall_reader
        .map(|mut r| r.read().map(|m| m.bolt).collect())
        .unwrap_or_default();
    let breaker_hit_bolts: Vec<Entity> = bolt_hit_breaker_reader
        .map(|mut r| r.read().map(|m| m.bolt).collect())
        .unwrap_or_default();
    let cell_destroyed_count = cell_destroyed_reader.map_or(0, |mut r| r.read().count());

    for (
        entity,
        mut triggers,
        mut velocity,
        mut active_speed,
        mut active_damage,
        mut active_piercings,
        mut active_size,
        mut active_bump,
    ) in &mut query
    {
        triggers.0.retain(|entry| {
            let matched = match &entry.trigger {
                Trigger::Impact(ImpactTarget::Cell) => cell_hit_bolts.contains(&entity),
                Trigger::Impact(ImpactTarget::Wall) => wall_hit_bolts.contains(&entity),
                Trigger::Impact(ImpactTarget::Breaker) => breaker_hit_bolts.contains(&entity),
                Trigger::CellDestroyed => cell_destroyed_count > 0,
                _ => false,
            };
            if matched {
                let mut active = ActiveBoosts {
                    velocity: velocity.as_deref_mut(),
                    speed: active_speed.as_deref_mut(),
                    damage: active_damage.as_deref_mut(),
                    piercings: active_piercings.as_deref_mut(),
                    size: active_size.as_deref_mut(),
                    bump_force: active_bump.as_deref_mut(),
                };
                reverse_children(entity, &entry.children, &mut active, &mut commands);
                false // Remove the entry
            } else {
                true // Keep the entry
            }
        });
    }
}
