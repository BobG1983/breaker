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
        children: Box<[Self; 4]>,
        /// Items stored at this branch level because they span multiple child
        /// quadrants and cannot be pushed down.
        items:    Vec<(Entity, Aabb2D, CollisionLayers)>,
    },
}

/// Spatial index mapping entities to `Aabb2D` bounds, supporting incremental
/// updates and spatial queries.
///
/// Items are stored in leaf nodes and split when a leaf exceeds
/// `max_items_per_leaf`, up to `max_depth` levels.
pub struct Quadtree {
    root:               QuadNode,
    bounds:             Aabb2D,
    max_items_per_leaf: usize,
    max_depth:          usize,
    len:                usize,
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

#[derive(Clone, Copy)]
struct TreeConfig {
    max_items_per_leaf: usize,
    max_depth:          usize,
    depth:              usize,
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
                    insert_into_node(node, node_bounds, e, b, l, cfg);
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
                        max_depth:          cfg.max_depth,
                        depth:              cfg.depth + 1,
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
                max_depth:          self.max_depth,
                depth:              0,
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
