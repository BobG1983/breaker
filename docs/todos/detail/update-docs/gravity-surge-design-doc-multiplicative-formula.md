# Gravity Surge: design doc uses multiplicative strength formula (matches impl)

## Problems addressed

- `audit/hazards/gravity_surge.md` Issue 2 \u2014 Design says strength = `base + per_level_diminishing * sqrt(stack - 1)` (additive). Impl uses `base * (1 + per_level_frac * sqrt(stack - 1))` (multiplicative). Functionally equivalent if tuned right. Impl's fractional scaling parameter is more intuitive: `per_level_strength_frac` is a "how much extra per stack, as a fraction of base."

## Remediation

Open `docs/todos/detail/mod-system-design/hazards/gravity_surge.md` \u00a7Config Resource and \u00a7Expected Behaviors.

\u00a7Config Resource \u2014 replace:

> `strength_per_level_diminishing: f32` \u2014 per-stack additive strength increment.

with:

> `per_level_strength_frac: f32` \u2014 fractional scaling per stack. Strength at stack N is `base_strength * (1.0 + per_level_strength_frac * sqrt(N - 1))`. At stack 1 the multiplier is 1.0 (strength = base); at stack 3 and `per_level_strength_frac = 0.5`, multiplier is `1 + 0.5 * sqrt(2) \u2248 1.707`.

\u00a7Expected Behaviors 3 \u2014 rewrite the worked example:

> 3. **Strength scales multiplicatively at stack 3**
>    - Given: `base_strength: 100.0`, `per_level_strength_frac: 0.5`, stack 3
>    - When: `strength_for_stacks(3)` is called
>    - Then: `100.0 * (1.0 + 0.5 * sqrt(2)) \u2248 170.7`

Similarly update Behavior 1 (stack 1):

> 1. **Strength at stack 1 equals base**
>    - Given: `base_strength: 100.0`, any `per_level_strength_frac`, stack 1
>    - When: `strength_for_stacks(1)` is called
>    - Then: `100.0 * (1.0 + frac * sqrt(0)) = 100.0` (sqrt(0) = 0 cancels the per-level term).

No code change. No test change. Doc-only.
