//! Incremental quadtree spatial index for broad-phase collision queries.

use bevy::{platform::collections::HashSet, prelude::*};

use crate::{
    aabb::Aabb2D,
    ccd::{CCD_EPSILON, SweepHit},
    collision_layers::CollisionLayers,
};

/// Internal node representation for the quadtree.
enum QuadNode {
    Leaf {
        items: Vec<(Entity, Aabb2D, CollisionLayers)>,
    },
    Branch {
        children: Box<[QuadNode; 4]>,
        /// Items stored at this branch level because they span multiple child
        /// quadrants and cannot be pushed down.
        items: Vec<(Entity, Aabb2D, CollisionLayers)>,
    },
}

/// Spatial index mapping entities to `Aabb2D` bounds, supporting incremental
/// updates and spatial queries.
///
/// Items are stored in leaf nodes and split when a leaf exceeds
/// `max_items_per_leaf`, up to `max_depth` levels.
pub struct Quadtree {
    root: QuadNode,
    bounds: Aabb2D,
    max_items_per_leaf: usize,
    max_depth: usize,
    len: usize,
}

/// Returns the `Aabb2D` for child quadrant `index` (0..4) within `parent_bounds`.
///
/// Quadrant mapping:
/// - 0: bottom-left
/// - 1: bottom-right
/// - 2: top-left
/// - 3: top-right
fn child_bounds(parent_bounds: &Aabb2D, index: usize) -> Aabb2D {
    let min = parent_bounds.min();
    let max = parent_bounds.max();
    let mid = parent_bounds.center;
    let child_half = parent_bounds.half_extents / 2.0;

    let child_center = match index {
        0 => Vec2::new(min.x.midpoint(mid.x), min.y.midpoint(mid.y)),
        1 => Vec2::new(mid.x.midpoint(max.x), min.y.midpoint(mid.y)),
        2 => Vec2::new(min.x.midpoint(mid.x), mid.y.midpoint(max.y)),
        3 => Vec2::new(mid.x.midpoint(max.x), mid.y.midpoint(max.y)),
        _ => unreachable!(),
    };

    Aabb2D::new(child_center, child_half)
}

/// Returns the unique child quadrant index (0..4) that fully contains
/// `item_bounds`, or `None` if it spans multiple quadrants.
fn fitting_quadrant(parent_bounds: &Aabb2D, item_bounds: &Aabb2D) -> Option<usize> {
    let mid = parent_bounds.center;

    let item_min = item_bounds.min();
    let item_max = item_bounds.max();

    // Determine which side of the midpoint each edge falls on
    let left = item_max.x <= mid.x;
    let right = item_min.x >= mid.x;
    let bottom = item_max.y <= mid.y;
    let top = item_min.y >= mid.y;

    match (left, right, bottom, top) {
        (true, false, true, false) => Some(0), // bottom-left
        (false, true, true, false) => Some(1), // bottom-right
        (true, false, false, true) => Some(2), // top-left
        (false, true, false, true) => Some(3), // top-right
        _ => None,                             // spans multiple quadrants
    }
}

struct TreeConfig {
    max_items_per_leaf: usize,
    max_depth: usize,
    depth: usize,
}

fn insert_into_node(
    node: &mut QuadNode,
    node_bounds: &Aabb2D,
    entity: Entity,
    bounds: Aabb2D,
    layers: CollisionLayers,
    cfg: TreeConfig,
) {
    match node {
        QuadNode::Leaf { items } => {
            items.push((entity, bounds, layers));
            // Split if over capacity and we haven't reached max depth
            if items.len() > cfg.max_items_per_leaf && cfg.depth < cfg.max_depth {
                let old_items: Vec<(Entity, Aabb2D, CollisionLayers)> = std::mem::take(items);
                let children = Box::new([
                    QuadNode::Leaf { items: Vec::new() },
                    QuadNode::Leaf { items: Vec::new() },
                    QuadNode::Leaf { items: Vec::new() },
                    QuadNode::Leaf { items: Vec::new() },
                ]);
                *node = QuadNode::Branch {
                    children,
                    items: Vec::new(),
                };
                // Re-insert all items
                for (e, b, l) in old_items {
                    insert_into_node(
                        node,
                        node_bounds,
                        e,
                        b,
                        l,
                        TreeConfig {
                            max_items_per_leaf: cfg.max_items_per_leaf,
                            max_depth: cfg.max_depth,
                            depth: cfg.depth,
                        },
                    );
                }
            }
        }
        QuadNode::Branch { children, items } => {
            if let Some(qi) = fitting_quadrant(node_bounds, &bounds) {
                let cb = child_bounds(node_bounds, qi);
                insert_into_node(
                    &mut children[qi],
                    &cb,
                    entity,
                    bounds,
                    layers,
                    TreeConfig {
                        max_items_per_leaf: cfg.max_items_per_leaf,
                        max_depth: cfg.max_depth,
                        depth: cfg.depth + 1,
                    },
                );
            } else {
                // Spans multiple quadrants — store at this branch level
                items.push((entity, bounds, layers));
            }
        }
    }
}

fn remove_from_node(node: &mut QuadNode, entity: Entity) -> bool {
    match node {
        QuadNode::Leaf { items } => {
            if let Some(pos) = items.iter().position(|(e, ..)| *e == entity) {
                items.swap_remove(pos);
                return true;
            }
            false
        }
        QuadNode::Branch { children, items } => {
            // Check branch-level items first
            if let Some(pos) = items.iter().position(|(e, ..)| *e == entity) {
                items.swap_remove(pos);
                return true;
            }
            // Recurse into children
            for child in children.iter_mut() {
                if remove_from_node(child, entity) {
                    return true;
                }
            }
            false
        }
    }
}

fn query_aabb_node(
    node: &QuadNode,
    node_bounds: &Aabb2D,
    region: &Aabb2D,
    results: &mut Vec<Entity>,
    seen: &mut HashSet<Entity>,
) {
    if !node_bounds.overlaps(region) {
        return;
    }

    match node {
        QuadNode::Leaf { items } => {
            for &(entity, ref item_bounds, _) in items {
                if item_bounds.overlaps(region) && seen.insert(entity) {
                    results.push(entity);
                }
            }
        }
        QuadNode::Branch { children, items } => {
            // Check branch-level items
            for &(entity, ref item_bounds, _) in items {
                if item_bounds.overlaps(region) && seen.insert(entity) {
                    results.push(entity);
                }
            }
            // Recurse into children
            for (i, child) in children.iter().enumerate() {
                let cb = child_bounds(node_bounds, i);
                query_aabb_node(child, &cb, region, results, seen);
            }
        }
    }
}

fn query_aabb_filtered_node(
    node: &QuadNode,
    node_bounds: &Aabb2D,
    region: &Aabb2D,
    query_layers: CollisionLayers,
    results: &mut Vec<Entity>,
    seen: &mut HashSet<Entity>,
) {
    if !node_bounds.overlaps(region) {
        return;
    }

    match node {
        QuadNode::Leaf { items } => {
            for &(entity, ref item_bounds, ref item_layers) in items {
                if item_bounds.overlaps(region)
                    && query_layers.interacts_with(item_layers)
                    && seen.insert(entity)
                {
                    results.push(entity);
                }
            }
        }
        QuadNode::Branch { children, items } => {
            // Check branch-level items
            for &(entity, ref item_bounds, ref item_layers) in items {
                if item_bounds.overlaps(region)
                    && query_layers.interacts_with(item_layers)
                    && seen.insert(entity)
                {
                    results.push(entity);
                }
            }
            // Recurse into children
            for (i, child) in children.iter().enumerate() {
                let cb = child_bounds(node_bounds, i);
                query_aabb_filtered_node(child, &cb, region, query_layers, results, seen);
            }
        }
    }
}

/// Checks whether a circle (center, radius) overlaps an AABB.
fn circle_overlaps_aabb(center: Vec2, radius: f32, aabb: &Aabb2D) -> bool {
    let aabb_min = aabb.min();
    let aabb_max = aabb.max();
    // Find the closest point on the AABB to the circle center
    let closest = Vec2::new(
        center.x.clamp(aabb_min.x, aabb_max.x),
        center.y.clamp(aabb_min.y, aabb_max.y),
    );
    let dist_sq = center.distance_squared(closest);
    dist_sq <= radius * radius
}

impl Quadtree {
    /// Creates a new empty `Quadtree` covering the given bounds.
    #[must_use]
    pub const fn new(bounds: Aabb2D, max_items_per_leaf: usize, max_depth: usize) -> Self {
        Self {
            root: QuadNode::Leaf { items: Vec::new() },
            bounds,
            max_items_per_leaf,
            max_depth,
            len: 0,
        }
    }

    /// Inserts an entity with the given bounds and collision layers into the tree.
    pub fn insert(&mut self, entity: Entity, bounds: Aabb2D, layers: CollisionLayers) {
        insert_into_node(
            &mut self.root,
            &self.bounds,
            entity,
            bounds,
            layers,
            TreeConfig {
                max_items_per_leaf: self.max_items_per_leaf,
                max_depth: self.max_depth,
                depth: 0,
            },
        );
        self.len += 1;
    }

    /// Removes an entity from the tree. Returns `true` if found and removed.
    pub fn remove(&mut self, entity: Entity) -> bool {
        if remove_from_node(&mut self.root, entity) {
            self.len -= 1;
            true
        } else {
            false
        }
    }

    /// Returns all entities whose bounds overlap the given region.
    ///
    /// This is the unfiltered query — it ignores `CollisionLayers` entirely
    /// and returns every spatially overlapping entity.
    #[must_use]
    pub fn query_aabb(&self, region: &Aabb2D) -> Vec<Entity> {
        let mut results = Vec::new();
        let mut seen = HashSet::new();
        query_aabb_node(&self.root, &self.bounds, region, &mut results, &mut seen);
        results
    }

    /// Returns entities whose bounds overlap the region AND whose
    /// `CollisionLayers` interact with the given query layers.
    ///
    /// Filtering rule: an entity is returned when
    /// `query_layers.interacts_with(&entity_layers)` is `true`, i.e.
    /// `query_layers.mask & entity.membership != 0`.
    #[must_use]
    pub fn query_aabb_filtered(&self, region: &Aabb2D, layers: CollisionLayers) -> Vec<Entity> {
        let mut results = Vec::new();
        let mut seen = HashSet::new();
        query_aabb_filtered_node(
            &self.root,
            &self.bounds,
            region,
            layers,
            &mut results,
            &mut seen,
        );
        results
    }

    /// Returns all entities whose bounds overlap a circle defined by center
    /// and radius.
    #[must_use]
    pub fn query_circle(&self, center: Vec2, radius: f32) -> Vec<Entity> {
        // Use AABB query as broad phase, then refine with circle test
        let broad_region = Aabb2D::new(center, Vec2::splat(radius));
        let candidates = self.query_aabb(&broad_region);
        let mut results = Vec::new();
        let mut seen = HashSet::new();
        // We need the item bounds to do the circle test. Walk the tree to
        // collect (entity, bounds) pairs for the candidates.
        let candidate_set: HashSet<Entity> = candidates.into_iter().collect();
        collect_matching_items(&self.root, &candidate_set, &mut |entity, item_bounds, _| {
            if circle_overlaps_aabb(center, radius, item_bounds) && seen.insert(entity) {
                results.push(entity);
            }
        });
        results
    }

    /// Returns entities whose bounds overlap a circle AND whose
    /// `CollisionLayers` interact with the given query layers.
    #[must_use]
    pub fn query_circle_filtered(
        &self,
        center: Vec2,
        radius: f32,
        layers: CollisionLayers,
    ) -> Vec<Entity> {
        // Use AABB broad phase with layer filtering, then refine with circle test
        let broad_region = Aabb2D::new(center, Vec2::splat(radius));
        let candidates = self.query_aabb_filtered(&broad_region, layers);
        let mut results = Vec::new();
        let mut seen = HashSet::new();
        let candidate_set: HashSet<Entity> = candidates.into_iter().collect();
        collect_matching_items(&self.root, &candidate_set, &mut |entity, item_bounds, _| {
            if circle_overlaps_aabb(center, radius, item_bounds) && seen.insert(entity) {
                results.push(entity);
            }
        });
        results
    }

    /// Returns the number of items in the tree.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the tree contains no items.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Removes all items from the tree.
    pub fn clear(&mut self) {
        self.root = QuadNode::Leaf { items: Vec::new() };
        self.len = 0;
    }

    /// Casts a circle along a ray and returns all hits sorted by distance.
    ///
    /// The circle is swept from `origin` in `direction` up to `max_dist`.
    /// Internally performs Minkowski expansion (expands each stored AABB by
    /// `radius`) and casts a point ray against the expanded AABBs.
    ///
    /// Only entities whose `CollisionLayers` interact with `layers` are tested.
    /// Results are sorted nearest-first by hit distance.
    #[must_use]
    pub fn cast_circle(
        &self,
        origin: Vec2,
        direction: Vec2,
        max_dist: f32,
        radius: f32,
        layers: CollisionLayers,
    ) -> Vec<SweepHit> {
        // Compute swept AABB for broad-phase
        let end = origin + direction * max_dist;
        let swept_aabb = Aabb2D::from_min_max(
            origin.min(end) - Vec2::splat(radius),
            origin.max(end) + Vec2::splat(radius),
        );

        // Broad-phase: find candidate entities via layer-filtered AABB query
        let candidates = self.query_aabb_filtered(&swept_aabb, layers);
        let candidate_set: HashSet<Entity> = candidates.into_iter().collect();

        // Narrow-phase: ray-cast against Minkowski-expanded AABBs
        let mut raw_hits: Vec<(f32, SweepHit)> = Vec::new();
        collect_matching_items(&self.root, &candidate_set, &mut |entity, stored_aabb, _| {
            let expanded = stored_aabb.expand_by(radius);
            if let Some(ray_hit) = expanded.ray_intersect(origin, direction, max_dist) {
                let safe_dist = (ray_hit.distance - CCD_EPSILON).max(0.0);
                let position = origin + direction * safe_dist;
                let remaining = (max_dist - ray_hit.distance).max(0.0);
                raw_hits.push((
                    ray_hit.distance,
                    SweepHit {
                        entity,
                        position,
                        normal: ray_hit.normal,
                        remaining,
                    },
                ));
            }
        });

        // Sort by raw hit distance (nearest first)
        raw_hits.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        raw_hits.into_iter().map(|(_, hit)| hit).collect()
    }

    /// Casts a zero-radius ray and returns all hits sorted by distance.
    ///
    /// Equivalent to `cast_circle` with `radius = 0.0`.
    #[must_use]
    pub fn cast_ray(
        &self,
        origin: Vec2,
        direction: Vec2,
        max_dist: f32,
        layers: CollisionLayers,
    ) -> Vec<SweepHit> {
        self.cast_circle(origin, direction, max_dist, 0.0, layers)
    }
}

fn collect_matching_items(
    node: &QuadNode,
    candidates: &HashSet<Entity>,
    callback: &mut impl FnMut(Entity, &Aabb2D, &CollisionLayers),
) {
    match node {
        QuadNode::Leaf { items } => {
            for &(entity, ref bounds, ref layers) in items {
                if candidates.contains(&entity) {
                    callback(entity, bounds, layers);
                }
            }
        }
        QuadNode::Branch { children, items } => {
            for &(entity, ref bounds, ref layers) in items {
                if candidates.contains(&entity) {
                    callback(entity, bounds, layers);
                }
            }
            for child in children.iter() {
                collect_matching_items(child, candidates, callback);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::world::World;

    use super::*;

    /// Helper: creates a quadtree covering [-500, 500] on both axes with default
    /// leaf capacity 8 and max depth 8.
    fn test_tree() -> Quadtree {
        Quadtree::new(Aabb2D::new(Vec2::ZERO, Vec2::new(500.0, 500.0)), 8, 8)
    }

    /// Helper: creates a small `Aabb2D` centered at the given position.
    fn small_aabb(x: f32, y: f32) -> Aabb2D {
        Aabb2D::new(Vec2::new(x, y), Vec2::new(1.0, 1.0))
    }

    /// Helper: spawns N distinct entities from a `World` for use as quadtree keys.
    fn spawn_entities(count: usize) -> Vec<Entity> {
        let mut world = World::new();
        (0..count).map(|_| world.spawn_empty().id()).collect()
    }

    // ════════════════════════════════════════════════════════════════
    // Existing tests — updated to 3-arg insert with CollisionLayers::default()
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn new_creates_empty_tree() {
        let tree = test_tree();
        assert_eq!(tree.len(), 0);
        assert!(tree.is_empty());
    }

    #[test]
    fn insert_increases_len() {
        let mut tree = test_tree();
        let entities = spawn_entities(1);
        tree.insert(
            entities[0],
            small_aabb(10.0, 10.0),
            CollisionLayers::default(),
        );
        assert_eq!(tree.len(), 1);
        assert!(!tree.is_empty());
    }

    #[test]
    #[expect(
        clippy::cast_precision_loss,
        reason = "test loop index is always small"
    )]
    fn insert_multiple_entities() {
        let mut tree = test_tree();
        let entities = spawn_entities(5);
        for (i, &e) in entities.iter().enumerate() {
            tree.insert(
                e,
                small_aabb(i as f32 * 10.0, 0.0),
                CollisionLayers::default(),
            );
        }
        assert_eq!(tree.len(), 5);
    }

    #[test]
    fn remove_decreases_len() {
        let mut tree = test_tree();
        let entities = spawn_entities(3);
        tree.insert(
            entities[0],
            small_aabb(0.0, 0.0),
            CollisionLayers::default(),
        );
        tree.insert(
            entities[1],
            small_aabb(10.0, 0.0),
            CollisionLayers::default(),
        );
        tree.insert(
            entities[2],
            small_aabb(20.0, 0.0),
            CollisionLayers::default(),
        );

        let removed = tree.remove(entities[1]);
        assert!(removed);
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn remove_returns_false_for_missing_entity() {
        let mut tree = test_tree();
        let entities = spawn_entities(2);
        tree.insert(
            entities[0],
            small_aabb(0.0, 0.0),
            CollisionLayers::default(),
        );
        // entities[1] was never inserted
        let removed = tree.remove(entities[1]);
        assert!(!removed);
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn query_aabb_finds_overlapping_entity() {
        let mut tree = test_tree();
        let entities = spawn_entities(1);
        tree.insert(
            entities[0],
            small_aabb(10.0, 10.0),
            CollisionLayers::default(),
        );

        let query_region = Aabb2D::new(Vec2::new(10.0, 10.0), Vec2::new(5.0, 5.0));
        let results = tree.query_aabb(&query_region);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], entities[0]);
    }

    #[test]
    fn query_aabb_misses_distant_entity() {
        let mut tree = test_tree();
        let entities = spawn_entities(1);
        tree.insert(
            entities[0],
            small_aabb(10.0, 10.0),
            CollisionLayers::default(),
        );

        let query_region = Aabb2D::new(Vec2::new(100.0, 100.0), Vec2::new(5.0, 5.0));
        let results = tree.query_aabb(&query_region);
        assert!(results.is_empty());
    }

    #[test]
    fn query_aabb_finds_multiple_overlapping_entities() {
        let mut tree = test_tree();
        let entities = spawn_entities(3);
        tree.insert(
            entities[0],
            small_aabb(0.0, 0.0),
            CollisionLayers::default(),
        );
        tree.insert(
            entities[1],
            small_aabb(2.0, 0.0),
            CollisionLayers::default(),
        );
        tree.insert(
            entities[2],
            small_aabb(0.0, 2.0),
            CollisionLayers::default(),
        );

        // Query region covering all three
        let query_region = Aabb2D::new(Vec2::new(1.0, 1.0), Vec2::new(5.0, 5.0));
        let mut results = tree.query_aabb(&query_region);
        results.sort();
        let mut expected = vec![entities[0], entities[1], entities[2]];
        expected.sort();
        assert_eq!(results.len(), 3);
        assert_eq!(results, expected);
    }

    #[test]
    fn query_circle_finds_entity_in_range() {
        let mut tree = test_tree();
        let entities = spawn_entities(1);
        tree.insert(
            entities[0],
            small_aabb(5.0, 0.0),
            CollisionLayers::default(),
        );

        let results = tree.query_circle(Vec2::ZERO, 10.0);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], entities[0]);
    }

    #[test]
    fn query_circle_misses_entity_out_of_range() {
        let mut tree = test_tree();
        let entities = spawn_entities(1);
        tree.insert(
            entities[0],
            small_aabb(100.0, 0.0),
            CollisionLayers::default(),
        );

        let results = tree.query_circle(Vec2::ZERO, 10.0);
        assert!(results.is_empty());
    }

    #[test]
    #[expect(
        clippy::cast_precision_loss,
        reason = "test loop index is always small"
    )]
    fn clear_empties_the_tree() {
        let mut tree = test_tree();
        let entities = spawn_entities(5);
        for (i, &e) in entities.iter().enumerate() {
            tree.insert(
                e,
                small_aabb(i as f32 * 10.0, 0.0),
                CollisionLayers::default(),
            );
        }
        assert_eq!(tree.len(), 5);

        tree.clear();
        assert_eq!(tree.len(), 0);
        assert!(tree.is_empty());
    }

    #[test]
    fn insert_then_remove_then_query_finds_nothing() {
        let mut tree = test_tree();
        let entities = spawn_entities(1);
        tree.insert(
            entities[0],
            small_aabb(10.0, 10.0),
            CollisionLayers::default(),
        );
        tree.remove(entities[0]);

        let query_region = Aabb2D::new(Vec2::new(10.0, 10.0), Vec2::new(5.0, 5.0));
        let results = tree.query_aabb(&query_region);
        assert!(
            results.is_empty(),
            "removed entity should not appear in query"
        );
    }

    #[test]
    #[expect(
        clippy::cast_precision_loss,
        reason = "test loop index is always small"
    )]
    fn handles_splitting_when_exceeding_leaf_capacity() {
        // Use a tree with max 4 items per leaf to force splits
        let mut tree = Quadtree::new(Aabb2D::new(Vec2::ZERO, Vec2::new(500.0, 500.0)), 4, 8);

        let entities = spawn_entities(10);
        // Insert 10 entities in the same area to trigger splitting
        for (i, &e) in entities.iter().enumerate() {
            tree.insert(
                e,
                small_aabb(i as f32 * 0.5, i as f32 * 0.5),
                CollisionLayers::default(),
            );
        }
        assert_eq!(tree.len(), 10);

        // All should still be queryable
        let query_region = Aabb2D::new(Vec2::ZERO, Vec2::new(20.0, 20.0));
        let results = tree.query_aabb(&query_region);
        assert_eq!(
            results.len(),
            10,
            "all entities should be found after splits"
        );
    }

    #[test]
    #[expect(
        clippy::cast_precision_loss,
        reason = "test loop index is always small"
    )]
    fn query_returns_no_duplicates_for_entity_spanning_quadrants() {
        let mut tree = Quadtree::new(Aabb2D::new(Vec2::ZERO, Vec2::new(500.0, 500.0)), 4, 8);

        let entities = spawn_entities(6);

        // Insert a large entity centered at the origin that spans all four quadrants
        let big_aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
        tree.insert(entities[0], big_aabb, CollisionLayers::default());

        // Also insert some small entities to potentially trigger splits
        for (i, &e) in entities[1..6].iter().enumerate() {
            tree.insert(
                e,
                small_aabb(200.0 + i as f32, 200.0),
                CollisionLayers::default(),
            );
        }

        // Query the area containing the large entity
        let query_region = Aabb2D::new(Vec2::ZERO, Vec2::new(50.0, 50.0));
        let results = tree.query_aabb(&query_region);

        let count = results.iter().filter(|&&e| e == entities[0]).count();
        assert_eq!(
            count, 1,
            "entity spanning multiple quadrants should appear exactly once"
        );
    }

    // ════════════════════════════════════════════════════════════════
    // New tests — behaviors 7-11 (insert with layers, filtered queries)
    // ════════════════════════════════════════════════════════════════

    // ── Behavior 7: insert takes (Entity, Aabb2D, CollisionLayers) ──

    #[test]
    fn insert_with_layers_increases_len() {
        let mut tree = test_tree();
        let entities = spawn_entities(1);
        let aabb = Aabb2D::new(Vec2::new(10.0, 10.0), Vec2::new(1.0, 1.0));
        let layers = CollisionLayers::new(0x01, 0x02);
        tree.insert(entities[0], aabb, layers);
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn insert_with_default_layers_stores_but_invisible_to_filtered_queries() {
        let mut tree = test_tree();
        let entities = spawn_entities(1);
        let aabb = small_aabb(10.0, 10.0);
        // Insert with default layers (membership=0, mask=0)
        tree.insert(entities[0], aabb, CollisionLayers::default());
        assert_eq!(tree.len(), 1);

        // Filtered query with any mask should not return it
        let region = Aabb2D::new(Vec2::new(10.0, 10.0), Vec2::new(5.0, 5.0));
        let results = tree.query_aabb_filtered(&region, CollisionLayers::new(0xFF, 0xFF));
        assert!(
            results.is_empty(),
            "entity with default layers (membership=0) should be invisible to filtered queries"
        );
    }

    // ── Behavior 8: query_aabb still returns all (unfiltered, backward compat) ──

    #[test]
    fn query_aabb_unfiltered_returns_all_regardless_of_layers() {
        let mut tree = test_tree();
        let entities = spawn_entities(3);
        tree.insert(
            entities[0],
            small_aabb(10.0, 10.0),
            CollisionLayers::new(0x01, 0x02),
        );
        tree.insert(
            entities[1],
            small_aabb(12.0, 10.0),
            CollisionLayers::new(0x02, 0x01),
        );
        // Entity with default (invisible) layers
        tree.insert(
            entities[2],
            small_aabb(10.0, 12.0),
            CollisionLayers::default(),
        );

        let region = Aabb2D::new(Vec2::new(11.0, 11.0), Vec2::new(10.0, 10.0));
        let mut results = tree.query_aabb(&region);
        results.sort();
        let mut expected = vec![entities[0], entities[1], entities[2]];
        expected.sort();
        assert_eq!(
            results, expected,
            "unfiltered query_aabb should return all spatially overlapping entities"
        );
    }

    // ── Behavior 9: query_aabb_filtered returns only matching layers ──

    #[test]
    fn query_aabb_filtered_returns_only_entities_whose_membership_matches_query_mask() {
        let mut tree = test_tree();
        let entities = spawn_entities(2);
        // entity_a: membership=0x01, mask=0x02
        tree.insert(
            entities[0],
            small_aabb(10.0, 10.0),
            CollisionLayers::new(0x01, 0x02),
        );
        // entity_b: membership=0x02, mask=0x00
        tree.insert(
            entities[1],
            small_aabb(12.0, 10.0),
            CollisionLayers::new(0x02, 0x00),
        );

        let region = Aabb2D::new(Vec2::new(11.0, 10.0), Vec2::new(10.0, 10.0));
        // Query with mask=0x02 → should match entity_b (membership=0x02)
        let results = tree.query_aabb_filtered(&region, CollisionLayers::new(0x00, 0x02));
        assert_eq!(results.len(), 1, "only entity_b should match mask=0x02");
        assert_eq!(results[0], entities[1]);
    }

    #[test]
    fn query_aabb_filtered_with_zero_mask_returns_empty() {
        let mut tree = test_tree();
        let entities = spawn_entities(1);
        tree.insert(
            entities[0],
            small_aabb(10.0, 10.0),
            CollisionLayers::new(0xFF, 0xFF),
        );

        let region = Aabb2D::new(Vec2::new(10.0, 10.0), Vec2::new(5.0, 5.0));
        let results = tree.query_aabb_filtered(&region, CollisionLayers::new(0x00, 0x00));
        assert!(
            results.is_empty(),
            "query with mask=0 should return nothing"
        );
    }

    // ── Behavior 10: query_aabb_filtered with game-like layer config ──

    #[test]
    fn query_aabb_filtered_game_like_layers_from_querier_perspective() {
        // Layer constants (game-agnostic, just bit positions)
        const LAYER_A: u32 = 1 << 0; // 0x01
        const LAYER_B: u32 = 1 << 1; // 0x02
        const LAYER_C: u32 = 1 << 2; // 0x04
        const LAYER_D: u32 = 1 << 3; // 0x08

        let mut tree = test_tree();
        let entities = spawn_entities(4);

        // "querier" entity at (0,0) — membership=LAYER_A, mask=LAYER_B|LAYER_C
        tree.insert(
            entities[0],
            small_aabb(0.0, 0.0),
            CollisionLayers::new(LAYER_A, LAYER_B | LAYER_C),
        );
        // "target_b" entity at (5,0) — membership=LAYER_B, mask=LAYER_A
        tree.insert(
            entities[1],
            small_aabb(5.0, 0.0),
            CollisionLayers::new(LAYER_B, LAYER_A),
        );
        // "target_c" entity at (10,0) — membership=LAYER_C, mask=LAYER_A
        tree.insert(
            entities[2],
            small_aabb(10.0, 0.0),
            CollisionLayers::new(LAYER_C, LAYER_A),
        );
        // "unrelated_d" entity at (15,0) — membership=LAYER_D, mask=LAYER_A
        tree.insert(
            entities[3],
            small_aabb(15.0, 0.0),
            CollisionLayers::new(LAYER_D, LAYER_A),
        );

        let big_region = Aabb2D::new(Vec2::new(7.5, 0.0), Vec2::new(20.0, 5.0));

        // Query "from querier's perspective": mask = LAYER_B | LAYER_C
        let mut results = tree.query_aabb_filtered(
            &big_region,
            CollisionLayers::new(LAYER_A, LAYER_B | LAYER_C),
        );
        results.sort();

        let mut expected = vec![entities[1], entities[2]];
        expected.sort();

        assert_eq!(
            results, expected,
            "should find target_b and target_c but not querier itself or unrelated_d"
        );
    }

    #[test]
    fn query_aabb_filtered_only_one_layer_from_multi_layer_query() {
        const LAYER_B: u32 = 1 << 1; // 0x02
        const LAYER_C: u32 = 1 << 2; // 0x04

        let mut tree = test_tree();
        let entities = spawn_entities(2);

        tree.insert(
            entities[0],
            small_aabb(5.0, 0.0),
            CollisionLayers::new(LAYER_B, 0x01),
        );
        tree.insert(
            entities[1],
            small_aabb(10.0, 0.0),
            CollisionLayers::new(LAYER_C, 0x01),
        );

        let big_region = Aabb2D::new(Vec2::new(7.5, 0.0), Vec2::new(20.0, 5.0));

        // Query with only LAYER_B mask — should only find first entity
        let results = tree.query_aabb_filtered(&big_region, CollisionLayers::new(0x00, LAYER_B));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], entities[0]);
    }

    // ── Behavior 11: query_circle_filtered returns only matching entities in radius ──

    #[test]
    fn query_circle_filtered_returns_only_interacting_entities_within_radius() {
        let mut tree = test_tree();
        let entities = spawn_entities(2);

        // entity_a: membership=0x02, mask=0x01
        tree.insert(
            entities[0],
            small_aabb(5.0, 0.0),
            CollisionLayers::new(0x02, 0x01),
        );
        // entity_b: membership=0x04, mask=0x01
        tree.insert(
            entities[1],
            small_aabb(5.0, 0.0),
            CollisionLayers::new(0x04, 0x01),
        );

        // Query with mask=0x02 only — should match entity_a (membership=0x02)
        let results =
            tree.query_circle_filtered(Vec2::ZERO, 10.0, CollisionLayers::new(0x01, 0x02));
        assert_eq!(results.len(), 1, "only entity_a should match");
        assert_eq!(results[0], entities[0]);
    }

    #[test]
    fn query_circle_filtered_excludes_entity_outside_radius_on_correct_layer() {
        let mut tree = test_tree();
        let entities = spawn_entities(2);

        // entity_a in range: membership=0x02 at (5,0)
        tree.insert(
            entities[0],
            small_aabb(5.0, 0.0),
            CollisionLayers::new(0x02, 0x01),
        );
        // entity_b far away: membership=0x02 at (200,0)
        tree.insert(
            entities[1],
            small_aabb(200.0, 0.0),
            CollisionLayers::new(0x02, 0x01),
        );

        // Both on the right layer, but only entity_a is within radius
        let results =
            tree.query_circle_filtered(Vec2::ZERO, 10.0, CollisionLayers::new(0x01, 0x02));
        assert_eq!(
            results.len(),
            1,
            "only in-range entity_a should be returned"
        );
        assert_eq!(results[0], entities[0]);
    }

    // ════════════════════════════════════════════════════════════════
    // Sweep API tests — cast_circle, cast_ray
    // ════════════════════════════════════════════════════════════════

    // ── Behavior 5: cast_circle finds entity in path ──

    #[test]
    fn cast_circle_finds_entity_in_path() {
        let mut tree = test_tree();
        let entities = spawn_entities(1);

        // Entity at (0, 50) with half_extents (10, 10), membership=0x01
        let aabb = Aabb2D::new(Vec2::new(0.0, 50.0), Vec2::new(10.0, 10.0));
        tree.insert(entities[0], aabb, CollisionLayers::new(0x01, 0x01));

        // Cast circle from origin upward, radius=5
        // Stored AABB: center=50, half=10. Expanded by radius 5 → half=15.
        // Bottom face of expanded AABB at y = 50 - 15 = 35.
        // Ray from y=0 going up → hits at distance 35.0.
        let hits = tree.cast_circle(
            Vec2::new(0.0, 0.0),
            Vec2::Y,
            200.0,
            5.0,
            CollisionLayers::new(0x00, 0x01),
        );

        assert_eq!(hits.len(), 1, "should find one entity in path");
        assert_eq!(hits[0].entity, entities[0]);
        // Position should be safe stop point just before contact (~35.0 along Y)
        assert!(
            (hits[0].position.y - 35.0).abs() < 1.0,
            "position.y should be ~35.0, got {}",
            hits[0].position.y
        );
        assert_eq!(
            hits[0].normal,
            Vec2::NEG_Y,
            "should report bottom face normal"
        );
        // Remaining = max_dist - distance = 200.0 - 35.0 = 165.0
        assert!(
            (hits[0].remaining - 165.0).abs() < 1.0,
            "remaining should be ~165.0, got {}",
            hits[0].remaining
        );
    }

    #[test]
    fn cast_circle_entity_not_on_matching_layer_returns_empty() {
        let mut tree = test_tree();
        let entities = spawn_entities(1);

        // Entity on layer 0x02
        let aabb = Aabb2D::new(Vec2::new(0.0, 50.0), Vec2::new(10.0, 10.0));
        tree.insert(entities[0], aabb, CollisionLayers::new(0x02, 0x02));

        // Cast with mask=0x01 — does not match membership=0x02
        let hits = tree.cast_circle(
            Vec2::new(0.0, 0.0),
            Vec2::Y,
            200.0,
            5.0,
            CollisionLayers::new(0x00, 0x01),
        );

        assert!(
            hits.is_empty(),
            "entity on non-matching layer should not be hit"
        );
    }

    // ── Behavior 6: cast_circle returns hits sorted by distance ──

    #[test]
    fn cast_circle_returns_hits_sorted_nearest_first() {
        let mut tree = test_tree();
        let entities = spawn_entities(2);

        // entity_a at (0, 30) with half_extents (5, 5)
        let aabb_a = Aabb2D::new(Vec2::new(0.0, 30.0), Vec2::new(5.0, 5.0));
        tree.insert(entities[0], aabb_a, CollisionLayers::new(0x01, 0x01));

        // entity_b at (0, 80) with half_extents (5, 5)
        let aabb_b = Aabb2D::new(Vec2::new(0.0, 80.0), Vec2::new(5.0, 5.0));
        tree.insert(entities[1], aabb_b, CollisionLayers::new(0x01, 0x01));

        // Cast ray (radius=0) from origin upward
        // entity_a bottom face at y=25, entity_b bottom face at y=75
        let hits = tree.cast_circle(
            Vec2::new(0.0, 0.0),
            Vec2::Y,
            200.0,
            0.0,
            CollisionLayers::new(0x00, 0x01),
        );

        assert_eq!(hits.len(), 2, "should find both entities");
        assert_eq!(
            hits[0].entity, entities[0],
            "first hit should be entity_a (closer)"
        );
        assert_eq!(
            hits[1].entity, entities[1],
            "second hit should be entity_b (farther)"
        );
    }

    // ── Behavior 7: cast_circle misses entity outside path ──

    #[test]
    fn cast_circle_misses_entity_outside_path() {
        let mut tree = test_tree();
        let entities = spawn_entities(1);

        // Entity far to the right at (100, 50)
        let aabb = Aabb2D::new(Vec2::new(100.0, 50.0), Vec2::new(5.0, 5.0));
        tree.insert(entities[0], aabb, CollisionLayers::new(0x01, 0x01));

        // Cast circle straight up with radius=5 — should not reach entity at x=100
        let hits = tree.cast_circle(
            Vec2::new(0.0, 0.0),
            Vec2::Y,
            200.0,
            5.0,
            CollisionLayers::new(0x00, 0x01),
        );

        assert!(
            hits.is_empty(),
            "entity at x=100 should not be hit by ray along Y axis"
        );
    }

    // ── Behavior 8: cast_circle respects collision layers ──

    #[test]
    fn cast_circle_respects_collision_layers() {
        let mut tree = test_tree();
        let entities = spawn_entities(2);

        // entity_a on layer 0x01, directly in path at (0, 30)
        let aabb_a = Aabb2D::new(Vec2::new(0.0, 30.0), Vec2::new(5.0, 5.0));
        tree.insert(entities[0], aabb_a, CollisionLayers::new(0x01, 0x01));

        // entity_b on layer 0x02, also in path at (0, 60)
        let aabb_b = Aabb2D::new(Vec2::new(0.0, 60.0), Vec2::new(5.0, 5.0));
        tree.insert(entities[1], aabb_b, CollisionLayers::new(0x02, 0x02));

        // Cast with mask=0x01 — should only hit entity_a
        let hits = tree.cast_circle(
            Vec2::new(0.0, 0.0),
            Vec2::Y,
            200.0,
            5.0,
            CollisionLayers::new(0x00, 0x01),
        );

        assert_eq!(hits.len(), 1, "should only hit entity on matching layer");
        assert_eq!(
            hits[0].entity, entities[0],
            "should hit entity_a (layer 0x01)"
        );
    }

    // ── Behavior 9: cast_ray works (zero radius) ──

    #[test]
    fn cast_ray_hits_unexpanded_aabb() {
        let mut tree = test_tree();
        let entities = spawn_entities(1);

        // Entity at (0, 50) with half_extents (10, 10)
        // Unexpanded bottom face at y=40
        let aabb = Aabb2D::new(Vec2::new(0.0, 50.0), Vec2::new(10.0, 10.0));
        tree.insert(entities[0], aabb, CollisionLayers::new(0x01, 0x01));

        // cast_ray = cast_circle with radius 0
        let hits = tree.cast_ray(
            Vec2::new(0.0, 0.0),
            Vec2::Y,
            200.0,
            CollisionLayers::new(0x00, 0x01),
        );

        assert_eq!(hits.len(), 1, "ray should hit entity");
        assert_eq!(hits[0].entity, entities[0]);
        // Ray hits unexpanded AABB at y=40 (no Minkowski expansion)
        assert!(
            (hits[0].position.y - 40.0).abs() < 1.0,
            "position.y should be ~40.0 (unexpanded), got {}",
            hits[0].position.y
        );
        assert_eq!(hits[0].normal, Vec2::NEG_Y);
    }
}
