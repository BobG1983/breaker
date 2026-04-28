//! `VulnerableStack` component — a `Vec`-backed collection of target-side
//! incoming-damage multipliers. Target-side companion of `DamageBoostStack`
//! (damage received rather than damage dealt). Append-semantic (no
//! de-duplication of duplicate sources); callers pair each persistent
//! multiplier with a `SourceId` so they can later retract only their own
//! contribution, and one-shot multipliers are drained on the next aggregate
//! call.

use bevy::prelude::*;

use crate::{SourceId, source_id::entry_applies};

// Behavior 79a: negative compile-time contract — `VulnerableStack` does NOT
// derive `Clone`. The following single line is a documented negative contract.
// Leave it commented. Uncommenting must cause the file to fail to compile.
// If it ever compiles, the absent-Clone contract has been broken.
//
// let _ = VulnerableStack::default().clone(); // must fail to compile — VulnerableStack does not derive Clone

/// Persistent-lane entry: source for retraction, multiplier, and optional
/// emission-source filter (per `entry_applies`). Crate-private — tests must
/// observe behavior via aggregate methods, not by inspecting fields.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PersistentEntry {
    pub(crate) source:     SourceId,
    pub(crate) multiplier: f32,
    pub(crate) filter:     Option<SourceId>,
}

/// One-shot-lane entry: untagged multiplier and optional emission-source
/// filter (per `entry_applies`). Crate-private — tests must observe behavior
/// via aggregate methods, not by inspecting fields.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OneShotEntry {
    pub(crate) multiplier: f32,
    pub(crate) filter:     Option<SourceId>,
}

/// Target-side stack of incoming-damage multipliers. Symmetric counterpart
/// of `DamageBoostStack` (which sits on dealers).
///
/// Two disjoint lanes:
/// - `persistent`: tagged entries with optional source-filter. Callers
///   retract themselves by `SourceId` (matching the `source` field, NOT the
///   `filter` field). Appended — duplicate sources are NOT collapsed, so
///   `N` calls to `add(source, m)` contribute `m^N`.
/// - `one_shots`: untagged-source multipliers with optional source-filter.
///   Drained (cleared) by `aggregate_and_consume_one_shots` ONLY for entries
///   whose filter applies to the emission source.
///
/// Both lanes initialise empty (`Vec::new()`) via `#[derive(Default)]`.
#[derive(Component, Debug, Default)]
pub struct VulnerableStack {
    persistent: Vec<PersistentEntry>,
    one_shots:  Vec<OneShotEntry>,
}

impl VulnerableStack {
    /// Append a persistent multiplier tagged with its source. Duplicate
    /// sources are NOT collapsed — each call adds an entry. The entry has
    /// no filter (`filter: None`) and applies to all emissions.
    pub fn add(&mut self, source: SourceId, multiplier: f32) {
        self.persistent.push(PersistentEntry {
            source,
            multiplier,
            filter: None,
        });
    }

    /// Append a persistent multiplier tagged with its source and scoped via
    /// `filter`. The entry contributes only when an emission's source matches
    /// the filter (per `entry_applies`); never when the emission has no source.
    pub fn add_filtered(&mut self, source: SourceId, multiplier: f32, filter: SourceId) {
        self.persistent.push(PersistentEntry {
            source,
            multiplier,
            filter: Some(filter),
        });
    }

    /// Retract every persistent entry whose source matches. Takes a
    /// reference to `SourceId` so callers keep ownership of the key.
    /// Matches by the `source` field only — does NOT inspect `filter`.
    pub fn remove_by_source(&mut self, source: &SourceId) {
        self.persistent.retain(|e| &e.source != source);
    }

    /// Append a one-shot multiplier. Consumed on the next call to
    /// `aggregate_and_consume_one_shots`. The entry has no filter
    /// (`filter: None`) and applies to all emissions.
    pub fn add_one_shot(&mut self, multiplier: f32) {
        self.one_shots.push(OneShotEntry {
            multiplier,
            filter: None,
        });
    }

    /// Append a one-shot multiplier with an emission-source filter. Consumed
    /// only when a matching emission is processed; non-matching emissions
    /// preserve the entry on the lane.
    pub fn add_one_shot_filtered(&mut self, multiplier: f32, filter: SourceId) {
        self.one_shots.push(OneShotEntry {
            multiplier,
            filter: Some(filter),
        });
    }

    /// Multiplicative aggregate of every persistent entry whose filter
    /// applies to `emission_source` (per `entry_applies`). Empty stack — or
    /// stack with no applicable entries — returns `1.0` (multiplicative
    /// identity).
    #[must_use]
    pub fn aggregate_persistent(&self, emission_source: Option<&SourceId>) -> f32 {
        self.persistent
            .iter()
            .filter(|entry| entry_applies(entry.filter.as_ref(), emission_source))
            .map(|entry| entry.multiplier)
            .product()
    }

    /// Multiplicative aggregate of every one-shot entry whose filter applies
    /// to `emission_source`, AND consumes (removes) those applicable entries
    /// in place. Non-applicable entries MUST be retained. Empty queue — or
    /// queue with no applicable entries — returns `1.0`.
    pub fn aggregate_and_consume_one_shots(&mut self, emission_source: Option<&SourceId>) -> f32 {
        let mut product: f32 = 1.0;
        self.one_shots.retain(|entry| {
            if entry_applies(entry.filter.as_ref(), emission_source) {
                product *= entry.multiplier;
                false // applicable → consume (drop from Vec)
            } else {
                true // not applicable → retain
            }
        });
        product
    }

    /// Multiplicative aggregate of every one-shot entry whose filter applies
    /// to `emission_source` WITHOUT consuming any. Peek-only sibling of
    /// `aggregate_and_consume_one_shots`. Empty queue — or queue with no
    /// applicable entries — returns `1.0`.
    #[must_use]
    pub fn aggregate_one_shots(&self, emission_source: Option<&SourceId>) -> f32 {
        self.one_shots
            .iter()
            .filter(|entry| entry_applies(entry.filter.as_ref(), emission_source))
            .map(|entry| entry.multiplier)
            .product()
    }

    /// True iff BOTH lanes are empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.persistent.is_empty() && self.one_shots.is_empty()
    }
}
