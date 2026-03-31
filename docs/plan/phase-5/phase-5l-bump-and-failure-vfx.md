# 5l: Bump Grade & Failure State VFX

**Goal**: Wire gameplay events to VFX. Bump grades fire breaker recipes. Failure states fire screen effects + recipes. Remove all floating text feedback.

Architecture: `docs/architecture/rendering/communication.md` — Game-Side VFX Orchestration

## What to Build

### 1. Bump Grade VFX

On `BumpPerformed { grade }`, the breaker/ domain reads the breaker's rendering config and sends `ExecuteRecipe` with the appropriate recipe:
- Perfect → `perfect_bump_recipe` (gold flash + sparks + micro-shake)
- Early → `early_bump_recipe` (dim flash + few sparks)
- Late → `late_bump_recipe` (minimal flash)
- Whiff → nothing (silence IS feedback)

### 2. Bolt Lost VFX

`bolt_lost` system sends VFX after gameplay logic:
- `ExecuteRecipe` with bolt's `death_recipe` (exit streak)
- `TriggerSlowMotion { factor: 0.3, duration: 0.3 }`
- `TriggerDesaturation { target_factor: 0.7, duration: 0.3 }`

### 3. Shield Absorption VFX

When shield absorbs a bolt-loss:
- Shield barrier damage recipe fires (sparks + crack seed)
- Bolt bounces back
- No slow-mo — the save is instant and satisfying

### 4. Life Lost VFX

`life_lost::fire()` sends:
- `TriggerSlowMotion { factor: 0.2, duration: 0.5 }` (longer than bolt lost)
- `TriggerVignettePulse` (danger flash)
- Life orb dissolve recipe (from 5q HUD)

### 5. Run Won / Run Over VFX

- Run won: `TriggerSlowMotion { factor: 0.0 }` (freeze-frame) + `TriggerScreenFlash` (white) + transition
- Run over: `TriggerSlowMotion { factor: 0.15 }` (extended) + `TriggerDesaturation { target_factor: 1.0 }` (full monochrome)

### 6. Time Expired VFX

Timer wall shatters (recipe), red-orange vignette pulse, desaturation from edges inward.

### 7. Remove Floating Text

Remove all existing floating text feedback: "PERFECT", "EARLY", "LATE", "WHIFF", "BOLT LOST". Replaced by the visual-only systems above.

## Dependencies

- **Requires**: 5g (bolt visuals for death recipe), 5h (breaker recipes for bump grades), 5j (shield barrier), 5k (screen effects: shake, slow-mo, vignette, flash, desaturation)

## Verification

- No floating text remains
- Perfect bump produces gold flash + sparks + micro-shake
- Whiff produces zero visual feedback
- Bolt lost triggers slow-mo + streak + desaturation
- Shield absorption triggers barrier damage without slow-mo
- Run end states produce appropriate VFX
- All existing tests pass
