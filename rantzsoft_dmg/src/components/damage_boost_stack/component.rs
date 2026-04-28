use bevy::prelude::*;

use crate::{SourceId, source_id::entry_applies};

// Behavior 67a: negative compile-time contract — `DamageBoostStack` does NOT
// derive `Clone`. The following single line is a documented negative contract.
// Leave it commented. Uncommenting must cause the file to fail to compile.
// If it ever compiles, the absent-Clone contract has been broken.
//
// let _ = DamageBoostStack::default().clone(); // must fail to compile — DamageBoostStack does not derive Clone

/// A persistent-lane entry: tagged with its source for retraction, carries a
/// multiplier, and an optional filter that scopes which `emission_source`
/// values the entry applies to.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PersistentEntry {
    pub(crate) source:     SourceId,
    pub(crate) multiplier: f32,
    pub(crate) filter:     Option<SourceId>,
}

/// A one-shot-lane entry: bare multiplier with an optional filter that scopes
/// which `emission_source` values the entry applies to.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OneShotEntry {
    pub(crate) multiplier: f32,
    pub(crate) filter:     Option<SourceId>,
}

/// Dealer-side stack of damage multipliers.
///
/// Two disjoint lanes:
/// - `persistent`: tagged `PersistentEntry` records. Callers retract
///   themselves by `SourceId`. Appended — duplicate sources are NOT
///   collapsed, so `N` calls to `add(source, m)` contribute `m^N`.
/// - `one_shots`: untagged `OneShotEntry` records. Drained (cleared) by
///   `aggregate_and_consume_one_shots`.
///
/// Both lanes initialise empty (`Vec::new()`) via `#[derive(Default)]`.
#[derive(Component, Debug, Default)]
pub struct DamageBoostStack {
    persistent: Vec<PersistentEntry>,
    one_shots:  Vec<OneShotEntry>,
}

impl DamageBoostStack {
    /// Append a persistent multiplier tagged with its source. Duplicate
    /// sources are NOT collapsed — each call adds an entry. The entry has
    /// no filter — it applies to every emission.
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
    /// Filter-blind — matches by `source` field only.
    pub fn remove_by_source(&mut self, source: &SourceId) {
        self.persistent.retain(|entry| &entry.source != source);
    }

    /// Append a one-shot multiplier. Consumed on the next call to
    /// `aggregate_and_consume_one_shots`. The entry has no filter — it
    /// applies to every emission.
    pub fn add_one_shot(&mut self, multiplier: f32) {
        self.one_shots.push(OneShotEntry {
            multiplier,
            filter: None,
        });
    }

    /// Append a one-shot multiplier scoped to emissions whose `source` field
    /// equals `filter`. Consumed only when a matching emission is processed;
    /// non-matching emissions preserve the entry on the lane.
    pub fn add_one_shot_filtered(&mut self, multiplier: f32, filter: SourceId) {
        self.one_shots.push(OneShotEntry {
            multiplier,
            filter: Some(filter),
        });
    }

    /// Multiplicative aggregate of every persistent entry whose filter
    /// applies to `emission_source` (filterless entries always apply;
    /// filtered entries apply only when their filter equals
    /// `emission_source`). Empty (or fully-filtered-out) returns `1.0`
    /// (multiplicative identity).
    #[must_use]
    pub fn aggregate_persistent(&self, emission_source: Option<&SourceId>) -> f32 {
        self.persistent
            .iter()
            .filter(|e| entry_applies(e.filter.as_ref(), emission_source))
            .map(|e| e.multiplier)
            .product()
    }

    /// Multiplicative aggregate of every one-shot entry whose filter
    /// applies to `emission_source`. Matching entries are CONSUMED;
    /// non-matching entries are PRESERVED for a future call. Empty (or
    /// fully-filtered-out) returns `1.0` (multiplicative identity).
    pub fn aggregate_and_consume_one_shots(&mut self, emission_source: Option<&SourceId>) -> f32 {
        let mut product = 1.0_f32;
        self.one_shots.retain(|entry| {
            if entry_applies(entry.filter.as_ref(), emission_source) {
                product *= entry.multiplier;
                false // consume — drop from the lane
            } else {
                true // preserve — wait for a matching emission
            }
        });
        product
    }

    /// Peek-only sibling of `aggregate_and_consume_one_shots`. Multiplicative
    /// aggregate of every one-shot entry whose filter applies to
    /// `emission_source` WITHOUT consuming any entry. Empty (or
    /// fully-filtered-out) returns `1.0` (multiplicative identity).
    #[must_use]
    pub fn aggregate_one_shots(&self, emission_source: Option<&SourceId>) -> f32 {
        self.one_shots
            .iter()
            .filter(|e| entry_applies(e.filter.as_ref(), emission_source))
            .map(|e| e.multiplier)
            .product()
    }

    /// True iff BOTH lanes are empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.persistent.is_empty() && self.one_shots.is_empty()
    }
}
