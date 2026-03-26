//! Bump force boost chip effect observer — adds flat bump force to breaker.

use bevy::prelude::*;

use super::stack_f32;
use crate::{breaker::components::Breaker, chips::components::BumpForceBoost};

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a bump force passive effect is applied via chip selection.
#[derive(Event, Clone, Debug)]
pub(crate) struct BumpForceApplied {
    /// Bump force increase per stack.
    pub per_stack: f32,
    /// Maximum number of stacks allowed.
    pub max_stacks: u32,
    /// Name of the chip that applied this effect.
    pub chip_name: String,
}

/// Query for breakers with optional bump force boost and active bump force tracking.
type BumpForceQuery = (
    Entity,
    Option<&'static mut BumpForceBoost>,
    Option<&'static mut ActiveBumpForces>,
);

/// Observer: applies bump force boost stacking to all breaker entities.
///
/// Also pushes the force boost value onto each breaker's [`ActiveBumpForces`] vec
/// (if present) so that Until reversal can remove individual entries.
pub(crate) fn handle_bump_force_boost(
    trigger: On<BumpForceApplied>,
    mut query: Query<BumpForceQuery, With<Breaker>>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let per_stack = event.per_stack;
    let max_stacks = event.max_stacks;
    for (entity, mut existing, mut active_forces) in &mut query {
        stack_f32(
            entity,
            existing.as_deref_mut().map(|c| &mut c.0),
            per_stack,
            max_stacks,
            &mut commands,
            BumpForceBoost,
        );
        if let Some(ref mut forces) = active_forces {
            forces.0.push(per_stack);
        }
    }
}

/// Per-breaker tracking of active bump force values from individual chip stacks.
///
/// Each entry is a force boost (e.g. 10.0 for +10 bump force). The
/// [`ActiveBumpForces::total`] method returns the sum of all entries, used
/// to recalculate `BumpForceBoost` when stacks change.
/// Until reversal removes entries from the vec.
#[derive(Component, Debug, Default, Clone, PartialEq)]
pub(crate) struct ActiveBumpForces(pub Vec<f32>);

impl ActiveBumpForces {
    /// Returns the total bump force boost: the sum of all active entries.
    ///
    /// Returns 0.0 if no entries are active (empty vec).
    pub fn total(&self) -> f32 {
        self.0.iter().sum()
    }
}

/// Recalculates `BumpForceBoost` from the sum of [`ActiveBumpForces`] entries.
///
/// Runs after bridge dispatch and Until reversal to keep bump force values
/// consistent with the active stack vec.
pub(crate) fn apply_active_bump_forces(
    mut query: Query<(&mut BumpForceBoost, &ActiveBumpForces), With<Breaker>>,
) {
    for (mut force, active) in &mut query {
        force.0 = active.total();
    }
}

/// Registers all observers and systems for the bump force boost effect.
pub(crate) fn register(app: &mut App) {
    use crate::{
        effect::{effect_nodes::until, sets::EffectSystems},
        shared::PlayingState,
    };

    app.add_observer(handle_bump_force_boost);

    // Bump force recalculation — after bridge and Until reversal
    app.add_systems(
        FixedUpdate,
        apply_active_bump_forces
            .after(EffectSystems::Bridge)
            .after(until::tick_until_timers)
            .after(until::check_until_triggers)
            .run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_bump_force_boost);
        app
    }

    #[test]
    fn inserts_bump_force_boost_on_breaker() {
        let mut app = test_app();
        let breaker = app.world_mut().spawn(Breaker).id();

        app.world_mut().commands().trigger(BumpForceApplied {
            per_stack: 10.0,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let b = app.world().entity(breaker).get::<BumpForceBoost>().unwrap();
        assert!((b.0 - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn stacks_bump_force_boost() {
        let mut app = test_app();
        let breaker = app.world_mut().spawn((Breaker, BumpForceBoost(10.0))).id();

        app.world_mut().commands().trigger(BumpForceApplied {
            per_stack: 10.0,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let b = app.world().entity(breaker).get::<BumpForceBoost>().unwrap();
        assert!(
            (b.0 - 20.0).abs() < f32::EPSILON,
            "BumpForceBoost should stack from 10.0 to 20.0, got {}",
            b.0
        );
    }

    #[test]
    fn respects_max_stacks_bump_force_boost() {
        let mut app = test_app();
        let breaker = app.world_mut().spawn((Breaker, BumpForceBoost(30.0))).id();

        app.world_mut().commands().trigger(BumpForceApplied {
            per_stack: 10.0,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let b = app.world().entity(breaker).get::<BumpForceBoost>().unwrap();
        assert!(
            (b.0 - 30.0).abs() < f32::EPSILON,
            "BumpForceBoost should not exceed max_stacks cap, got {}",
            b.0
        );
    }

    // =========================================================================
    // ActiveBumpForces — vec-based bump force management
    // =========================================================================

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // --- Test 1: handle_bump_force_boost pushes to ActiveBumpForces ---

    #[test]
    fn handle_bump_force_boost_pushes_to_active_bump_forces() {
        let mut app = test_app();
        let breaker = app
            .world_mut()
            .spawn((Breaker, ActiveBumpForces(vec![])))
            .id();

        app.world_mut().commands().trigger(BumpForceApplied {
            per_stack: 10.0,
            max_stacks: 5,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let forces = app
            .world()
            .entity(breaker)
            .get::<ActiveBumpForces>()
            .expect("breaker should have ActiveBumpForces");
        assert_eq!(
            forces.0,
            vec![10.0],
            "ActiveBumpForces should contain [10.0] after BumpForceApplied, got {:?}",
            forces.0
        );
    }

    // --- Test 2: ActiveBumpForces::total returns sum ---

    #[test]
    fn active_bump_forces_total_returns_sum() {
        let forces = ActiveBumpForces(vec![10.0, 15.0, 5.0]);
        let total = forces.total();
        assert!(
            (total - 30.0).abs() < f32::EPSILON,
            "total of [10.0, 15.0, 5.0] should be 30.0, got {total}"
        );
    }

    // --- Test 3: apply_active_bump_forces recalculates from vec ---

    #[test]
    fn apply_active_bump_forces_recalculates_from_vec() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, apply_active_bump_forces);

        let breaker = app
            .world_mut()
            .spawn((
                Breaker,
                BumpForceBoost(99.0),
                ActiveBumpForces(vec![10.0, 15.0]),
            ))
            .id();

        tick(&mut app);

        let b = app
            .world()
            .entity(breaker)
            .get::<BumpForceBoost>()
            .unwrap();
        assert!(
            (b.0 - 25.0).abs() < f32::EPSILON,
            "BumpForceBoost should be recalculated to 25.0 (sum of [10.0, 15.0]), got {}",
            b.0
        );
    }

    // --- Test 4: apply_active_bump_forces empty vec sets zero ---

    #[test]
    fn apply_active_bump_forces_empty_vec_sets_zero() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, apply_active_bump_forces);

        let breaker = app
            .world_mut()
            .spawn((
                Breaker,
                BumpForceBoost(20.0),
                ActiveBumpForces(vec![]),
            ))
            .id();

        tick(&mut app);

        let b = app
            .world()
            .entity(breaker)
            .get::<BumpForceBoost>()
            .unwrap();
        assert!(
            (b.0).abs() < f32::EPSILON,
            "BumpForceBoost should be 0.0 with empty ActiveBumpForces, got {}",
            b.0
        );
    }
}
