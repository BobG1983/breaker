# Diffusion — flat-share-per-ring attenuation in the canonical design doc

## Target file

`docs/design/hazards/diffusion.md` (promoted during this sweep).

## What the target doc must say

§ Expected Behaviors — new subsection under attenuation:

> **Ring attenuation is flat per ring, not per-path**
>
> When damage diffuses through the cell grid, each ring gets a single uniform share of the previous ring's total damage. If a cell is reachable via multiple paths from the hit cell (a "diamond" topology where cell D is adjacent to both B and C, which are both adjacent to the hit cell A), D takes damage ONCE at its ring's share — it does NOT accumulate from each path.
>
> This is a deliberate simplification. Per-path accumulation is a possible future tuning knob but is not current behaviour.

§ Algorithm — pin the formula as load-bearing:

> Diffusion uses "frontier-ratio" attenuation: `per_neighbor_ring_N = previous_ring_total * share_frac / current_ring_size`. Each ring is a uniform distribution of the previous ring's shared damage, not a per-parent aggregation. Flat-share is correct for the common gameplay topologies (linear chains, small branches); diamond topologies underweight reached-via-multiple-paths cells by design.

§ Pipeline position:

> Diffusion is a **pre-apply damage mutator** — it participates in the `MutateDamage` chain inside `DeathPipelineSystems::MutateDamage`, transforming `DamageDealt<Cell>` messages before they reach `ApplyDamage`. Lives in `mutators/hazards/diffusion/` alongside other mutator-style hazards (post-TODO #2).

## What the target doc must NOT say

- Do not describe Diffusion as inline-in-cells-domain (`cells/systems/apply_damage_to_cells`) — post-TODO #2 it lives in `mutators/hazards/diffusion/` as a `MessageMutator<DamageDealt<Cell>>` participant.
- Do not call per-path accumulation "correct" or the flat-share a bug — it is a deliberate simplification.

## Why

Ring attenuation semantics are load-bearing for mechanical understanding and test authoring. Documenting flat-share vs per-path explicitly prevents the next reader (writer, reviewer, or design agent) from treating diamond-topology underweighting as a bug.
