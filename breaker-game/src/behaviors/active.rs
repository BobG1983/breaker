//! `ActiveBehaviors` resource — runtime trigger→consequence lookup.

use bevy::prelude::*;

use super::definition::{BehaviorBinding, Consequence, Trigger};

/// Flattened trigger→consequence bindings built at run start from the archetype.
///
/// Bridge systems query this resource to look up which consequences to fire
/// for a given trigger.
#[derive(Resource, Debug, Default)]
pub struct ActiveBehaviors(pub Vec<(Trigger, Consequence)>);

impl ActiveBehaviors {
    /// Builds `ActiveBehaviors` by flattening multi-trigger bindings.
    ///
    /// Each binding with N triggers produces N `(trigger, consequence)` pairs.
    pub fn from_bindings(behaviors: &[BehaviorBinding]) -> Self {
        let mut bindings = Vec::new();
        for behavior in behaviors {
            for trigger in &behavior.triggers {
                bindings.push((trigger.clone(), behavior.consequence.clone()));
            }
        }
        Self(bindings)
    }
}

impl ActiveBehaviors {
    /// Returns all consequences bound to a given trigger.
    pub fn consequences_for(&self, trigger: Trigger) -> impl Iterator<Item = &Consequence> {
        self.0
            .iter()
            .filter(move |(t, _)| *t == trigger)
            .map(|(_, c)| c)
    }

    /// Whether any behaviors are bound to a given trigger.
    #[must_use]
    pub fn has_trigger(&self, trigger: Trigger) -> bool {
        self.0.iter().any(|(t, _)| *t == trigger)
    }

    /// Whether any bump-related triggers are bound.
    #[must_use]
    pub fn has_trigger_any_bump(&self) -> bool {
        self.0.iter().any(|(t, _)| {
            matches!(
                t,
                Trigger::PerfectBump | Trigger::EarlyBump | Trigger::LateBump | Trigger::BumpWhiff
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aegis_bindings() -> ActiveBehaviors {
        ActiveBehaviors(vec![
            (Trigger::BoltLost, Consequence::LoseLife),
            (Trigger::PerfectBump, Consequence::BoltSpeedBoost(1.5)),
            (Trigger::EarlyBump, Consequence::BoltSpeedBoost(0.8)),
            (Trigger::LateBump, Consequence::BoltSpeedBoost(0.8)),
        ])
    }

    #[test]
    fn consequences_for_bolt_lost() {
        let bindings = aegis_bindings();
        let results: Vec<_> = bindings.consequences_for(Trigger::BoltLost).collect();
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], Consequence::LoseLife));
    }

    #[test]
    fn consequences_for_perfect_bump() {
        let bindings = aegis_bindings();
        let results: Vec<_> = bindings.consequences_for(Trigger::PerfectBump).collect();
        assert_eq!(results.len(), 1);
        assert!(
            matches!(results[0], Consequence::BoltSpeedBoost(m) if (*m - 1.5).abs() < f32::EPSILON)
        );
    }

    #[test]
    fn consequences_for_unbound_trigger() {
        let bindings = aegis_bindings();
        assert!(
            bindings
                .consequences_for(Trigger::BumpWhiff)
                .next()
                .is_none()
        );
    }

    #[test]
    fn has_trigger_true_for_bound() {
        let bindings = aegis_bindings();
        assert!(bindings.has_trigger(Trigger::BoltLost));
        assert!(bindings.has_trigger(Trigger::PerfectBump));
    }

    #[test]
    fn has_trigger_false_for_unbound() {
        let bindings = aegis_bindings();
        assert!(!bindings.has_trigger(Trigger::BumpWhiff));
    }

    #[test]
    fn has_trigger_any_bump_true() {
        let bindings = aegis_bindings();
        assert!(bindings.has_trigger_any_bump());
    }

    #[test]
    fn has_trigger_any_bump_false_when_no_bumps() {
        let bindings = ActiveBehaviors(vec![(Trigger::BoltLost, Consequence::LoseLife)]);
        assert!(!bindings.has_trigger_any_bump());
    }

    #[test]
    fn default_is_empty() {
        let bindings = ActiveBehaviors::default();
        assert!(bindings.0.is_empty());
        assert!(!bindings.has_trigger(Trigger::BoltLost));
        assert!(!bindings.has_trigger_any_bump());
    }
}
