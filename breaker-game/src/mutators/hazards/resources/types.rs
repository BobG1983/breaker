//! Hazard resources — registries, active stacks, offers, and run-condition helpers.

use std::collections::HashMap;

use bevy::prelude::*;
use rantzsoft_defaults::prelude::SeedableRegistry;

use super::super::definition::{HazardDefinition, HazardKind};

// ── HazardRegistry ──────────────────────────────────────────────────────

/// Registry of hazard definitions, keyed on [`HazardKind`].
#[derive(Resource, Debug, Default)]
pub struct HazardRegistry {
    hazards: HashMap<HazardKind, HazardDefinition>,
}

impl HazardRegistry {
    /// Insert or replace a definition keyed on its kind.
    ///
    /// Production seeding goes through [`SeedableRegistry::seed`]; this helper
    /// only exists so tests can hand-construct a registry.
    #[cfg(test)]
    pub(crate) fn insert(&mut self, def: HazardDefinition) {
        self.hazards.insert(def.kind(), def);
    }

    /// Look up a definition by kind.
    #[must_use]
    pub fn get(&self, kind: HazardKind) -> Option<&HazardDefinition> {
        self.hazards.get(&kind)
    }

    /// Number of stored hazard definitions. Used by tests that assert on
    /// default/insert behaviour.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.hazards.len()
    }

    /// True when no definitions have been inserted. Used by tests that assert
    /// on default behaviour.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.hazards.is_empty()
    }
}

impl SeedableRegistry for HazardRegistry {
    type Asset = HazardDefinition;

    fn asset_dir() -> &'static str {
        "hazards"
    }

    fn extensions() -> &'static [&'static str] {
        &["hazard.ron"]
    }

    fn seed(&mut self, assets: &[(AssetId<Self::Asset>, Self::Asset)]) {
        self.hazards.clear();
        for (_id, def) in assets {
            self.hazards.insert(def.kind(), def.clone());
        }
    }

    fn update_single(&mut self, _id: AssetId<Self::Asset>, asset: &Self::Asset) {
        self.hazards.insert(asset.kind(), asset.clone());
    }
}

// ── ActiveHazards ───────────────────────────────────────────────────────

/// Hazards currently active in the run, with per-hazard stack counts.
///
/// Invariant: the map never contains a 0-value entry — `add_stack` is the
/// only insertion path and always increments to ≥1 before returning.
/// `stacks(absent) == 0`.
#[derive(Resource, Debug, Default)]
pub struct ActiveHazards {
    stacks: HashMap<HazardKind, u32>,
}

impl ActiveHazards {
    /// Increments the stack count for `kind`. Starts at 1 on first call.
    /// Returns the new stack count.
    pub fn add_stack(&mut self, kind: HazardKind) -> u32 {
        let entry = self.stacks.entry(kind).or_insert(0);
        *entry += 1;
        *entry
    }

    /// Returns the stack count, or 0 if the hazard is not active.
    #[must_use]
    pub fn stacks(&self, kind: HazardKind) -> u32 {
        self.stacks.get(&kind).copied().unwrap_or(0)
    }

    /// True when `kind` has ≥ 1 stack.
    #[must_use]
    pub(crate) fn is_active(&self, kind: HazardKind) -> bool {
        self.stacks(kind) > 0
    }

    /// Iterate `(kind, stack_count)` pairs for every active hazard.
    pub fn iter(&self) -> impl Iterator<Item = (HazardKind, u32)> + '_ {
        self.stacks.iter().map(|(&k, &v)| (k, v))
    }

    /// Remove every active hazard. Called by `reset_run_state` between runs.
    pub(crate) fn clear(&mut self) {
        self.stacks.clear();
    }

    /// Number of distinct hazards with ≥1 stack.
    #[must_use]
    pub fn len(&self) -> usize {
        self.stacks.len()
    }

    /// True when no hazards are active.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stacks.is_empty()
    }

    /// Test/scenario-runner backdoor: insert a raw (kind, stacks) entry
    /// bypassing `add_stack`. Used by invariant self-test scenarios to force
    /// the zero-stack violation that `add_stack` cannot produce.
    pub fn force_insert_entry(&mut self, kind: HazardKind, stacks: u32) {
        self.stacks.insert(kind, stacks);
    }
}

// ── HazardOffers ────────────────────────────────────────────────────────

/// Hazards offered on the `HazardSelect` screen, populated by the
/// hazard-offering system on entry and consumed by the selection UI.
#[derive(Resource, Debug, Clone, Default)]
pub(crate) struct HazardOffers(
    /// Offered hazard definitions in display order.
    pub(crate) Vec<HazardDefinition>,
);

// ── hazard_active() ─────────────────────────────────────────────────────

/// Run condition closure: passes when `kind` has ≥ 1 stack.
pub(crate) fn hazard_active(kind: HazardKind) -> impl Fn(Res<ActiveHazards>) -> bool {
    move |active: Res<ActiveHazards>| active.is_active(kind)
}
