# Diffusion: document flat-share-per-ring attenuation as deliberate simplification

## Problems addressed

- `audit/hazards/diffusion.md` Issue 1 — Branching-graph topology (diamond paths) is not specified by design; impl picks a simplification. User decision: keep the simplification; document it.

## Context

Split from the original `diffusion-flat-share-per-ring.md`. The test addition (new `diamond_topology.rs` pinning the behavior) lives at `audit/remediations/diffusion-diamond-topology-test.md`. This file retains only the design-doc + architecture-doc updates.

## Remediation

### Design-doc update

Open `docs/todos/detail/mod-system-design/hazards/diffusion.md`. Add a new subsection under §Expected Behaviors or §Attenuation:

> **Ring attenuation is flat per ring, not per-path**
>
> When damage diffuses through the cell grid, each ring gets a single uniform share of the previous ring's total damage. If a cell is reachable via multiple paths from the hit cell (a "diamond" topology where cell D is adjacent to both B and C, which are both adjacent to the hit cell A), D takes damage ONCE at its ring's share — it does NOT accumulate from each path.
>
> This is a deliberate simplification. Per-path accumulation is a possible future tuning knob but is not current behavior.

### Architecture-doc note

Open `docs/architecture/effects.md` or the appropriate adjacent doc. Add a line noting the attenuation algorithm as load-bearing:

> Diffusion uses "frontier-ratio" attenuation: `per_neighbor_ring_N = previous_ring_total * share_frac / current_ring_size`. This treats each ring as a uniform distribution of the previous ring's shared damage, not a per-parent aggregation. Flat-share is correct for the common gameplay topologies (linear chains, small branches); diamond topologies underweight reached-via-multiple-paths cells by design.

No code change. No test change (test addition is split off to `diffusion-diamond-topology-test.md`).

## Scope note

This remediation is part of the broader design-doc-alignment sweep.
