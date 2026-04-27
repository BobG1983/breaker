//! Protocol resources — registries, active sets, offers, and run-condition helpers.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use rantzsoft_defaults::prelude::SeedableRegistry;

use super::super::definition::{ProtocolDefinition, ProtocolKind};

// ── ProtocolRegistry ────────────────────────────────────────────────────

/// Registry of protocol definitions, keyed on [`ProtocolKind`].
#[derive(Resource, Debug, Default)]
pub struct ProtocolRegistry {
    protocols: HashMap<ProtocolKind, ProtocolDefinition>,
}

impl ProtocolRegistry {
    /// Insert or replace a definition keyed on its kind.
    ///
    /// Production seeding goes through [`SeedableRegistry::seed`]; this helper
    /// only exists so tests can hand-construct a registry.
    #[cfg(test)]
    pub(crate) fn insert(&mut self, def: ProtocolDefinition) {
        self.protocols.insert(def.kind(), def);
    }

    /// Returns `true` when a definition exists for `kind`. Used by the
    /// scenario runner to validate `InjectProtocol` calls without leaking
    /// the internal `ProtocolDefinition` type.
    #[must_use]
    pub fn contains(&self, kind: ProtocolKind) -> bool {
        self.protocols.contains_key(&kind)
    }

    /// Look up a definition by kind.
    #[must_use]
    pub(crate) fn get(&self, kind: ProtocolKind) -> Option<&ProtocolDefinition> {
        self.protocols.get(&kind)
    }

    /// Number of stored protocol definitions. Used by tests that assert on
    /// default/insert behaviour.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.protocols.len()
    }

    /// True when no definitions have been inserted. Used by tests that assert
    /// on default behaviour.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.protocols.is_empty()
    }
}

impl SeedableRegistry for ProtocolRegistry {
    type Asset = ProtocolDefinition;

    fn asset_dir() -> &'static str {
        "protocols"
    }

    fn extensions() -> &'static [&'static str] {
        &["protocol.ron"]
    }

    fn seed(&mut self, assets: &[(AssetId<Self::Asset>, Self::Asset)]) {
        self.protocols.clear();
        for (_id, def) in assets {
            self.protocols.insert(def.kind(), def.clone());
        }
    }

    fn update_single(&mut self, _id: AssetId<Self::Asset>, asset: &Self::Asset) {
        self.protocols.insert(asset.kind(), asset.clone());
    }
}

// ── ActiveProtocols ─────────────────────────────────────────────────────

/// Protocols active in the current run. Populated by the protocol-offering
/// dispatch system on selection; cleared by `reset_run_state` on each new run.
#[derive(Resource, Debug, Default)]
pub struct ActiveProtocols {
    protocols: HashMap<ProtocolKind, ProtocolDefinition>,
}

impl ActiveProtocols {
    /// Insert or replace an active protocol keyed on its kind.
    pub fn insert(&mut self, def: ProtocolDefinition) {
        self.protocols.insert(def.kind(), def);
    }

    /// True when `kind` is currently active.
    #[must_use]
    pub fn contains(&self, kind: ProtocolKind) -> bool {
        self.protocols.contains_key(&kind)
    }

    /// Look up the active definition for `kind`, if any. Used by tests that
    /// verify the definition stored for an active protocol.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn get(&self, kind: ProtocolKind) -> Option<&ProtocolDefinition> {
        self.protocols.get(&kind)
    }

    /// Iterate only the kinds of active protocols. Exposed for
    /// scenario-runner invariant checkers that do not need the full
    /// definition payload.
    pub fn iter_kinds(&self) -> impl Iterator<Item = ProtocolKind> + '_ {
        self.protocols.keys().copied()
    }

    /// Remove a single protocol from the active set by kind. No-op when the
    /// kind is not present. Exposed `pub` so the scenario runner's mutation
    /// system can drive the cleanup-on-removal contract without reaching
    /// into `breaker-game` internals.
    pub fn remove(&mut self, kind: ProtocolKind) {
        self.protocols.remove(&kind);
    }

    /// Remove every active protocol. Called by `reset_run_state` between runs.
    pub(crate) fn clear(&mut self) {
        self.protocols.clear();
    }

    /// Number of active protocols.
    #[must_use]
    pub fn len(&self) -> usize {
        self.protocols.len()
    }

    /// True when no protocols are active.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.protocols.is_empty()
    }
}

// ── UnlockedProtocols ───────────────────────────────────────────────────

/// Protocols the player has unlocked for offering. Default = all 15
/// (meta-progression filtering is future work).
#[derive(Resource, Debug, Clone)]
pub(crate) struct UnlockedProtocols {
    unlocked: HashSet<ProtocolKind>,
}

impl Default for UnlockedProtocols {
    fn default() -> Self {
        Self {
            unlocked: ProtocolKind::ALL.iter().copied().collect(),
        }
    }
}

impl UnlockedProtocols {
    /// Empty unlocked set — no protocols are unlocked. Used by tests that
    /// need to force `generate_protocol_offering` into the empty-pool path
    /// without touching the default list.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn empty() -> Self {
        Self {
            unlocked: HashSet::new(),
        }
    }

    /// True when `kind` is unlocked for offering.
    #[must_use]
    pub(crate) fn contains(&self, kind: ProtocolKind) -> bool {
        self.unlocked.contains(&kind)
    }

    /// Number of unlocked protocols. Used by tests that assert the default
    /// set contains every kind.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.unlocked.len()
    }
}

// ── ProtocolOffer ───────────────────────────────────────────────────────

/// The protocol offered on the chip-select screen this visit. `None` when no
/// offer is active. Populated by the protocol-offering system on entry to
/// chip-select; cleared on player accept or decline.
#[derive(Resource, Debug, Clone, Default)]
pub(crate) struct ProtocolOffer(
    /// The pending offer, if any.
    pub(crate) Option<ProtocolDefinition>,
);

// ── protocol_active() ───────────────────────────────────────────────────

/// Run condition closure: passes when `kind` is in [`ActiveProtocols`].
pub(crate) fn protocol_active(kind: ProtocolKind) -> impl Fn(Res<ActiveProtocols>) -> bool {
    move |active: Res<ActiveProtocols>| active.contains(kind)
}
