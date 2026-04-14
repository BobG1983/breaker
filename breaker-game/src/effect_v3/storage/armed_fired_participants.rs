//! `ArmedFiredParticipants` — owner-side tracking of entities on which
//! armed `On` entries have fired an effect.

use std::collections::HashMap;

use bevy::prelude::*;

/// Tracks entities on which armed `On` entries have fired an effect,
/// keyed by the armed source string (e.g. `"chip_redirect#armed[0]"`).
///
/// Used by the Shape D disarm path to reverse effects on the exact
/// participants they were fired on, not on the owner entity.
///
/// Each fire appends a participant to the vector; duplicates are
/// intentional because `commands.reverse_effect()` reverses a single
/// instance per call — N fires produce N reverses.
#[derive(Component, Default, Debug)]
pub struct ArmedFiredParticipants(pub HashMap<String, Vec<Entity>>);

impl ArmedFiredParticipants {
    /// Append a fired participant under the given armed source key.
    ///
    /// Creates the entry if it is absent. Does not deduplicate — the
    /// same participant may appear multiple times for multiple fires.
    pub fn track(&mut self, armed_source: String, participant: Entity) {
        self.0.entry(armed_source).or_default().push(participant);
    }

    /// Drain and return tracked participants for an armed source key,
    /// removing the entry from the map. Returns an empty `Vec` when the
    /// key is absent.
    pub fn drain(&mut self, armed_source: &str) -> Vec<Entity> {
        self.0.remove(armed_source).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----- Behavior 20: `track` appends to Vec under key -----

    #[test]
    fn track_appends_to_vec_under_key() {
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();

        let mut tracked = ArmedFiredParticipants::default();
        tracked.track("key".to_owned(), e1);
        tracked.track("key".to_owned(), e2);

        let vec = tracked
            .0
            .get("key")
            .expect("key should be present after tracking");
        assert_eq!(vec.len(), 2, "Vec should have 2 entries in insertion order");
        assert_eq!(vec[0], e1, "first entry should be e1");
        assert_eq!(vec[1], e2, "second entry should be e2");
    }

    // Edge case: track the same entity twice — Vec is [e1, e1] (no dedup)
    #[test]
    fn track_does_not_deduplicate_repeated_participant() {
        let mut world = World::new();
        let e1 = world.spawn_empty().id();

        let mut tracked = ArmedFiredParticipants::default();
        tracked.track("key".to_owned(), e1);
        tracked.track("key".to_owned(), e1);

        let vec = tracked.0.get("key").expect("key should be present");
        assert_eq!(vec.len(), 2, "Vec should have 2 entries (no dedup)");
        assert_eq!(vec[0], e1);
        assert_eq!(vec[1], e1);
    }

    // ----- Behavior 21: `drain` removes and returns entries for key -----

    #[test]
    fn drain_removes_and_returns_entries_for_key() {
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();

        let mut tracked = ArmedFiredParticipants::default();
        tracked.track("key".to_owned(), e1);
        tracked.track("key".to_owned(), e2);

        let drained = tracked.drain("key");
        assert_eq!(drained.len(), 2, "drain should return 2 entries");
        assert_eq!(drained[0], e1, "first drained entry should be e1");
        assert_eq!(drained[1], e2, "second drained entry should be e2");

        // Second drain returns empty
        let second = tracked.drain("key");
        assert!(
            second.is_empty(),
            "second drain of the same key should return an empty Vec"
        );
    }

    // ----- Behavior 22: `drain` on unknown key returns empty Vec (no panic) -----

    #[test]
    fn drain_on_unknown_key_returns_empty_vec() {
        let mut tracked = ArmedFiredParticipants::default();

        let drained = tracked.drain("absent_key");
        assert!(
            drained.is_empty(),
            "drain on unknown key should return empty Vec"
        );
    }
}
