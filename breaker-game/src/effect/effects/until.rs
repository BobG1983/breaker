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
        effects::{damage_boost::ActiveDamageBoosts, speed_boost::ActiveSpeedBoosts},
    },
};

/// Query for entities with timed Until buffs, velocity, and active boost tracking.
type UntilTimerQuery = (
    Entity,
    &'static mut UntilTimers,
    Option<&'static mut Velocity2D>,
    Option<&'static mut ActiveSpeedBoosts>,
    Option<&'static mut ActiveDamageBoosts>,
);

/// Query for entities with trigger-based Until buffs, velocity, and active boost tracking.
type UntilTriggerQuery = (
    Entity,
    &'static mut UntilTriggers,
    Option<&'static mut Velocity2D>,
    Option<&'static mut ActiveSpeedBoosts>,
    Option<&'static mut ActiveDamageBoosts>,
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

/// Reverses applicable child effects on an entity.
///
/// - `SpeedBoost`: if [`ActiveSpeedBoosts`] is present, removes one matching
///   multiplier entry (letting [`apply_speed_boosts`] recalculate velocity).
///   Otherwise falls back to dividing `Velocity2D` by the multiplier.
/// - `DamageBoost`: if [`ActiveDamageBoosts`] is present, removes one matching
///   multiplier entry. Otherwise removes the `DamageBoost` component.
/// - All other effects: no-op (fire-and-forget).
fn reverse_children(
    entity: Entity,
    children: &[EffectNode],
    mut velocity: Option<&mut Velocity2D>,
    mut active_speed_boosts: Option<&mut ActiveSpeedBoosts>,
    mut active_damage_boosts: Option<&mut ActiveDamageBoosts>,
    commands: &mut Commands,
) {
    for child in children {
        match child {
            EffectNode::Do(Effect::SpeedBoost { multiplier, .. }) => {
                if let Some(ref mut boosts) = active_speed_boosts {
                    let pos = boosts
                        .0
                        .iter()
                        .position(|&m| (m - multiplier).abs() < f32::EPSILON);
                    if let Some(idx) = pos {
                        boosts.0.swap_remove(idx);
                    }
                } else if let Some(ref mut vel) = velocity {
                    vel.0 /= *multiplier;
                }
            }
            EffectNode::Do(Effect::DamageBoost(multiplier)) => {
                if let Some(ref mut boosts) = active_damage_boosts {
                    let pos = boosts
                        .0
                        .iter()
                        .position(|&m| (m - multiplier).abs() < f32::EPSILON);
                    if let Some(idx) = pos {
                        boosts.0.swap_remove(idx);
                    }
                } else {
                    commands.entity(entity).remove::<DamageBoost>();
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
    for (entity, mut timers, mut velocity, mut active_speed, mut active_damage) in &mut query {
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
            reverse_children(
                entity,
                &entry.children,
                velocity.as_deref_mut(),
                active_speed.as_deref_mut(),
                active_damage.as_deref_mut(),
                &mut commands,
            );
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

    for (entity, mut triggers, mut velocity, mut active_speed, mut active_damage) in &mut query {
        triggers.0.retain(|entry| {
            let matched = match &entry.trigger {
                Trigger::OnImpact(ImpactTarget::Cell) => cell_hit_bolts.contains(&entity),
                Trigger::OnImpact(ImpactTarget::Wall) => wall_hit_bolts.contains(&entity),
                Trigger::OnImpact(ImpactTarget::Breaker) => breaker_hit_bolts.contains(&entity),
                Trigger::OnCellDestroyed => cell_destroyed_count > 0,
                _ => false,
            };
            if matched {
                reverse_children(
                    entity,
                    &entry.children,
                    velocity.as_deref_mut(),
                    active_speed.as_deref_mut(),
                    active_damage.as_deref_mut(),
                    &mut commands,
                );
                false // Remove the entry
            } else {
                true // Keep the entry
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use rantzsoft_spatial2d::components::Velocity2D;

    use super::*;
    use crate::{
        bolt::components::{Bolt, BoltBaseSpeed, BoltMaxSpeed},
        chips::components::DamageBoost,
        effect::{
            definition::{Effect, EffectNode, ImpactTarget, Target, Trigger},
            effects::{
                damage_boost::ActiveDamageBoosts,
                speed_boost::{ActiveSpeedBoosts, apply_speed_boosts},
            },
        },
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // =========================================================================
    // Sub-Feature A: Until Node — tick_until_timers (behaviors 5-6, 9-11a)
    // =========================================================================

    // --- Behavior 5: tick_until_timers decrements remaining each tick ---

    #[test]
    fn tick_until_timers_decrements_remaining_by_dt() {
        let mut app = test_app();
        app.add_systems(FixedUpdate, tick_until_timers);

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 800.0)),
                BoltBaseSpeed(400.0),
                BoltMaxSpeed(800.0),
                UntilTimers(vec![UntilTimerEntry {
                    remaining: 3.0,
                    children: vec![EffectNode::Do(Effect::SpeedBoost {
                        target: Target::Bolt,
                        multiplier: 2.0,
                    })],
                }]),
            ))
            .id();

        tick(&mut app);

        let timers = app.world().get::<UntilTimers>(bolt).unwrap();
        let dt = 1.0 / 64.0; // MinimalPlugins default fixed timestep
        let expected = 3.0 - dt;
        assert!(
            (timers.0[0].remaining - expected).abs() < f32::EPSILON,
            "remaining should be {expected}, got {}",
            timers.0[0].remaining
        );
    }

    // --- Behavior 6: tick_until_timers reverses SpeedBoost on expiry ---

    #[test]
    fn tick_until_timers_reverses_speed_boost_on_expiry() {
        let mut app = test_app();
        app.add_systems(FixedUpdate, tick_until_timers);

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 800.0)),
                BoltBaseSpeed(400.0),
                BoltMaxSpeed(800.0),
                UntilTimers(vec![UntilTimerEntry {
                    remaining: 0.01, // Will expire on next tick (dt = 1/64 > 0.01)
                    children: vec![EffectNode::Do(Effect::SpeedBoost {
                        target: Target::Bolt,
                        multiplier: 2.0,
                    })],
                }]),
            ))
            .id();

        tick(&mut app);

        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        // Reversal: 800.0 / 2.0 = 400.0
        assert!(
            (vel.0.y - 400.0).abs() < 1.0,
            "velocity should be reversed to 400.0, got {}",
            vel.0.y
        );

        let timers = app.world().get::<UntilTimers>(bolt);
        // The entry should be removed. Since it was the only entry, UntilTimers
        // component should be removed entirely (behavior 11a).
        assert!(
            timers.is_none(),
            "UntilTimers component should be removed when empty"
        );
    }

    // --- Behavior 8: Until removal with OnImpact(Cell) replaces OneShotDamageBoost ---

    #[test]
    fn check_until_triggers_removes_damage_boost_on_cell_impact() {
        use crate::bolt::messages::BoltHitCell;

        #[derive(Resource)]
        struct SendMsg(Option<BoltHitCell>);

        fn enqueue(msg: Res<SendMsg>, mut writer: MessageWriter<BoltHitCell>) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        let mut app = test_app();
        app.add_message::<BoltHitCell>();
        app.add_systems(FixedUpdate, (enqueue, check_until_triggers).chain());

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 400.0)),
                DamageBoost(2.0),
                UntilTriggers(vec![UntilTriggerEntry {
                    trigger: Trigger::OnImpact(ImpactTarget::Cell),
                    children: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
                }]),
            ))
            .id();

        let cell = app.world_mut().spawn_empty().id();
        app.insert_resource(SendMsg(Some(BoltHitCell { cell, bolt })));

        tick(&mut app);

        // DamageBoost component should be removed
        assert!(
            app.world().get::<DamageBoost>(bolt).is_none(),
            "DamageBoost should be removed after OnImpact(Cell) trigger"
        );

        // UntilTriggers entry should be removed
        let triggers = app.world().get::<UntilTriggers>(bolt);
        assert!(
            triggers.is_none() || triggers.unwrap().0.is_empty(),
            "UntilTriggers entry should be removed after trigger match"
        );
    }

    // --- Behavior 7: Until removal with OnImpact(Breaker) ---

    #[test]
    fn check_until_triggers_reverses_speed_boost_on_breaker_impact() {
        use crate::bolt::messages::BoltHitBreaker;

        #[derive(Resource)]
        struct SendMsg(Option<BoltHitBreaker>);

        fn enqueue(msg: Res<SendMsg>, mut writer: MessageWriter<BoltHitBreaker>) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        let mut app = test_app();
        app.add_message::<BoltHitBreaker>();
        app.add_systems(FixedUpdate, (enqueue, check_until_triggers).chain());

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 600.0)),
                BoltBaseSpeed(400.0),
                BoltMaxSpeed(800.0),
                UntilTriggers(vec![UntilTriggerEntry {
                    trigger: Trigger::OnImpact(ImpactTarget::Breaker),
                    children: vec![EffectNode::Do(Effect::SpeedBoost {
                        target: Target::Bolt,
                        multiplier: 1.5,
                    })],
                }]),
            ))
            .id();

        app.insert_resource(SendMsg(Some(BoltHitBreaker { bolt })));

        tick(&mut app);

        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        // Reversal: 600.0 / 1.5 = 400.0
        assert!(
            (vel.0.y - 400.0).abs() < 1.0,
            "velocity should be reversed to 400.0, got {}",
            vel.0.y
        );
    }

    // --- Behavior 7 edge case: OnImpact(Cell) does NOT trigger removal of OnImpact(Breaker) ---

    #[test]
    fn check_until_triggers_cell_impact_does_not_remove_breaker_until() {
        use crate::bolt::messages::BoltHitCell;

        #[derive(Resource)]
        struct SendMsg(Option<BoltHitCell>);

        fn enqueue(msg: Res<SendMsg>, mut writer: MessageWriter<BoltHitCell>) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        let mut app = test_app();
        app.add_message::<BoltHitCell>();
        app.add_systems(FixedUpdate, (enqueue, check_until_triggers).chain());

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 600.0)),
                BoltBaseSpeed(400.0),
                BoltMaxSpeed(800.0),
                UntilTriggers(vec![UntilTriggerEntry {
                    trigger: Trigger::OnImpact(ImpactTarget::Breaker),
                    children: vec![EffectNode::Do(Effect::SpeedBoost {
                        target: Target::Bolt,
                        multiplier: 1.5,
                    })],
                }]),
            ))
            .id();

        let cell = app.world_mut().spawn_empty().id();
        app.insert_resource(SendMsg(Some(BoltHitCell { cell, bolt })));

        tick(&mut app);

        // UntilTriggers should still have the breaker entry
        let triggers = app.world().get::<UntilTriggers>(bolt).unwrap();
        assert_eq!(
            triggers.0.len(),
            1,
            "OnImpact(Cell) should NOT trigger removal of OnImpact(Breaker) entry"
        );

        // Velocity should be unchanged
        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        assert!(
            (vel.0.y - 600.0).abs() < f32::EPSILON,
            "velocity should be unchanged, got {}",
            vel.0.y
        );
    }

    // --- Behavior 10: Non-reversible effect (SpawnBolts) — reversal is no-op ---

    #[test]
    fn tick_until_timers_non_reversible_effect_is_noop_on_expiry() {
        let mut app = test_app();
        app.add_systems(FixedUpdate, tick_until_timers);

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 400.0)),
                UntilTimers(vec![UntilTimerEntry {
                    remaining: 0.01,
                    children: vec![EffectNode::Do(Effect::SpawnBolts {
                        count: 1,
                        lifespan: None,
                        inherit: false,
                    })],
                }]),
            ))
            .id();

        tick(&mut app);

        // Timer entry should be removed. No panic, no velocity change.
        let timers = app.world().get::<UntilTimers>(bolt);
        assert!(
            timers.is_none(),
            "UntilTimers should be removed after expiry of non-reversible effect"
        );

        // Velocity should be unchanged (no reversal for SpawnBolts)
        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        assert!(
            (vel.0.y - 400.0).abs() < f32::EPSILON,
            "velocity should be unchanged for non-reversible effect, got {}",
            vel.0.y
        );
    }

    // --- Behavior 11a: Empty UntilTimers component is removed ---

    #[test]
    fn tick_until_timers_removes_component_when_last_entry_expires() {
        let mut app = test_app();
        app.add_systems(FixedUpdate, tick_until_timers);

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 400.0)),
                UntilTimers(vec![UntilTimerEntry {
                    remaining: 0.01,
                    children: vec![EffectNode::Do(Effect::SpawnBolts {
                        count: 1,
                        lifespan: None,
                        inherit: false,
                    })],
                }]),
            ))
            .id();

        tick(&mut app);

        assert!(
            app.world().get::<UntilTimers>(bolt).is_none(),
            "UntilTimers component should be removed when all entries expire"
        );
    }

    // --- Behavior 9: Multiple buffs stack multiplicatively, removed independently ---

    #[test]
    fn tick_until_timers_removes_only_expired_entry_independently() {
        let mut app = test_app();
        app.add_systems(FixedUpdate, tick_until_timers);

        // Two Until buffs: SpeedBoost(2.0) expiring at 0.01 and SpeedBoost(1.5) expiring at 5.0
        // Base 400 * 2.0 * 1.5 = 1200, clamped to 800. When 2.0x expires: 800 / 2.0 = 400
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 800.0)),
                BoltBaseSpeed(400.0),
                BoltMaxSpeed(800.0),
                UntilTimers(vec![
                    UntilTimerEntry {
                        remaining: 0.01, // Expires this tick
                        children: vec![EffectNode::Do(Effect::SpeedBoost {
                            target: Target::Bolt,
                            multiplier: 2.0,
                        })],
                    },
                    UntilTimerEntry {
                        remaining: 5.0, // Still active
                        children: vec![EffectNode::Do(Effect::SpeedBoost {
                            target: Target::Bolt,
                            multiplier: 1.5,
                        })],
                    },
                ]),
            ))
            .id();

        tick(&mut app);

        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        // Only the 2.0x buff expired: 800 / 2.0 = 400.0
        assert!(
            (vel.0.y - 400.0).abs() < 1.0,
            "velocity should be 400.0 after 2.0x buff expires, got {}",
            vel.0.y
        );

        // One entry should remain
        let timers = app.world().get::<UntilTimers>(bolt).unwrap();
        assert_eq!(timers.0.len(), 1, "only the unexpired entry should remain");
    }

    // =========================================================================
    // BLOCKING: Behavior 11 — check_until_triggers reads all message types
    // =========================================================================

    // --- check_until_triggers removes on wall impact ---

    #[test]
    fn check_until_triggers_removes_on_wall_impact() {
        use crate::bolt::messages::BoltHitWall;

        #[derive(Resource)]
        struct SendMsg(Option<BoltHitWall>);

        fn enqueue(msg: Res<SendMsg>, mut writer: MessageWriter<BoltHitWall>) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        let mut app = test_app();
        app.add_message::<BoltHitWall>();
        app.add_systems(FixedUpdate, (enqueue, check_until_triggers).chain());

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 520.0)),
                BoltBaseSpeed(400.0),
                BoltMaxSpeed(800.0),
                UntilTriggers(vec![UntilTriggerEntry {
                    trigger: Trigger::OnImpact(ImpactTarget::Wall),
                    children: vec![EffectNode::Do(Effect::SpeedBoost {
                        target: Target::Bolt,
                        multiplier: 1.3,
                    })],
                }]),
            ))
            .id();

        app.insert_resource(SendMsg(Some(BoltHitWall { bolt })));

        tick(&mut app);

        // UntilTriggers entry should be removed
        let triggers = app.world().get::<UntilTriggers>(bolt);
        assert!(
            triggers.is_none() || triggers.unwrap().0.is_empty(),
            "UntilTriggers entry should be removed after OnImpact(Wall) trigger"
        );

        // Velocity should be divided by multiplier: 520.0 / 1.3 = 400.0
        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        assert!(
            (vel.0.y - 400.0).abs() < 1.0,
            "velocity should be reversed to 400.0, got {}",
            vel.0.y
        );
    }

    // --- check_until_triggers removes on cell destroyed ---

    #[test]
    fn check_until_triggers_removes_on_cell_destroyed() {
        use crate::cells::messages::CellDestroyedAt;

        #[derive(Resource)]
        struct SendMsg(Option<CellDestroyedAt>);

        fn enqueue(msg: Res<SendMsg>, mut writer: MessageWriter<CellDestroyedAt>) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        let mut app = test_app();
        app.add_message::<CellDestroyedAt>();
        app.add_systems(FixedUpdate, (enqueue, check_until_triggers).chain());

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 600.0)),
                BoltBaseSpeed(400.0),
                BoltMaxSpeed(800.0),
                UntilTriggers(vec![UntilTriggerEntry {
                    trigger: Trigger::OnCellDestroyed,
                    children: vec![EffectNode::Do(Effect::SpeedBoost {
                        target: Target::Bolt,
                        multiplier: 1.5,
                    })],
                }]),
            ))
            .id();

        app.insert_resource(SendMsg(Some(CellDestroyedAt {
            position: Vec2::ZERO,
            was_required_to_clear: true,
        })));

        tick(&mut app);

        // UntilTriggers entry should be removed
        let triggers = app.world().get::<UntilTriggers>(bolt);
        assert!(
            triggers.is_none() || triggers.unwrap().0.is_empty(),
            "UntilTriggers entry should be removed after OnCellDestroyed trigger"
        );

        // Velocity should be divided by multiplier: 600.0 / 1.5 = 400.0
        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        assert!(
            (vel.0.y - 400.0).abs() < 1.0,
            "velocity should be reversed to 400.0, got {}",
            vel.0.y
        );
    }

    // =========================================================================
    // IMPORTANT: Behavior 9 edge — both buffs expire same frame
    // =========================================================================

    #[test]
    fn tick_until_timers_both_expire_same_frame_divides_sequentially() {
        let mut app = test_app();
        app.add_systems(FixedUpdate, tick_until_timers);

        // Two buffs both expiring on the same tick:
        // SpeedBoost(2.0) and SpeedBoost(1.5)
        // Starting velocity 600.0: 600.0 / 2.0 / 1.5 = 200.0
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 600.0)),
                BoltBaseSpeed(200.0),
                BoltMaxSpeed(800.0),
                UntilTimers(vec![
                    UntilTimerEntry {
                        remaining: 0.01, // dt = 1/64 > 0.01 — expires this tick
                        children: vec![EffectNode::Do(Effect::SpeedBoost {
                            target: Target::Bolt,
                            multiplier: 2.0,
                        })],
                    },
                    UntilTimerEntry {
                        remaining: 0.01, // Also expires this tick
                        children: vec![EffectNode::Do(Effect::SpeedBoost {
                            target: Target::Bolt,
                            multiplier: 1.5,
                        })],
                    },
                ]),
            ))
            .id();

        tick(&mut app);

        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        // Both expired: 600.0 / 2.0 / 1.5 = 200.0
        assert!(
            (vel.0.y - 200.0).abs() < 1.0,
            "velocity should be 200.0 after both buffs expire, got {}",
            vel.0.y
        );

        // Both entries removed — component should be gone
        let timers = app.world().get::<UntilTimers>(bolt);
        assert!(
            timers.is_none(),
            "UntilTimers should be removed when all entries expire"
        );
    }

    // =========================================================================
    // IMPORTANT: Behavior 6 edge — remaining exactly 0.0
    // =========================================================================

    #[test]
    fn tick_until_timers_remaining_zero_triggers_removal_on_next_tick() {
        let mut app = test_app();
        app.add_systems(FixedUpdate, tick_until_timers);

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 800.0)),
                BoltBaseSpeed(400.0),
                BoltMaxSpeed(800.0),
                UntilTimers(vec![UntilTimerEntry {
                    remaining: 0.0, // Exactly zero — should expire on next tick
                    children: vec![EffectNode::Do(Effect::SpeedBoost {
                        target: Target::Bolt,
                        multiplier: 2.0,
                    })],
                }]),
            ))
            .id();

        tick(&mut app);

        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        // Reversal: 800.0 / 2.0 = 400.0
        assert!(
            (vel.0.y - 400.0).abs() < 1.0,
            "velocity should be reversed to 400.0, got {}",
            vel.0.y
        );

        // Entry should be removed
        let timers = app.world().get::<UntilTimers>(bolt);
        assert!(
            timers.is_none(),
            "UntilTimers should be removed after zero-remaining entry expires"
        );
    }

    // =========================================================================
    // Nested When/Once children inside Until entries
    // =========================================================================

    // --- Behavior: Timer expiry removes entry with nested When child ---

    #[test]
    fn tick_until_timers_nested_when_removed_on_expiry() {
        let mut app = test_app();
        app.add_systems(FixedUpdate, tick_until_timers);

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 400.0)),
                UntilTimers(vec![UntilTimerEntry {
                    remaining: 0.01, // dt = 1/64 > 0.01 — expires this tick
                    children: vec![EffectNode::When {
                        trigger: Trigger::OnImpact(ImpactTarget::Cell),
                        then: vec![EffectNode::Do(Effect::Shockwave {
                            base_range: 64.0,
                            range_per_level: 0.0,
                            stacks: 1,
                            speed: 400.0,
                        })],
                    }],
                }]),
            ))
            .id();

        tick(&mut app);

        // No reversal needed for When nodes — they're just removed.
        // UntilTimers component should be removed (empty after removal).
        assert!(
            app.world().get::<UntilTimers>(bolt).is_none(),
            "UntilTimers should be removed when nested When child's timer expires"
        );

        // Velocity should be unchanged (When is not a reversible effect)
        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        assert!(
            (vel.0.y - 400.0).abs() < f32::EPSILON,
            "velocity should be unchanged after When child removal, got {}",
            vel.0.y
        );
    }

    // --- Behavior: Timer expiry with mixed Do + When children ---

    #[test]
    fn tick_until_timers_mixed_children_reverses_do_ignores_when() {
        let mut app = test_app();
        app.add_systems(FixedUpdate, tick_until_timers);

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 600.0)),
                BoltBaseSpeed(400.0),
                BoltMaxSpeed(800.0),
                UntilTimers(vec![UntilTimerEntry {
                    remaining: 0.01, // Expires this tick
                    children: vec![
                        EffectNode::Do(Effect::SpeedBoost {
                            target: Target::Bolt,
                            multiplier: 1.5,
                        }),
                        EffectNode::When {
                            trigger: Trigger::OnImpact(ImpactTarget::Cell),
                            then: vec![EffectNode::Do(Effect::Shockwave {
                                base_range: 48.0,
                                range_per_level: 0.0,
                                stacks: 1,
                                speed: 400.0,
                            })],
                        },
                    ],
                }]),
            ))
            .id();

        tick(&mut app);

        // SpeedBoost reversed: 600.0 / 1.5 = 400.0
        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        assert!(
            (vel.0.y - 400.0).abs() < 1.0,
            "SpeedBoost should be reversed to 400.0, got {}",
            vel.0.y
        );

        // Entry removed — When node is simply gone (no reversal needed)
        assert!(
            app.world().get::<UntilTimers>(bolt).is_none(),
            "UntilTimers should be removed after mixed children entry expires"
        );
    }

    // --- Behavior: Trigger match removes entry with nested When child ---

    #[test]
    fn check_until_triggers_nested_when_removed_on_trigger() {
        use crate::bolt::messages::BoltHitBreaker;

        #[derive(Resource)]
        struct SendMsg(Option<BoltHitBreaker>);

        fn enqueue(msg: Res<SendMsg>, mut writer: MessageWriter<BoltHitBreaker>) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        let mut app = test_app();
        app.add_message::<BoltHitBreaker>();
        app.add_systems(FixedUpdate, (enqueue, check_until_triggers).chain());

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 400.0)),
                UntilTriggers(vec![UntilTriggerEntry {
                    trigger: Trigger::OnImpact(ImpactTarget::Breaker),
                    children: vec![EffectNode::When {
                        trigger: Trigger::OnImpact(ImpactTarget::Cell),
                        then: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
                    }],
                }]),
            ))
            .id();

        app.insert_resource(SendMsg(Some(BoltHitBreaker { bolt })));

        tick(&mut app);

        // UntilTriggers entry should be removed — the nested When is gone.
        let triggers = app.world().get::<UntilTriggers>(bolt);
        assert!(
            triggers.is_none() || triggers.unwrap().0.is_empty(),
            "UntilTriggers entry should be removed after OnImpact(Breaker) trigger"
        );
    }

    // =========================================================================
    // Vec-based Until reversal — removes entries from ActiveSpeedBoosts/
    // ActiveDamageBoosts instead of dividing velocity / removing component
    // =========================================================================

    // --- Test 11: Timer expiry removes speed boost entry from vec ---

    #[test]
    fn tick_until_timers_removes_speed_boost_from_vec_on_expiry() {
        let mut app = test_app();
        app.add_systems(FixedUpdate, (tick_until_timers, apply_speed_boosts).chain());

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 1200.0)),
                BoltBaseSpeed(400.0),
                BoltMaxSpeed(2000.0),
                ActiveSpeedBoosts(vec![1.5, 2.0]),
                UntilTimers(vec![UntilTimerEntry {
                    remaining: 0.01, // dt = 1/64 > 0.01 — expires this tick
                    children: vec![EffectNode::Do(Effect::SpeedBoost {
                        target: Target::Bolt,
                        multiplier: 1.5,
                    })],
                }]),
            ))
            .id();

        tick(&mut app);

        // The 1.5 entry should be removed, leaving [2.0]
        let boosts = app
            .world()
            .get::<ActiveSpeedBoosts>(bolt)
            .expect("bolt should have ActiveSpeedBoosts");
        assert_eq!(
            boosts.0,
            vec![2.0],
            "ActiveSpeedBoosts should be [2.0] after 1.5 expires, got {:?}",
            boosts.0
        );

        // Velocity should be recalculated: 400.0 * 2.0 = 800.0
        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        assert!(
            (vel.speed() - 800.0).abs() < 1.0,
            "velocity should be recalculated to 800.0 (400 * 2.0), got {:.1}",
            vel.speed()
        );
    }

    // --- Test 12: Trigger removes damage boost entry from vec ---

    #[test]
    fn check_until_triggers_removes_damage_boost_from_vec_on_trigger() {
        use crate::bolt::messages::BoltHitCell;

        #[derive(Resource)]
        struct SendCellMsg(Option<BoltHitCell>);

        fn enqueue_cell_msg(msg: Res<SendCellMsg>, mut writer: MessageWriter<BoltHitCell>) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        let mut app = test_app();
        app.add_message::<BoltHitCell>();
        app.add_systems(
            FixedUpdate,
            (enqueue_cell_msg, check_until_triggers).chain(),
        );

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 400.0)),
                ActiveDamageBoosts(vec![2.0, 1.5]),
                UntilTriggers(vec![UntilTriggerEntry {
                    trigger: Trigger::OnImpact(ImpactTarget::Cell),
                    children: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
                }]),
            ))
            .id();

        let cell = app.world_mut().spawn_empty().id();
        app.insert_resource(SendCellMsg(Some(BoltHitCell { cell, bolt })));

        tick(&mut app);

        // The 2.0 entry should be removed, leaving [1.5]
        let boosts = app
            .world()
            .get::<ActiveDamageBoosts>(bolt)
            .expect("bolt should have ActiveDamageBoosts");
        assert_eq!(
            boosts.0,
            vec![1.5],
            "ActiveDamageBoosts should be [1.5] after 2.0 is removed, got {:?}",
            boosts.0
        );
    }
}
