//! Node subdomain resources — registry, active layout, timer, and completion tracking.

use std::collections::HashMap;

use bevy::prelude::*;
use rantzsoft_defaults::prelude::SeedableRegistry;

use super::super::definition::{NodeLayout, NodePool};

/// The active node layout for the current node.
#[derive(Resource, Debug, Clone)]
pub struct ActiveNodeLayout(pub NodeLayout);

/// Registry of all loaded node layouts.
///
/// Stores layouts in a `HashMap` keyed by name, with a separate `Vec` preserving
/// insertion order for index-based access (node progression).
#[derive(Resource, Debug, Default, Clone)]
pub struct NodeLayoutRegistry {
    layouts: HashMap<String, NodeLayout>,
    order: Vec<String>,
    pools: HashMap<NodePool, Vec<String>>,
}

impl NodeLayoutRegistry {
    /// Look up a layout by name.
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<&NodeLayout> {
        self.layouts.get(name)
    }

    /// Look up a layout by insertion-order index.
    #[must_use]
    pub fn get_by_index(&self, index: usize) -> Option<&NodeLayout> {
        self.order
            .get(index)
            .and_then(|name| self.layouts.get(name))
    }

    /// Insert a layout with its declared pool (appended to insertion order).
    pub fn insert(&mut self, layout: NodeLayout) {
        let name = layout.name.clone();
        let pool = layout.pool;
        self.pools.entry(pool).or_default().push(name.clone());
        self.layouts.insert(name.clone(), layout);
        self.order.push(name);
    }

    /// Get all layout names in a given pool.
    #[must_use]
    pub fn get_pool(&self, pool: NodePool) -> Vec<&NodeLayout> {
        self.pools.get(&pool).map_or_else(Vec::new, |names| {
            names
                .iter()
                .filter_map(|name| self.layouts.get(name))
                .collect()
        })
    }

    /// Number of registered layouts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.layouts.len()
    }

    /// Whether the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.layouts.is_empty()
    }

    /// Remove all layouts.
    pub fn clear(&mut self) {
        self.layouts.clear();
        self.order.clear();
        self.pools.clear();
    }
}

impl SeedableRegistry for NodeLayoutRegistry {
    type Asset = NodeLayout;

    fn asset_dir() -> &'static str {
        "nodes"
    }

    fn extensions() -> &'static [&'static str] {
        &["node.ron"]
    }

    fn seed(&mut self, assets: &[(AssetId<NodeLayout>, NodeLayout)]) {
        self.clear();
        for (_id, layout) in assets {
            self.insert(layout.clone());
        }
    }

    fn update_single(&mut self, _id: AssetId<NodeLayout>, asset: &NodeLayout) {
        if let Some(existing) = self.layouts.get_mut(&asset.name) {
            *existing = asset.clone();
        } else {
            self.insert(asset.clone());
        }
    }
}

/// Countdown timer for the current node.
#[derive(Resource, Debug, Clone, Default)]
pub struct NodeTimer {
    /// Seconds remaining.
    pub remaining: f32,
    /// Total seconds for this node (used for ratio calculations).
    pub total: f32,
}

/// When set, overrides normal index-based layout selection in `set_active_layout`.
///
/// Set `Some(name)` before entering `GameState::Playing` to force a specific
/// named layout. Used by the scenario runner to drive deterministic test runs.
/// `None` (the default) restores normal index-based selection.
#[derive(Resource, Debug, Default, Clone)]
pub struct ScenarioLayoutOverride(pub Option<String>);

/// Tracks remaining cells that must be cleared for node completion.
#[derive(Resource, Debug, Default)]
pub struct ClearRemainingCount {
    /// Number of `RequiredToClear` cells still alive.
    pub remaining: u32,
}
