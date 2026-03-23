//! Incremental quadtree spatial index for broad-phase collision queries.

use bevy::{platform::collections::HashSet, prelude::*};

use crate::aabb::Aabb2D;

/// Internal node representation for the quadtree.
enum QuadNode {
    Leaf {
        items: Vec<(Entity, Aabb2D)>,
    },
    Branch {
        children: Box<[QuadNode; 4]>,
        /// Items stored at this branch level because they span multiple child
        /// quadrants and cannot be pushed down.
        items: Vec<(Entity, Aabb2D)>,
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

fn insert_into_node(
    node: &mut QuadNode,
    node_bounds: &Aabb2D,
    entity: Entity,
    bounds: Aabb2D,
    max_items_per_leaf: usize,
    max_depth: usize,
    depth: usize,
) {
    match node {
        QuadNode::Leaf { items } => {
            items.push((entity, bounds));
            // Split if over capacity and we haven't reached max depth
            if items.len() > max_items_per_leaf && depth < max_depth {
                let old_items: Vec<(Entity, Aabb2D)> = std::mem::take(items);
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
                for (e, b) in old_items {
                    insert_into_node(
                        node,
                        node_bounds,
                        e,
                        b,
                        max_items_per_leaf,
                        max_depth,
                        depth,
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
                    max_items_per_leaf,
                    max_depth,
                    depth + 1,
                );
            } else {
                // Spans multiple quadrants — store at this branch level
                items.push((entity, bounds));
            }
        }
    }
}

fn remove_from_node(node: &mut QuadNode, entity: Entity) -> bool {
    match node {
        QuadNode::Leaf { items } => {
            if let Some(pos) = items.iter().position(|(e, _)| *e == entity) {
                items.swap_remove(pos);
                return true;
            }
            false
        }
        QuadNode::Branch { children, items } => {
            // Check branch-level items first
            if let Some(pos) = items.iter().position(|(e, _)| *e == entity) {
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
            for &(entity, ref item_bounds) in items {
                if item_bounds.overlaps(region) && seen.insert(entity) {
                    results.push(entity);
                }
            }
        }
        QuadNode::Branch { children, items } => {
            // Check branch-level items
            for &(entity, ref item_bounds) in items {
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

    /// Inserts an entity with the given bounds into the tree.
    pub fn insert(&mut self, entity: Entity, bounds: Aabb2D) {
        insert_into_node(
            &mut self.root,
            &self.bounds,
            entity,
            bounds,
            self.max_items_per_leaf,
            self.max_depth,
            0,
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
    #[must_use]
    pub fn query_aabb(&self, region: &Aabb2D) -> Vec<Entity> {
        let mut results = Vec::new();
        let mut seen = HashSet::new();
        query_aabb_node(&self.root, &self.bounds, region, &mut results, &mut seen);
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
        collect_matching_items(&self.root, &candidate_set, &mut |entity, item_bounds| {
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
}

fn collect_matching_items(
    node: &QuadNode,
    candidates: &HashSet<Entity>,
    callback: &mut impl FnMut(Entity, &Aabb2D),
) {
    match node {
        QuadNode::Leaf { items } => {
            for &(entity, ref bounds) in items {
                if candidates.contains(&entity) {
                    callback(entity, bounds);
                }
            }
        }
        QuadNode::Branch { children, items } => {
            for &(entity, ref bounds) in items {
                if candidates.contains(&entity) {
                    callback(entity, bounds);
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
        tree.insert(entities[0], small_aabb(10.0, 10.0));
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
            tree.insert(e, small_aabb(i as f32 * 10.0, 0.0));
        }
        assert_eq!(tree.len(), 5);
    }

    #[test]
    fn remove_decreases_len() {
        let mut tree = test_tree();
        let entities = spawn_entities(3);
        tree.insert(entities[0], small_aabb(0.0, 0.0));
        tree.insert(entities[1], small_aabb(10.0, 0.0));
        tree.insert(entities[2], small_aabb(20.0, 0.0));

        let removed = tree.remove(entities[1]);
        assert!(removed);
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn remove_returns_false_for_missing_entity() {
        let mut tree = test_tree();
        let entities = spawn_entities(2);
        tree.insert(entities[0], small_aabb(0.0, 0.0));
        // entities[1] was never inserted
        let removed = tree.remove(entities[1]);
        assert!(!removed);
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn query_aabb_finds_overlapping_entity() {
        let mut tree = test_tree();
        let entities = spawn_entities(1);
        tree.insert(entities[0], small_aabb(10.0, 10.0));

        let query_region = Aabb2D::new(Vec2::new(10.0, 10.0), Vec2::new(5.0, 5.0));
        let results = tree.query_aabb(&query_region);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], entities[0]);
    }

    #[test]
    fn query_aabb_misses_distant_entity() {
        let mut tree = test_tree();
        let entities = spawn_entities(1);
        tree.insert(entities[0], small_aabb(10.0, 10.0));

        let query_region = Aabb2D::new(Vec2::new(100.0, 100.0), Vec2::new(5.0, 5.0));
        let results = tree.query_aabb(&query_region);
        assert!(results.is_empty());
    }

    #[test]
    fn query_aabb_finds_multiple_overlapping_entities() {
        let mut tree = test_tree();
        let entities = spawn_entities(3);
        tree.insert(entities[0], small_aabb(0.0, 0.0));
        tree.insert(entities[1], small_aabb(2.0, 0.0));
        tree.insert(entities[2], small_aabb(0.0, 2.0));

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
        tree.insert(entities[0], small_aabb(5.0, 0.0));

        let results = tree.query_circle(Vec2::ZERO, 10.0);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], entities[0]);
    }

    #[test]
    fn query_circle_misses_entity_out_of_range() {
        let mut tree = test_tree();
        let entities = spawn_entities(1);
        tree.insert(entities[0], small_aabb(100.0, 0.0));

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
            tree.insert(e, small_aabb(i as f32 * 10.0, 0.0));
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
        tree.insert(entities[0], small_aabb(10.0, 10.0));
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
            tree.insert(e, small_aabb(i as f32 * 0.5, i as f32 * 0.5));
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
        tree.insert(entities[0], big_aabb);

        // Also insert some small entities to potentially trigger splits
        for (i, &e) in entities[1..6].iter().enumerate() {
            tree.insert(e, small_aabb(200.0 + i as f32, 200.0));
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
}
