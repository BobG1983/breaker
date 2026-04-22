# Gravity Surge — multiplicative strength formula in the canonical design doc

## Target file

`docs/design/hazards/gravity_surge.md` (promoted during this sweep).

## What the target doc must say

§ Config Resource — replace additive-scaling field with fractional-scaling field:

> `per_level_strength_frac: f32` — fractional scaling per stack. Strength at stack N is `base_strength * (1.0 + per_level_strength_frac * sqrt(N - 1))`. At stack 1 the multiplier is 1.0 (strength = base); at stack 3 and `per_level_strength_frac = 0.5`, multiplier is `1 + 0.5 * sqrt(2) ≈ 1.707`.

§ Expected Behaviors — worked examples:

> 1. **Strength at stack 1 equals base**
>    - Given: `base_strength: 100.0`, any `per_level_strength_frac`, stack 1
>    - When: `strength_for_stacks(1)` is called
>    - Then: `100.0 * (1.0 + frac * sqrt(0)) = 100.0` (sqrt(0) = 0 cancels the per-level term).
>
> 3. **Strength scales multiplicatively at stack 3**
>    - Given: `base_strength: 100.0`, `per_level_strength_frac: 0.5`, stack 3
>    - When: `strength_for_stacks(3)` is called
>    - Then: `100.0 * (1.0 + 0.5 * sqrt(2)) ≈ 170.7`

§ Cross-Domain Dependencies — Gravity Surge applies force via the bolt-force pipeline:

> The computed strength becomes the magnitude of an `ApplyBoltForce { bolt, force: Vec2 }` message (bolt domain consumer aggregates and integrates in `FixedUpdate` before `BoltSystems::IntegrateMotion`). Gravity Surge does NOT write `Velocity2D` directly — the force-pipeline owns that write.

## What the target doc must NOT say

- Do not describe strength as additive (`base + per_level_diminishing * sqrt(stack - 1)`).
- Do not reference the deprecated `strength_per_level_diminishing` field name.
- Do not describe Gravity Surge as writing `Velocity2D` directly — that path is retired.

## Pipeline position (dmg crate)

- **Not in the death pipeline.** Gravity Surge does not participate in any `DeathPipelineSystems` set.
- **Trigger**: reads `Destroyed<Cell>` from the `rantzsoft_dmg` crate — spawns a gravity-well entity at the killed cell's position with lifetime + strength per stack.
- **Force emission**: emits `ApplyBoltForce { bolt, force: Vec2 }` in `FixedUpdate` before `BoltSystems::IntegrateMotion` (bolt-domain consumer — NOT part of the damage pipeline).
- **No** `DamageBoostStack`/`VulnerableStack` interaction. **No** direct `Velocity2D` write.

## Why

The multiplicative formula matches the impl and the fractional parameter is more intuitive. The `ApplyBoltForce` pipeline is the canonical path for force-emitting mechanics.
