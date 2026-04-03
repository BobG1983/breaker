# Visual Composition Model

Every visual in the game is composed from generic primitives via two paths:

## 1. Recipe Path (data-driven)

All params authored in RON. `ExecuteRecipe { recipe, position }`. Steps can use phase `offset` for spatial layout relative to the anchor. For fully-authored effects where all visual params are known at author time.

## 2. Direct Primitive Path (code-driven)

Game sends typed primitive messages directly (`SpawnExpandingRing`, `TriggerScreenShake`, etc.). For effects with runtime-variable params (radius scales with stacks, beam endpoints depend on entity positions).

No override system bridges the two. Each effect chooses the path that fits. Most effects use recipes; effects with runtime variation use direct primitives.

## Five Visual Categories

1. **Entity visuals** — gameplay domains send `AttachVisuals { entity, config }` carrying an `EntityVisualConfig` struct (shape, color, glow params, aura, trail). The VFX crate receives the message and attaches mesh, material, shaders, aura, and trail. No entity-type-specific rendering code.

2. **Effect VFX** — composed from visual primitives via RON visual recipes or direct primitive messages. A "shockwave" isn't a rendering module — it's a recipe: "expanding ring + radial distortion + small screen shake."

3. **Evolution VFX** — same recipe system with more/fancier steps and more dramatic parameters. If an evolution needs a genuinely new visual capability, that's a new *primitive*, not a new effect module.

4. **Dynamic visuals** — all dynamic visual state (bolt speed → trail length, piercing → spike count) is driven through the **modifier system**. Gameplay sends `SetModifier` (overwrites by source key, per-frame) or `AddModifier` (stacks with diminishing returns, chip effects). No `*RenderState` bridge components.

5. **Event VFX** — damage, death, hit, lock-unlock, healing pulse, etc. are triggered via `ExecuteRecipe` at event time. Not configured at attach time.

New rendering code is only needed when a genuinely new visual primitive is required. Combinations of existing primitives are pure data.

## Anchored Primitives Enable Entity-Relative Recipes

Most entity-relative effects are recipe-able via `Source`/`Target` entity references on `ExecuteRecipe`. Anchored primitives track entity positions per-frame and self-despawn when the entity despawns.

| Effect | How It Works |
|--------|-------------|
| **Gravity Well** | Recipe: `AnchoredDistortion(entity: Source)` + `AnchoredGlowMotes(entity: Source, inward: true)`. Fired with `source: Some(well_entity)`. |
| **Tether Beam** | Recipe: `AnchoredBeam(entity_a: Source, entity_b: Target)`. Fired with `source: Some(bolt_a), target: Some(bolt_b)`. |
| **Attraction** | Recipe: `AnchoredArc(entity_a: Source, entity_b: Target, curvature: 0.3)`. Fired with source/target. |
| **Ramping Damage ring** | Recipe: `AnchoredRing(entity: Source)`. Ring rotation speed driven by modifier. |
| **Chain Lightning** | Same "electric_arc" recipe fired N times: source→target1, target1→target2, etc. Game iterates target chain and sends one `ExecuteRecipe` per arc. |
| **ArcWelder web** | Same "tether" recipe fired per bolt pair. Game iterates bolt sequence and sends one per consecutive pair. |

## Effects With Game-Specific Logic (partial code-composition)

Some effects need game-specific systems for parts of their VFX, while still using recipes/primitives for other parts:

| Effect | Recipe Part | Game-Specific Part |
|--------|------------|-------------------|
| **Shield barrier** | `GlowLine` for base barrier + `Fracture(entity: Source)` on last charge break | Crack pattern per charge loss (game drives damage_recipe on the barrier entity based on charge count) |
| **Circuit Breaker indicator** | `AnchoredRing` or `GlowMotes` for charge nodes | 3-node triangle layout, which nodes are lit (game tracks perfect bump count) |
| **Split Decision fission** | Recipe: `Split(entity: Source, axis)` + `SparkBurst` + `ExpandingRing`. Game spawns two bolts beneath the split halves. | Game determines split axis and spawns bolt entities at the half positions. |
| **FlashStep** | Departure recipe: `Disintegrate(entity: Source)` + Beam (light-streak). Arrival recipe: `ExpandingRing` + `RadialDistortion` + `SparkBurst`. | Game handles breaker teleport (position change between departure and arrival recipes). |
| **Time Penalty line to HUD** | Beam or ElectricArc for the energy streak | World-to-screen coordinate mapping (game computes HUD position, sends Beam with resolved start/end) |
