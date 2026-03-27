//! Node subdomain resources — registry, active layout, timer, and completion tracking.

use std::collections::HashMap;

use bevy::prelude::*;
use rantzsoft_defaults::prelude::SeedableRegistry;

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
            entity_scale: 1.0,
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
            entity_scale: 1.0,
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

    // ── SeedableRegistry tests ──────────────────────────────────────

    /// Helper: creates an `App` with `AssetPlugin` and returns `AssetId`s for
    /// a list of `NodeLayout` values. This gives us real (non-default)
    /// `AssetId`s backed by the Bevy asset system.
    fn asset_ids_for(layouts: &[NodeLayout]) -> (App, Vec<(AssetId<NodeLayout>, NodeLayout)>) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<NodeLayout>();

        let pairs: Vec<_> = {
            let mut assets = app.world_mut().resource_mut::<Assets<NodeLayout>>();
            layouts
                .iter()
                .map(|l| {
                    let handle = assets.add(l.clone());
                    (handle.id(), l.clone())
                })
                .collect()
        };

        (app, pairs)
    }

    // ── Behavior 1: seed() populates from asset pairs ───────────────

    /// `seed()` populates the registry from 2 `NodeLayout` assets
    /// (`test_a` and `test_b`) — len 2, `get_by_index(0)` returns `test_a`.
    #[test]
    fn seed_populates_registry_from_node_layouts() {
        let test_a = make_layout("test_a");
        let test_b = make_layout("test_b");
        let (_app, pairs) = asset_ids_for(&[test_a, test_b]);

        let mut registry = NodeLayoutRegistry::default();
        registry.seed(&pairs);

        assert_eq!(registry.len(), 2, "registry should contain 2 layouts");
        assert!(
            registry.get_by_name("test_a").is_some(),
            "registry should contain test_a"
        );
        assert!(
            registry.get_by_name("test_b").is_some(),
            "registry should contain test_b"
        );
        assert_eq!(
            registry.get_by_index(0).unwrap().name,
            "test_a",
            "get_by_index(0) should return test_a"
        );
    }

    // ── Behavior 2: seed() clears existing entries ──────────────────

    /// `seed()` clears previously inserted entries before populating.
    #[test]
    fn seed_clears_existing_entries() {
        let old = make_layout("old");
        let test_a = make_layout("test_a");
        let (_app, pairs_old) = asset_ids_for(&[old]);
        let (_app2, pairs_new) = asset_ids_for(&[test_a]);

        let mut registry = NodeLayoutRegistry::default();
        registry.seed(&pairs_old);
        assert_eq!(registry.len(), 1);
        assert!(registry.get_by_name("old").is_some());

        // Seed again with only test_a
        registry.seed(&pairs_new);

        assert_eq!(
            registry.len(),
            1,
            "after re-seed, only test_a should remain"
        );
        assert!(
            registry.get_by_name("test_a").is_some(),
            "test_a should be present after re-seed"
        );
        assert!(
            registry.get_by_name("old").is_none(),
            "old should be gone after re-seed"
        );
    }

    // ── Behavior 3: seed() inserts all layouts without cross-registry validation ─

    /// `seed()` inserts layouts without requiring a `CellTypeRegistry`.
    /// No validation at seed time — just insert.
    #[test]
    fn seed_inserts_without_cross_registry_validation() {
        let layout_with_unknown_alias = NodeLayout {
            name: "unknown_chars".to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['Z', 'Q']],
            pool: NodePool::default(),
            entity_scale: 1.0,
        };
        let (_app, pairs) = asset_ids_for(&[layout_with_unknown_alias]);

        let mut registry = NodeLayoutRegistry::default();
        registry.seed(&pairs);

        assert_eq!(
            registry.len(),
            1,
            "seed should insert layout even with unknown cell aliases"
        );
        assert!(
            registry.get_by_name("unknown_chars").is_some(),
            "layout with unknown aliases should be in the registry"
        );
    }

    // ── Behavior 4: update_single() upserts by name ─────────────────

    /// `update_single()` updates an existing layout's fields by name.
    #[test]
    fn update_single_upserts_existing_layout_by_name() {
        let test_a = make_layout("test_a");
        let (_app, pairs) = asset_ids_for(&[test_a]);

        let mut registry = NodeLayoutRegistry::default();
        registry.seed(&pairs);
        assert!(
            (registry.get_by_name("test_a").unwrap().timer_secs - 60.0).abs() < f32::EPSILON,
            "test_a timer_secs should be 60.0 initially"
        );

        // Update with timer_secs = 120.0
        let mut updated = make_layout("test_a");
        updated.timer_secs = 120.0;
        let id = pairs[0].0;
        registry.update_single(id, &updated);

        assert!(
            (registry.get_by_name("test_a").unwrap().timer_secs - 120.0).abs() < f32::EPSILON,
            "test_a timer_secs should be 120.0 after update_single, got {}",
            registry.get_by_name("test_a").unwrap().timer_secs
        );
    }

    /// `update_single()` with a new name inserts it.
    #[test]
    fn update_single_inserts_new_layout() {
        let test_a = make_layout("test_a");
        let new_layout = make_layout("test_b");
        let (_app, pairs) = asset_ids_for(&[test_a, new_layout]);

        let mut registry = NodeLayoutRegistry::default();
        // Only seed test_a
        registry.seed(&pairs[..1]);
        assert_eq!(registry.len(), 1);

        // update_single with test_b (not previously in registry)
        registry.update_single(pairs[1].0, &pairs[1].1);

        assert_eq!(
            registry.len(),
            2,
            "registry should have test_a and test_b after upsert"
        );
        assert!(
            registry.get_by_name("test_b").is_some(),
            "test_b should be present after update_single insert"
        );
    }

    // ── Behavior 5: update_all() resets and re-seeds ────────────────

    /// `update_all()` resets to default then seeds, removing old entries.
    #[test]
    fn update_all_resets_and_reseeds() {
        let old = make_layout("old");
        let test_a = make_layout("test_a");
        let (_app, pairs_old) = asset_ids_for(&[old]);
        let (_app2, pairs_new) = asset_ids_for(&[test_a]);

        let mut registry = NodeLayoutRegistry::default();
        registry.seed(&pairs_old);
        assert!(registry.get_by_name("old").is_some());

        registry.update_all(&pairs_new);

        assert_eq!(
            registry.len(),
            1,
            "after update_all, only test_a should remain"
        );
        assert!(
            registry.get_by_name("test_a").is_some(),
            "test_a should be present after update_all"
        );
        assert!(
            registry.get_by_name("old").is_none(),
            "old should be gone after update_all"
        );
    }

    // ── Behavior 6: asset_dir() and extensions() ────────────────────

    /// `asset_dir()` returns "nodes".
    #[test]
    fn asset_dir_returns_nodes() {
        assert_eq!(
            NodeLayoutRegistry::asset_dir(),
            "nodes",
            "asset_dir() should return \"nodes\""
        );
    }

    /// `extensions()` returns `&["node.ron"]`.
    #[test]
    fn extensions_returns_node_ron() {
        assert_eq!(
            NodeLayoutRegistry::extensions(),
            &["node.ron"],
            "extensions() should return [\"node.ron\"]"
        );
    }
}
