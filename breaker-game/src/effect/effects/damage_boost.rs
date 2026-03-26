//! Damage boost chip effect observer — multiplies bolt damage.
//!
//! [`ActiveDamageBoosts`] tracks a vec of multipliers on each bolt.
//! The [`ActiveDamageBoosts::multiplier`] method returns the product
//! of all entries, used at `DamageCell` write time to compute effective damage.

use bevy::prelude::*;

use super::stack_f32;
use crate::{
    bolt::components::Bolt, chips::components::DamageBoost,
    effect::typed_events::DamageBoostApplied,
};

/// Query for bolts with optional damage boost and active damage boost tracking.
type DamageBoostQuery = (
    Entity,
    Option<&'static mut DamageBoost>,
    Option<&'static mut ActiveDamageBoosts>,
);

/// Per-bolt tracking of active damage boost multipliers.
///
/// Each entry is a multiplier (e.g. 1.5 for 50% damage increase). The
/// effective damage multiplier is the product of all entries. Until
/// reversal removes entries from the vec.
#[derive(Component, Debug, Default, Clone, PartialEq)]
pub(crate) struct ActiveDamageBoosts(pub Vec<f32>);

impl ActiveDamageBoosts {
    /// Returns the effective damage multiplier: the product of all active boosts.
    ///
    /// Returns 1.0 if no boosts are active (empty vec).
    pub fn multiplier(&self) -> f32 {
        self.0.iter().product::<f32>()
    }
}

/// Observer: applies damage boost stacking to all bolt entities.
///
/// Also pushes the multiplier value onto each bolt's [`ActiveDamageBoosts`] vec
/// (if present) so that Until reversal can remove individual entries.
pub(crate) fn handle_damage_boost(
    trigger: On<DamageBoostApplied>,
    mut query: Query<DamageBoostQuery, With<Bolt>>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let per_stack = event.per_stack;
    let max_stacks = event.max_stacks;
    for (entity, mut existing, mut active_boosts) in &mut query {
        stack_f32(
            entity,
            existing.as_deref_mut().map(|c| &mut c.0),
            per_stack,
            max_stacks,
            &mut commands,
            DamageBoost,
        );
        if let Some(ref mut boosts) = active_boosts {
            boosts.0.push(per_stack);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_damage_boost);
        app
    }

    #[test]
    fn inserts_damage_boost_on_bolt() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn(Bolt).id();

        app.world_mut().commands().trigger(DamageBoostApplied {
            per_stack: 1.5,
            max_stacks: 2,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let d = app.world().entity(bolt).get::<DamageBoost>().unwrap();
        assert!((d.0 - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn stacks_damage_boost() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn((Bolt, DamageBoost(1.5))).id();

        app.world_mut().commands().trigger(DamageBoostApplied {
            per_stack: 1.5,
            max_stacks: 2,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let d = app.world().entity(bolt).get::<DamageBoost>().unwrap();
        assert!((d.0 - 3.0).abs() < f32::EPSILON);
    }

    // =========================================================================
    // ActiveDamageBoosts — vec-based damage boost management
    // =========================================================================

    // --- Test 8: handle_damage_boost pushes to ActiveDamageBoosts ---

    #[test]
    fn handle_damage_boost_pushes_to_active_damage_boosts() {
        let mut app = test_app();
        let bolt = app
            .world_mut()
            .spawn((Bolt, ActiveDamageBoosts(vec![])))
            .id();

        app.world_mut().commands().trigger(DamageBoostApplied {
            per_stack: 1.5,
            max_stacks: 5,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let boosts = app
            .world()
            .entity(bolt)
            .get::<ActiveDamageBoosts>()
            .expect("bolt should have ActiveDamageBoosts");
        assert_eq!(
            boosts.0,
            vec![1.5],
            "ActiveDamageBoosts should contain [1.5], got {:?}",
            boosts.0
        );
    }

    // --- Test 9: handle_damage_boost stacks in vec ---

    #[test]
    fn handle_damage_boost_stacks_in_vec() {
        let mut app = test_app();
        let bolt = app
            .world_mut()
            .spawn((Bolt, ActiveDamageBoosts(vec![1.5])))
            .id();

        app.world_mut().commands().trigger(DamageBoostApplied {
            per_stack: 2.0,
            max_stacks: 5,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let boosts = app
            .world()
            .entity(bolt)
            .get::<ActiveDamageBoosts>()
            .expect("bolt should have ActiveDamageBoosts");
        assert_eq!(
            boosts.0,
            vec![1.5, 2.0],
            "ActiveDamageBoosts should contain [1.5, 2.0], got {:?}",
            boosts.0
        );
    }

    // --- Test 10: ActiveDamageBoosts::multiplier returns product ---

    #[test]
    fn damage_multiplier_from_active_damage_boosts() {
        let boosts = ActiveDamageBoosts(vec![1.5, 2.0]);
        let mult = boosts.multiplier();
        assert!(
            (mult - 3.0).abs() < f32::EPSILON,
            "multiplier of [1.5, 2.0] should be 3.0, got {mult}"
        );
    }

    #[test]
    fn damage_multiplier_empty_vec_returns_one() {
        let boosts = ActiveDamageBoosts(vec![]);
        let mult = boosts.multiplier();
        assert!(
            (mult - 1.0).abs() < f32::EPSILON,
            "multiplier of empty vec should be 1.0, got {mult}"
        );
    }

    #[test]
    fn damage_multiplier_single_entry() {
        let boosts = ActiveDamageBoosts(vec![2.5]);
        let mult = boosts.multiplier();
        assert!(
            (mult - 2.5).abs() < f32::EPSILON,
            "multiplier of [2.5] should be 2.5, got {mult}"
        );
    }
}
