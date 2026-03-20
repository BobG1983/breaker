//! Node subdomain resources — registry, active layout, timer, and completion tracking.

use std::collections::HashMap;

use bevy::prelude::*;

use super::definition::{NodeLayout, NodePool};

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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_layout(name: &str) -> NodeLayout {
        NodeLayout {
            name: name.to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['.', '.']],
            pool: NodePool::default(),
        }
    }

    fn make_node_registry(names: &[&str]) -> NodeLayoutRegistry {
        let mut registry = NodeLayoutRegistry::default();
        for name in names {
            registry.insert(make_layout(name));
        }
        registry
    }

    #[test]
    fn get_by_name_returns_layout_with_matching_name() {
        let registry = make_node_registry(&["corridor", "open"]);
        let result = registry.get_by_name("corridor");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "corridor");
    }

    #[test]
    fn get_by_name_returns_none_for_missing_name() {
        let registry = make_node_registry(&["corridor"]);
        assert!(registry.get_by_name("missing").is_none());
    }

    #[test]
    fn get_by_name_on_empty_registry_returns_none() {
        let registry = NodeLayoutRegistry::default();
        assert!(registry.get_by_name("anything").is_none());
    }

    #[test]
    fn get_by_index_returns_in_insertion_order() {
        let registry = make_node_registry(&["first", "second", "third"]);
        assert_eq!(registry.get_by_index(0).unwrap().name, "first");
        assert_eq!(registry.get_by_index(1).unwrap().name, "second");
        assert_eq!(registry.get_by_index(2).unwrap().name, "third");
        assert!(registry.get_by_index(3).is_none());
    }

    #[test]
    fn len_and_is_empty() {
        let mut registry = NodeLayoutRegistry::default();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        registry.insert(make_layout("test"));
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn clear_removes_all() {
        let mut registry = make_node_registry(&["a", "b"]);
        assert_eq!(registry.len(), 2);
        registry.clear();
        assert!(registry.is_empty());
        assert!(registry.get_by_name("a").is_none());
        assert!(registry.get_by_index(0).is_none());
    }

    // --- Pool support tests ---

    fn make_pool_layout(name: &str, pool: NodePool) -> NodeLayout {
        NodeLayout {
            name: name.to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['.', '.']],
            pool,
        }
    }

    #[test]
    fn insert_uses_layout_pool_field() {
        let mut registry = NodeLayoutRegistry::default();
        registry.insert(make_pool_layout("arena", NodePool::Active));

        let active_layouts = registry.get_pool(NodePool::Active);
        assert_eq!(active_layouts.len(), 1);
        assert_eq!(active_layouts[0].name, "arena");
    }

    #[test]
    fn get_pool_returns_only_layouts_matching_pool() {
        let mut registry = NodeLayoutRegistry::default();
        registry.insert(make_pool_layout("a", NodePool::Passive));
        registry.insert(make_pool_layout("b", NodePool::Active));
        registry.insert(make_pool_layout("c", NodePool::Passive));

        let passive = registry.get_pool(NodePool::Passive);
        assert_eq!(passive.len(), 2);

        let passive_names: Vec<&str> = passive.iter().map(|l| l.name.as_str()).collect();
        assert!(passive_names.contains(&"a"));
        assert!(passive_names.contains(&"c"));
        assert!(!passive_names.contains(&"b"));
    }

    #[test]
    fn get_pool_for_empty_pool_returns_empty() {
        let mut registry = NodeLayoutRegistry::default();
        registry.insert(make_pool_layout("quiet", NodePool::Passive));
        registry.insert(make_pool_layout("calm", NodePool::Passive));

        let boss = registry.get_pool(NodePool::Boss);
        assert!(boss.is_empty());
    }

    #[test]
    fn clear_removes_pool_tracking() {
        let mut registry = NodeLayoutRegistry::default();
        registry.insert(make_pool_layout("a", NodePool::Passive));
        registry.insert(make_pool_layout("b", NodePool::Active));
        registry.insert(make_pool_layout("c", NodePool::Boss));

        registry.clear();

        assert!(registry.get_pool(NodePool::Passive).is_empty());
        assert!(registry.get_pool(NodePool::Active).is_empty());
        assert!(registry.get_pool(NodePool::Boss).is_empty());
    }
}
