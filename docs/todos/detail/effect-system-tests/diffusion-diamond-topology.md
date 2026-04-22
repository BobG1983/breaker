# Diffusion: diamond-topology regression test

## Problems addressed

- `audit/hazards/diffusion.md` Issue 1 — Document and PIN the flat-share-per-ring attenuation behavior. Doc portion split to `doc-fix-diffusion-flat-share-documentation.md`; this file retains the test addition.

## Prerequisite

Depends on TODO #1 (`rantzsoft_dmg` crate) landing. After that, Diffusion's BFS lives in `diffusion_mutate_damage` (a `MessageMutator<DamageDealt<Cell>>` chain member in `DeathPipelineSystems::MutateDamage`), not inside `apply_damage_to_cells` (deleted). This test seeds `DamageDealt<Cell>` and asserts on the mutated messages + emitted ring siblings.

Also depends on `fix-broken-protocols-and-hazards/grid.md` (orthogonal adjacency) if the test uses `CellGridPosition` — pick topology coordinates accordingly.

## Remediation

Add `mutators/hazards/diffusion/tests/diamond_topology.rs`:

1. Spawn a grid with cells A, B, C, D such that A is adjacent to B AND C, and D is adjacent to both B AND C (not A). Example grid positions under orthogonal-only adjacency:
   - A at `(1, 0)`
   - B at `(0, 0)`
   - C at `(2, 0)`
   - D at `(1, -1)` (adjacent to both B and C via column shift — adjust specific coordinates to make the topology work with the orthogonal rule).
2. Activate Diffusion with `share_frac: 0.5`, `depth: 2`.
3. Drive a `DamageDealt<Cell>` for cell A with `amount: 100`.
4. Assert:
   - A takes `100 * (1 - 0.5) = 50` damage.
   - B and C each take `100 * 0.5 / 2 = 25` damage.
   - D takes `previous_ring_total * 0.5 / next_ring_size` damage — compute the expected value for the specific topology picked and pin it exactly.
   - Critically: D takes damage ONCE, not twice.
5. Add a comment noting what per-path accumulation would produce (e.g., "D would take 2× the current value") so future readers understand why the assertion pins the lower number.

## Scope note

Pre-grid-orthogonal (if that migration hasn't landed): construct the topology using radius-based adjacency — carefully pick world coordinates so exactly the right cells are within `70` units of each other. The behavioral assertion is identical either way.
