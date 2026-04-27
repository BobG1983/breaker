//! `VulnerableStack` component — a `Vec`-backed collection of target-side
//! incoming-damage multipliers. Target-side companion of `DamageBoostStack`
//! (damage received rather than damage dealt). Append-semantic (no
//! de-duplication of duplicate sources); callers pair each persistent
//! multiplier with a `SourceId` so they can later retract only their own
//! contribution, and one-shot multipliers are drained on the next aggregate
//! call.

use bevy::prelude::*;

use crate::SourceId;

// Behavior 79a: negative compile-time contract — `VulnerableStack` does NOT
// derive `Clone`. The following single line is a documented negative contract.
// Leave it commented. Uncommenting must cause the file to fail to compile.
// If it ever compiles, the absent-Clone contract has been broken.
//
// let _ = VulnerableStack::default().clone(); // must fail to compile — VulnerableStack does not derive Clone

/// Target-side stack of incoming-damage multipliers.
///
/// Two disjoint lanes:
/// - `persistent`: tagged `(SourceId, f32)` entries. Callers retract
///   themselves by `SourceId`. Appended — duplicate sources are NOT
///   collapsed, so `N` calls to `add(source, m)` contribute `m^N`.
/// - `one_shots`: untagged multipliers. Drained (cleared) by
///   `aggregate_and_consume_one_shots`.
///
/// Both lanes initialise empty (`Vec::new()`) via `#[derive(Default)]`.
#[derive(Component, Debug, Default)]
pub struct VulnerableStack {
    persistent: Vec<(SourceId, f32)>,
    one_shots:  Vec<f32>,
}

impl VulnerableStack {
    /// Append a persistent multiplier tagged with its source. Duplicate
    /// sources are NOT collapsed — each call adds an entry.
    pub fn add(&mut self, source: SourceId, multiplier: f32) {
        self.persistent.push((source, multiplier));
    }

    /// Retract every persistent entry whose source matches. Takes a
    /// reference to `SourceId` so callers keep ownership of the key.
    pub fn remove_by_source(&mut self, source: &SourceId) {
        self.persistent.retain(|(s, _)| s != source);
    }

    /// Append a one-shot multiplier. Consumed on the next call to
    /// `aggregate_and_consume_one_shots`.
    pub fn add_one_shot(&mut self, multiplier: f32) {
        self.one_shots.push(multiplier);
    }

    /// Multiplicative aggregate of every persistent entry.
    /// Empty stack returns `1.0` (multiplicative identity).
    #[must_use]
    pub fn aggregate_persistent(&self) -> f32 {
        self.persistent.iter().map(|(_, m)| *m).product()
    }

    /// Multiplicative aggregate of every one-shot entry, then clears them.
    /// Empty queue returns `1.0` (multiplicative identity).
    pub fn aggregate_and_consume_one_shots(&mut self) -> f32 {
        let product: f32 = self.one_shots.iter().copied().product();
        self.one_shots.clear();
        product
    }

    /// Multiplicative aggregate of every one-shot entry WITHOUT consuming
    /// them. Peek-only sibling of `aggregate_and_consume_one_shots`.
    /// Empty queue returns `1.0` (multiplicative identity).
    #[must_use]
    pub fn aggregate_one_shots(&self) -> f32 {
        self.one_shots.iter().copied().product()
    }

    /// True iff BOTH lanes are empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.persistent.is_empty() && self.one_shots.is_empty()
    }
}
