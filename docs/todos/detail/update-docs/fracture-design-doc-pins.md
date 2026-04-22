# Fracture — cap-at-4 + fixed-order position selection in the canonical design doc

## Target file

`docs/design/hazards/fracture.md` (promoted during this sweep).

## What the target doc must say

§ Stacking Behavior (or equivalent) — pin the cap:

> Fracture caps debris spawns at 4 per death — one per orthogonal neighbor position. Stack values beyond the cap (stack 5+) do not add more debris per death; they have no effect on spawn count. If the design later wants higher counts (e.g., diagonals for stack 5+), expand the offset set.

§ Edge Cases — position-selection entry:

> Fracture uses a fixed-order offset list `[(+W, 0), (-W, 0), (0, +H), (0, -H)]` and takes the first `count` entries. Position selection is fully deterministic without RNG. If the count equals 4 (all slots used), order doesn't matter for the final scene. If the count is less than 4, the first `count` slots (right, then left, then up, then down) are used preferentially.
>
> Future tuning could introduce RNG-driven selection (pick `count` of 4 at random) using the project's seeded `GameRng` for determinism. Current impl doesn't need this; the fixed order is sufficient and simpler.

## Pipeline position (dmg crate)

- **Trigger**: reads `Destroyed<Cell>` from the `rantzsoft_dmg` crate — every cell death produces a message, Fracture reacts to it.
- **Not a damage emitter or mutator.** Fracture does NOT participate in any `DeathPipelineSystems` set. It spawns debris cells via the cells-domain `Cell::builder().debris()` path after the kill has already applied.
- **Ordering**: runs `.after(DeathPipelineSystems::ApplyKill)` so the primary cell is fully destroyed before debris spawn logic evaluates the grid.
- **No** `DamageDealt<T>` / `HealDealt<T>` / `DamageBoostStack` involvement.

## Why

Cap and position-selection order are both deterministic design choices that must be visible in the canonical doc — the tests and impl already pin them; the doc must match.
