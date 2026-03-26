//! Ramping damage chip effect — accumulates bonus damage per cell hit, resets on breaker bounce.
//!
//! Observes [`RampingDamageApplied`] and inserts or stacks [`RampingDamageState`] on bolt entities.
//! [`increment_ramping_damage`] increases the bonus on cell hits.
//! [`reset_ramping_damage`] resets the bonus on non-bump breaker impacts.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::{
    bolt::{
        components::Bolt,
        messages::{BoltHitBreaker, BoltHitCell},
    },
    breaker::messages::BumpPerformed,
    effect::typed_events::RampingDamageApplied,
};

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Tracks ramping damage state on a bolt entity.
///
/// `current_bonus` starts at 0.0 and increases by `bonus_per_hit` on each cell hit,
/// capping at `max_bonus`. Resets to 0.0 when the bolt hits the breaker without a bump.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct RampingDamageState {
    /// Current accumulated damage bonus.
    pub current_bonus: f32,
    /// Damage bonus added per cell hit.
    pub bonus_per_hit: f32,
    /// Maximum cumulative damage bonus.
    pub max_bonus: f32,
}

// ---------------------------------------------------------------------------
// Observer — inserts / stacks RampingDamageState
// ---------------------------------------------------------------------------

/// Observer: handles ramping damage application — inserts or stacks
/// [`RampingDamageState`] on all bolt entities.
pub(crate) fn handle_ramping_damage(
    trigger: On<RampingDamageApplied>,
    mut query: Query<(Entity, Option<&mut RampingDamageState>), With<Bolt>>,
    mut commands: Commands,
) {
    let event = trigger.event();
    for (entity, existing) in &mut query {
        if let Some(mut state) = existing {
            state.bonus_per_hit += event.bonus_per_hit;
            state.max_bonus += event.max_bonus;
        } else {
            commands.entity(entity).insert(RampingDamageState {
                current_bonus: 0.0,
                bonus_per_hit: event.bonus_per_hit,
                max_bonus: event.max_bonus,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Increment system — increases current_bonus on cell hit
// ---------------------------------------------------------------------------

/// Increments `RampingDamageState::current_bonus` by `bonus_per_hit` for each
/// `BoltHitCell` message, clamping at `max_bonus`.
pub(crate) fn increment_ramping_damage(
    mut reader: MessageReader<BoltHitCell>,
    mut query: Query<&mut RampingDamageState>,
) {
    for msg in reader.read() {
        if let Ok(mut state) = query.get_mut(msg.bolt) {
            state.current_bonus = (state.current_bonus + state.bonus_per_hit).min(state.max_bonus);
        }
    }
}

// ---------------------------------------------------------------------------
// Reset system — resets current_bonus on non-bump breaker impact
// ---------------------------------------------------------------------------

/// Resets `RampingDamageState::current_bonus` to 0.0 when a bolt hits the breaker
/// without a corresponding `BumpPerformed` message for the same bolt.
pub(crate) fn reset_ramping_damage(
    mut breaker_reader: MessageReader<BoltHitBreaker>,
    mut bump_reader: MessageReader<BumpPerformed>,
    mut query: Query<&mut RampingDamageState>,
) {
    let bumped: HashSet<Entity> = bump_reader.read().filter_map(|msg| msg.bolt).collect();
    for msg in breaker_reader.read() {
        if !bumped.contains(&msg.bolt)
            && let Ok(mut state) = query.get_mut(msg.bolt)
        {
            state.current_bonus = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Test infrastructure ---

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_ramping_damage);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // --- Helper: enqueue messages via resource ---

    #[derive(Resource, Default)]
    struct TestBoltHitCell(Vec<BoltHitCell>);

    fn enqueue_bolt_hit_cell(
        msg_res: Res<TestBoltHitCell>,
        mut writer: MessageWriter<BoltHitCell>,
    ) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    #[derive(Resource, Default)]
    struct TestBoltHitBreaker(Vec<BoltHitBreaker>);

    fn enqueue_bolt_hit_breaker(
        msg_res: Res<TestBoltHitBreaker>,
        mut writer: MessageWriter<BoltHitBreaker>,
    ) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    #[derive(Resource, Default)]
    struct TestBumpPerformed(Vec<BumpPerformed>);

    fn enqueue_bump_performed(
        msg_res: Res<TestBumpPerformed>,
        mut writer: MessageWriter<BumpPerformed>,
    ) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    fn test_app_message() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_ramping_damage)
            .add_message::<BoltHitCell>()
            .add_message::<BoltHitBreaker>()
            .add_message::<BumpPerformed>()
            .init_resource::<TestBoltHitCell>()
            .init_resource::<TestBoltHitBreaker>()
            .init_resource::<TestBumpPerformed>()
            .add_systems(
                FixedUpdate,
                (
                    enqueue_bolt_hit_cell.before(increment_ramping_damage),
                    enqueue_bolt_hit_breaker.before(reset_ramping_damage),
                    enqueue_bump_performed.before(reset_ramping_damage),
                    increment_ramping_damage,
                    reset_ramping_damage,
                ),
            );
        app
    }

    // =========================================================================
    // Behavior 7: handle_ramping_damage inserts RampingDamageState on bolt
    // =========================================================================

    #[test]
    fn handle_ramping_damage_inserts_state_on_bolt() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn(Bolt).id();

        app.world_mut().commands().trigger(RampingDamageApplied {
            bonus_per_hit: 0.02,
            max_bonus: 0.2,
            max_stacks: 2,
            chip_name: "Basic Amp".to_owned(),
        });
        app.world_mut().flush();

        let state = app
            .world()
            .get::<RampingDamageState>(bolt)
            .expect("bolt should have RampingDamageState after RampingDamageApplied");
        assert!(
            (state.current_bonus - 0.0).abs() < f32::EPSILON,
            "current_bonus should be 0.0, got {}",
            state.current_bonus
        );
        assert!(
            (state.bonus_per_hit - 0.02).abs() < f32::EPSILON,
            "bonus_per_hit should be 0.02, got {}",
            state.bonus_per_hit
        );
        assert!(
            (state.max_bonus - 0.2).abs() < f32::EPSILON,
            "max_bonus should be 0.2, got {}",
            state.max_bonus
        );
    }

    // =========================================================================
    // Behavior 8: handle_ramping_damage stacks additively
    // =========================================================================

    #[test]
    fn handle_ramping_damage_stacks_additively() {
        let mut app = test_app();
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                RampingDamageState {
                    current_bonus: 0.0,
                    bonus_per_hit: 0.02,
                    max_bonus: 0.2,
                },
            ))
            .id();

        app.world_mut().commands().trigger(RampingDamageApplied {
            bonus_per_hit: 0.04,
            max_bonus: 0.4,
            max_stacks: 2,
            chip_name: "Potent Amp".to_owned(),
        });
        app.world_mut().flush();

        let state = app
            .world()
            .get::<RampingDamageState>(bolt)
            .expect("bolt should still have RampingDamageState");
        assert!(
            (state.bonus_per_hit - 0.06).abs() < 1e-6,
            "bonus_per_hit should be 0.06 (0.02 + 0.04), got {}",
            state.bonus_per_hit
        );
        assert!(
            (state.max_bonus - 0.6).abs() < 1e-6,
            "max_bonus should be 0.6 (0.2 + 0.4), got {}",
            state.max_bonus
        );
    }

    // =========================================================================
    // Behavior 9: increment_ramping_damage on cell hit
    // =========================================================================

    #[test]
    fn increment_ramping_damage_on_cell_hit() {
        let mut app = test_app_message();
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                RampingDamageState {
                    current_bonus: 0.0,
                    bonus_per_hit: 0.04,
                    max_bonus: 0.4,
                },
            ))
            .id();
        let cell = app.world_mut().spawn_empty().id();

        app.world_mut()
            .resource_mut::<TestBoltHitCell>()
            .0
            .push(BoltHitCell { cell, bolt });

        tick(&mut app);

        let state = app
            .world()
            .get::<RampingDamageState>(bolt)
            .expect("bolt should have RampingDamageState");
        assert!(
            (state.current_bonus - 0.04).abs() < f32::EPSILON,
            "current_bonus should be 0.04 after one cell hit, got {}",
            state.current_bonus
        );
    }

    // =========================================================================
    // Behavior 10: increment_ramping_damage clamps at max_bonus
    // =========================================================================

    #[test]
    fn increment_ramping_damage_clamps_at_max_bonus() {
        let mut app = test_app_message();
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                RampingDamageState {
                    current_bonus: 0.38,
                    bonus_per_hit: 0.04,
                    max_bonus: 0.4,
                },
            ))
            .id();
        let cell = app.world_mut().spawn_empty().id();

        app.world_mut()
            .resource_mut::<TestBoltHitCell>()
            .0
            .push(BoltHitCell { cell, bolt });

        tick(&mut app);

        let state = app
            .world()
            .get::<RampingDamageState>(bolt)
            .expect("bolt should have RampingDamageState");
        assert!(
            (state.current_bonus - 0.4).abs() < f32::EPSILON,
            "current_bonus should be clamped at 0.4, got {}",
            state.current_bonus
        );
    }

    // =========================================================================
    // Behavior 11: reset_ramping_damage on non-bump breaker impact
    // =========================================================================

    #[test]
    fn reset_ramping_damage_on_non_bump_breaker_impact() {
        let mut app = test_app_message();
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                RampingDamageState {
                    current_bonus: 0.2,
                    bonus_per_hit: 0.04,
                    max_bonus: 0.4,
                },
            ))
            .id();

        // Send BoltHitBreaker but NO BumpPerformed
        app.world_mut()
            .resource_mut::<TestBoltHitBreaker>()
            .0
            .push(BoltHitBreaker { bolt });

        tick(&mut app);

        let state = app
            .world()
            .get::<RampingDamageState>(bolt)
            .expect("bolt should have RampingDamageState");
        assert!(
            (state.current_bonus - 0.0).abs() < f32::EPSILON,
            "current_bonus should be reset to 0.0 on non-bump breaker impact, got {}",
            state.current_bonus
        );
        // bonus_per_hit and max_bonus should be unchanged
        assert!(
            (state.bonus_per_hit - 0.04).abs() < f32::EPSILON,
            "bonus_per_hit should be unchanged at 0.04, got {}",
            state.bonus_per_hit
        );
        assert!(
            (state.max_bonus - 0.4).abs() < f32::EPSILON,
            "max_bonus should be unchanged at 0.4, got {}",
            state.max_bonus
        );
    }

    // =========================================================================
    // Behavior 12: reset_ramping_damage does NOT reset when bump occurred
    // =========================================================================

    #[test]
    fn reset_ramping_damage_preserves_bonus_when_bump_performed() {
        let mut app = test_app_message();
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                RampingDamageState {
                    current_bonus: 0.2,
                    bonus_per_hit: 0.04,
                    max_bonus: 0.4,
                },
            ))
            .id();

        // Send BOTH BoltHitBreaker and BumpPerformed for the same bolt
        app.world_mut()
            .resource_mut::<TestBoltHitBreaker>()
            .0
            .push(BoltHitBreaker { bolt });
        app.world_mut()
            .resource_mut::<TestBumpPerformed>()
            .0
            .push(BumpPerformed {
                grade: crate::breaker::messages::BumpGrade::Perfect,
                bolt: Some(bolt),
            });

        tick(&mut app);

        let state = app
            .world()
            .get::<RampingDamageState>(bolt)
            .expect("bolt should have RampingDamageState");
        assert!(
            (state.current_bonus - 0.2).abs() < f32::EPSILON,
            "current_bonus should be preserved at 0.2 when bump was performed, got {}",
            state.current_bonus
        );
    }
}
