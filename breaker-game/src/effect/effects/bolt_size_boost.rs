//! Bolt size boost chip effect observer — increases bolt radius.

use bevy::prelude::*;

use super::stack_f32;
use crate::{bolt::components::Bolt, chips::components::BoltSizeBoost, effect::definition::Target};

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a size boost passive effect is applied via chip selection.
#[derive(Event, Clone, Debug)]
pub(crate) struct SizeBoostApplied {
    /// Which entity to apply the size change to.
    pub target: Target,
    /// Size boost per stack.
    pub per_stack: f32,
    /// Maximum number of stacks allowed.
    pub max_stacks: u32,
    /// Name of the chip that applied this effect.
    pub chip_name: String,
}

/// Query for bolts with optional size boost and active size boost tracking.
type SizeBoostQuery = (
    Entity,
    Option<&'static mut BoltSizeBoost>,
    Option<&'static mut ActiveSizeBoosts>,
);

/// Observer: applies bolt size boost stacking to all bolt entities.
///
/// Also pushes the size boost value onto each bolt's [`ActiveSizeBoosts`] vec
/// (if present) so that Until reversal can remove individual entries.
pub(crate) fn handle_bolt_size_boost(
    trigger: On<SizeBoostApplied>,
    mut query: Query<SizeBoostQuery, With<Bolt>>,
    mut commands: Commands,
) {
    let event = trigger.event();
    if event.target != crate::effect::definition::Target::Bolt {
        return;
    }
    let per_stack = event.per_stack;
    let max_stacks = event.max_stacks;
    for (entity, mut existing, mut active_boosts) in &mut query {
        stack_f32(
            entity,
            existing.as_deref_mut().map(|c| &mut c.0),
            per_stack,
            max_stacks,
            &mut commands,
            BoltSizeBoost,
        );
        if let Some(ref mut boosts) = active_boosts {
            boosts.0.push(per_stack);
        }
    }
}

/// Per-bolt tracking of active size boost values from individual chip stacks.
///
/// Each entry is a size boost fraction (e.g. 0.5 for 50% size increase). The
/// [`ActiveSizeBoosts::total`] method returns the sum of all entries, used
/// to recalculate `BoltSizeBoost` when stacks change.
/// Until reversal removes entries from the vec.
#[derive(Component, Debug, Default, Clone, PartialEq)]
pub(crate) struct ActiveSizeBoosts(pub Vec<f32>);

impl ActiveSizeBoosts {
    /// Returns the total size boost: the sum of all active entries.
    ///
    /// Returns 0.0 if no entries are active (empty vec).
    pub fn total(&self) -> f32 {
        self.0.iter().sum()
    }
}

/// Recalculates `BoltSizeBoost` from the sum of [`ActiveSizeBoosts`] entries.
///
/// Runs after bridge dispatch and Until reversal to keep size boost values
/// consistent with the active stack vec.
pub(crate) fn apply_active_size_boosts(
    mut query: Query<(&mut BoltSizeBoost, &ActiveSizeBoosts), With<Bolt>>,
) {
    for (mut size_boost, active) in &mut query {
        size_boost.0 = active.total();
    }
}

/// Registers all observers and systems for the bolt size boost effect.
pub(crate) fn register(app: &mut App) {
    use crate::{
        effect::{effect_nodes::until, sets::EffectSystems},
        shared::PlayingState,
    };

    app.add_observer(handle_bolt_size_boost);

    // Size boost recalculation — after bridge and Until reversal
    app.add_systems(
        FixedUpdate,
        apply_active_size_boosts
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
            .add_observer(handle_bolt_size_boost);
        app
    }

    #[test]
    fn inserts_bolt_size_boost_on_bolt() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn(Bolt).id();

        app.world_mut().commands().trigger(SizeBoostApplied {
            target: crate::effect::definition::Target::Bolt,
            per_stack: 0.5,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let s = app.world().entity(bolt).get::<BoltSizeBoost>().unwrap();
        assert!((s.0 - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn stacks_bolt_size_boost() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn((Bolt, BoltSizeBoost(0.5))).id();

        app.world_mut().commands().trigger(SizeBoostApplied {
            target: crate::effect::definition::Target::Bolt,
            per_stack: 0.5,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let s = app.world().entity(bolt).get::<BoltSizeBoost>().unwrap();
        assert!(
            (s.0 - 1.0).abs() < f32::EPSILON,
            "BoltSizeBoost should stack from 0.5 to 1.0, got {}",
            s.0
        );
    }

    #[test]
    fn ignores_breaker_target() {
        let mut app = test_app();
        app.world_mut().spawn(Bolt);

        app.world_mut().commands().trigger(SizeBoostApplied {
            target: crate::effect::definition::Target::Breaker,
            per_stack: 20.0,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        assert!(
            app.world_mut()
                .query::<&BoltSizeBoost>()
                .iter(app.world())
                .next()
                .is_none(),
            "handle_bolt_size_boost should ignore Target::Breaker"
        );
    }

    // =========================================================================
    // ActiveSizeBoosts — vec-based size boost management
    // =========================================================================

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // --- Test 1: handle_bolt_size_boost pushes to ActiveSizeBoosts ---

    #[test]
    fn handle_bolt_size_boost_pushes_to_active_size_boosts() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn((Bolt, ActiveSizeBoosts(vec![]))).id();

        app.world_mut().commands().trigger(SizeBoostApplied {
            target: crate::effect::definition::Target::Bolt,
            per_stack: 0.5,
            max_stacks: 5,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let boosts = app
            .world()
            .entity(bolt)
            .get::<ActiveSizeBoosts>()
            .expect("bolt should have ActiveSizeBoosts");
        assert_eq!(
            boosts.0,
            vec![0.5],
            "ActiveSizeBoosts should contain [0.5] after SizeBoostApplied, got {:?}",
            boosts.0
        );
    }

    // --- Test 2: ActiveSizeBoosts::total returns sum ---

    #[test]
    fn active_size_boosts_total_returns_sum() {
        let boosts = ActiveSizeBoosts(vec![0.5, 0.3, 0.2]);
        let total = boosts.total();
        assert!(
            (total - 1.0).abs() < f32::EPSILON,
            "total of [0.5, 0.3, 0.2] should be 1.0, got {total}"
        );
    }

    // --- Test 3: apply_active_size_boosts recalculates from vec ---

    #[test]
    fn apply_active_size_boosts_recalculates_from_vec() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, apply_active_size_boosts);

        let bolt = app
            .world_mut()
            .spawn((Bolt, BoltSizeBoost(99.0), ActiveSizeBoosts(vec![0.5, 0.3])))
            .id();

        tick(&mut app);

        let s = app.world().entity(bolt).get::<BoltSizeBoost>().unwrap();
        assert!(
            (s.0 - 0.8).abs() < f32::EPSILON,
            "BoltSizeBoost should be recalculated to 0.8 (sum of [0.5, 0.3]), got {}",
            s.0
        );
    }

    // --- Test 4: apply_active_size_boosts empty vec sets zero ---

    #[test]
    fn apply_active_size_boosts_empty_vec_sets_zero() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, apply_active_size_boosts);

        let bolt = app
            .world_mut()
            .spawn((Bolt, BoltSizeBoost(1.0), ActiveSizeBoosts(vec![])))
            .id();

        tick(&mut app);

        let s = app.world().entity(bolt).get::<BoltSizeBoost>().unwrap();
        assert!(
            (s.0).abs() < f32::EPSILON,
            "BoltSizeBoost should be 0.0 with empty ActiveSizeBoosts, got {}",
            s.0
        );
    }
}
