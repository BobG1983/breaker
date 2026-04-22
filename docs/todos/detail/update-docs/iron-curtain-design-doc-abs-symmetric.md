# Iron Curtain — abs-symmetric falloff in the canonical design doc

## Target file

`docs/design/protocols/iron_curtain.md` (promoted during this sweep).

## What the target doc must say

Under § Edge Cases, the behaviour for cells below the breaker:

> Cells at distance `|cell.y - breaker.y|` within the falloff radius are damaged symmetrically. In practice, cells rarely spawn below the breaker, but the wave does not explicitly mask them — if a cell exists below, it receives damage consistent with the symmetric falloff formula.

The § Damage Falloff Formula must use `|cell.y - breaker.y|` (abs-symmetric).

## Pipeline position (dmg crate)

- **Trigger**: reads `BoltLost` message (bolt-lifecycle — NOT a `DeathPipelineSystems` set).
- **Emits**: `DamageDealt<Cell>` in `DeathPipelineSystems::EmitDamage` for each cell within falloff radius.
- **Source**: `"protocol:iron_curtain"`.
- **No** `DamageBoostStack`/`VulnerableStack` interaction — the emitted damage flows through the standard chain (`ApplyDamageBoosts` → `MutateDamage` → `ApplyVulnerable` → `ApplyDamage`) like any other damage source.

## What the target doc must NOT say

Do not describe the wave as "spreading upward only". The impl and the `cells_below_breaker_are_damaged_by_abs_symmetric_falloff` test (`tests/on_bolt_lost.rs:294-313`) pin abs-symmetric behaviour.

## Why

The falloff formula and the wording must agree. `|cell.y - breaker.y|` is abs-symmetric; "upward only" contradicts it.
