//! Lock resolution: topological sort of lock dependencies and spawning of
//! locked cells in dependency order.

use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;

use super::system::GridCellContext;
use crate::{prelude::*, state::run::node::definition::LockMap};

/// Pass 2: resolves lock dependencies via topological sort and spawns locked
/// cells in dependency order. Returns the additional required-to-clear count.
pub(super) fn resolve_and_spawn_locks(
    ctx: &GridCellContext<'_>,
    commands: &mut Commands,
    entity_map: &mut HashMap<(usize, usize), Entity>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> u32 {
    let Some(ref locks) = ctx.layout.locks else {
        return 0;
    };
    if locks.is_empty() {
        return 0;
    }

    // Filter out lock keys that reference cells already spawned in Pass 1.
    let effective_locks: LockMap = locks
        .iter()
        .filter(|&(&(r, c), _)| !entity_map.contains_key(&(r, c)))
        .map(|(&k, v)| (k, v.clone()))
        .collect();

    let (sorted, cyclic) = topological_sort_locks(&effective_locks);
    let mut required_count = 0u32;

    // Spawn sorted (acyclic) locked cells.
    for coord in sorted {
        required_count += spawn_locked_cell(
            ctx,
            commands,
            &effective_locks,
            entity_map,
            coord,
            meshes,
            materials,
        );
    }

    // Spawn cyclic entries as unlocked fallbacks.
    for coord in cyclic {
        required_count +=
            spawn_unlocked_fallback(ctx, commands, entity_map, coord, meshes, materials);
    }

    required_count
}

/// Spawns a single locked cell, resolving its lock targets to entity IDs.
/// Returns 1 if the cell is required-to-clear, 0 otherwise.
fn spawn_locked_cell(
    ctx: &GridCellContext<'_>,
    commands: &mut Commands,
    effective_locks: &LockMap,
    entity_map: &mut HashMap<(usize, usize), Entity>,
    coord: (usize, usize),
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> u32 {
    let (row_idx, col_idx) = coord;
    let Some(alias) = ctx
        .layout
        .grid
        .get(row_idx)
        .and_then(|row| row.get(col_idx))
    else {
        return 0;
    };
    if alias == "." {
        return 0;
    }
    let Some(def) = ctx.registry.get(alias) else {
        return 0;
    };

    let pos = ctx.params.cell_pos(row_idx, col_idx);
    let scaled_hp = ctx.compute_hp(def.toughness);

    // Resolve lock targets to entity IDs.
    let targets = &effective_locks[&coord];
    let resolved: Vec<Entity> = targets
        .iter()
        .filter_map(|target| {
            entity_map.get(target).copied().or_else(|| {
                debug!(
                    "layout '{}': lock target ({}, {}) for cell ({}, {}) \
                     has no spawned entity — skipping target",
                    ctx.layout.name, target.0, target.1, row_idx, col_idx
                );
                None
            })
        })
        .collect();

    // If no targets resolved, spawn without Locked (graceful degradation).
    let entity_id = if resolved.is_empty() {
        Cell::builder()
            .definition(def)
            .position(pos)
            .dimensions(ctx.params.cell_width, ctx.params.cell_height)
            .override_hp(scaled_hp)
            .alias(alias.clone())
            .rendered(meshes, materials)
            .spawn(commands)
    } else {
        Cell::builder()
            .definition(def)
            .position(pos)
            .dimensions(ctx.params.cell_width, ctx.params.cell_height)
            .override_hp(scaled_hp)
            .alias(alias.clone())
            .locked(resolved)
            .rendered(meshes, materials)
            .spawn(commands)
    };

    entity_map.insert(coord, entity_id);
    u32::from(def.required_to_clear)
}

/// Spawns a cell as an unlocked fallback (used for cyclic lock entries).
/// Returns 1 if the cell is required-to-clear, 0 otherwise.
fn spawn_unlocked_fallback(
    ctx: &GridCellContext<'_>,
    commands: &mut Commands,
    entity_map: &mut HashMap<(usize, usize), Entity>,
    coord: (usize, usize),
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> u32 {
    let (row_idx, col_idx) = coord;
    let Some(alias) = ctx
        .layout
        .grid
        .get(row_idx)
        .and_then(|row| row.get(col_idx))
    else {
        return 0;
    };
    if alias == "." {
        return 0;
    }
    let Some(def) = ctx.registry.get(alias) else {
        return 0;
    };

    let pos = ctx.params.cell_pos(row_idx, col_idx);
    let scaled_hp = ctx.compute_hp(def.toughness);

    let entity_id = Cell::builder()
        .definition(def)
        .position(pos)
        .dimensions(ctx.params.cell_width, ctx.params.cell_height)
        .override_hp(scaled_hp)
        .alias(alias.clone())
        .rendered(meshes, materials)
        .spawn(commands);

    entity_map.insert(coord, entity_id);
    u32::from(def.required_to_clear)
}

/// Performs a topological sort of lock entries using Kahn's algorithm.
///
/// Returns `(sorted, cyclic)`:
/// - `sorted`: lock keys in an order where all dependencies come first.
/// - `cyclic`: lock keys that participate in a cycle (could not be ordered).
type GridCoord = (usize, usize);

fn topological_sort_locks(locks: &LockMap) -> (Vec<GridCoord>, Vec<GridCoord>) {
    // Build in-degree map. Only edges between lock-map keys matter.
    let mut in_degree: HashMap<(usize, usize), usize> = HashMap::new();
    // dependents[target] = list of lock keys that depend on target
    let mut dependents: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();

    for &key in locks.keys() {
        in_degree.entry(key).or_insert(0);
    }

    for (&key, targets) in locks {
        for target in targets {
            // Only count edges where the target is also a lock key.
            if locks.contains_key(target) {
                *in_degree.entry(key).or_insert(0) += 1;
                dependents.entry(*target).or_default().push(key);
            }
        }
    }

    // Seed the queue with all lock keys that have in-degree 0.
    let mut queue: VecDeque<(usize, usize)> = in_degree
        .iter()
        .filter(|entry| *entry.1 == 0)
        .map(|entry| *entry.0)
        .collect();

    let mut sorted = Vec::with_capacity(locks.len());

    while let Some(node) = queue.pop_front() {
        sorted.push(node);
        if let Some(deps) = dependents.get(&node) {
            for &dep in deps {
                if let Some(deg) = in_degree.get_mut(&dep) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(dep);
                    }
                }
            }
        }
    }

    // Remaining nodes with in-degree > 0 are cyclic.
    let cyclic: Vec<(usize, usize)> = in_degree
        .into_iter()
        .filter(|(_, deg)| *deg > 0)
        .map(|(k, _)| k)
        .collect();

    if !cyclic.is_empty() {
        debug!(
            "topological_sort_locks: detected cycle among {} lock entries: {:?}",
            cyclic.len(),
            cyclic
        );
    }

    (sorted, cyclic)
}
