# 5k: Highlight Moments

## Summary

Replace text-based highlight popups with contextual emphasis per DR-4. Each highlight type gets a GlitchText label (Text2d entity with child GlitchMaterial overlay) at the relevant screen location plus per-highlight game element VFX. GlitchText uses the `glitch_text.wgsl` shader built in 5e. Labels last ~0.3s with Display typography, punch-scale animation, and scan-line dissolve fade. Remove all existing plain floating text highlight popups.

## Context

The current highlight system spawns plain `Text2d` entities with `PunchScale` + `FadeOut` components when a memorable moment triggers during gameplay. These are generic floating text — they do not match the game's visual identity and violate Pillar 4 (all feedback is visual-only, no plain text labels).

The replacement system uses the `GlitchMaterial` (built in 5e) to render stylized text that looks like projected neon light rather than a UI overlay. Each highlight type also triggers a game element VFX (barrier flash, cell field pulse, trail intensification, etc.) that reinforces what happened at the relevant screen location. The text IS spectacle — projected, glitchy, brief.

Key architecture change from the LEGACY plan: the old plan used a `SpawnGlitchText` message sent to the VFX crate, which would spawn the text entity. The new plan has the game-side highlight system spawn the GlitchText entity directly — no message to a crate, no recipe. The GlitchMaterial and GlitchUniforms types live in `visuals/materials/glitch.rs` (built in 5e). The highlight system creates the Text2d parent + GlitchMaterial child overlay directly in game code.

## What to Build

### 1. GlitchText Spawn Function

A reusable function that spawns a GlitchText entity at a given world position:

```
fn spawn_glitch_text(
    commands: &mut Commands,
    text: &str,
    position: Vec2,
    color: Hue,
    size: f32,         // font size
    duration: f32,     // seconds before despawn
    ...
) -> Entity
```

Implementation:
- Spawn parent entity with `Text2d` component at `position`, using Display-level typography (large, bold, geometric sans-serif font). Set font color to `color`.
- Spawn child entity: quad mesh sized to match the text bounds, with `GlitchMaterial` applied. The GlitchMaterial overlays scanlines, chromatic split, and block jitter on top of the parent text via additive blending.
- Attach `PunchScale` component on the parent entity: scale 1.0 -> 1.3 -> 1.0 over ~0.15s (the `PunchScale` system was migrated to visuals/ in 5e).
- Attach a `GlitchTextLifetime` component with countdown timer. When timer expires, trigger scan-line dissolve fade (ramp scanline density to maximum + alpha to zero over ~0.1s), then despawn.
- The child overlay's `GlitchUniforms` are set to: moderate scanline_density, moderate scanline_speed, small chromatic_offset (RGB split), medium jitter_intensity. These create the projected-neon-light feel.

### 2. GlitchText Lifecycle System

System running in `Update` that manages active GlitchText entities:

- Decrements `GlitchTextLifetime` timer each frame
- When timer is in the final ~0.1s ("dissolve phase"), ramps `GlitchUniforms.scanline_density` upward and `GlitchUniforms.jitter_intensity` upward — the text visually breaks apart with scan-line distortion before disappearing
- When timer reaches zero, despawns the parent entity and all children

System lives in `breaker-game/src/visuals/systems/glitch_text_lifecycle.rs`.

### 3. Per-Highlight VFX Dispatch

On `HighlightTriggered { kind, position }` message, a dispatch system spawns the appropriate GlitchText + game element VFX:

| Highlight | Text | Position | Game Element VFX |
|-----------|------|----------|-----------------|
| Close Save | "SAVE." | Bottom edge, near shield barrier | `TriggerScreenFlash` with barrier color (white per DR-3), low intensity. Shield barrier entity gets `AddModifier` with `GlowIntensity(3.0)` for 0.2s. |
| Mass Destruction | "OBLITERATE." | Center-screen, over the cell field | Expanding ring entity at center of cell field (reuse shockwave ring mesh pattern from 5l but dimmer, no distortion — visual-only pulse). |
| Combo King | "COMBO." | Near the bolt's current position | Bolt trail intensifies: `AddModifier` on bolt with `TrailLength(2.0)` + `GlowIntensity(2.0)` for 0.3s. `ParticleEmitter` burst near bolt. |
| Pinball Wizard | "RICOCHET." | At the wall the bolt just bounced from | Wall streak: spawn a brief beam entity along the wall surface at impact point, HDR >1.5, fades over 0.2s. `AddModifier` on wall with `GlowIntensity(2.0)` for 0.15s. |
| First Evolution | "EVOLVE." | Center-screen | `TriggerScreenFlash` with evolution-tier glow color (prismatic/gold), medium intensity. `TriggerChromaticAberration` brief pulse. Screen glow shift: brief `AddModifier` on background grid with `GlowIntensity(1.5)` for 0.3s. |
| Nail Biter | "CLUTCH." | Near the timer wall gauge | Timer wall gauge pulses: `AddModifier` on timer entity with `GlowIntensity(2.5)` for 0.3s. `TriggerVignette` brief pulse with timer color. |
| Perfect Streak | "STREAK." | Near the breaker | `ParticleEmitter` burst near breaker, gold color. Breaker glow: `AddModifier` with `GlowIntensity(2.0)` for 0.2s. |
| Fast Clear | "BLITZ." | Center-screen | `TriggerScreenFlash` brief white, low intensity. |
| No Damage Node | "FLAWLESS." | Center-screen | Expanding ring entity, brighter than Mass Destruction variant. |
| Speed Demon | "DEMON." | Along bolt trail | `ParticleEmitter` continuous burst along bolt's recent trajectory (spawn particles at several past positions). |
| Untouchable | "GHOST." | Around breaker | Glow mote particles drifting outward from breaker: `ParticleEmitter` with `Continuous { rate: 10.0 }` for 0.5s, dim motes. |
| Comeback | "SURGE." | Center-screen | `TriggerScreenFlash` with green tint, medium intensity. |
| Perfect Node | "PERFECT." | Center-screen | `TriggerScreenFlash` bright + `TriggerScreenShake { tier: Medium }`. |
| Most Powerful Evolution | "APEX." | Center-screen | `TriggerScreenFlash` bright + `TriggerChromaticAberration` strong + `TriggerScreenShake { tier: Heavy }`. Maximum spectacle for the rarest highlight. |

GlitchText color for each highlight matches its emphasis: gold for performance highlights (PERFECT, STREAK), white for save/barrier highlights, archetype color for bolt/breaker highlights.

Dispatch system lives in `breaker-game/src/run/systems/highlight_vfx.rs`.

### 4. Node-End Highlights

Highlights that trigger on node completion (FLAWLESS, BLITZ, PERFECT, GHOST, STREAK, DEMON) should display on the chip select screen rather than during gameplay — the node has ended, so showing them during chip selection gives the player a moment to appreciate the achievement.

These display sequentially (one at a time, ~0.5s each) at the top of the chip select screen before cards appear. Each uses the same GlitchText + VFX treatment but with slightly longer duration since there is no active gameplay to distract from.

### 5. Remove Existing Text Popups

Remove the current highlight popup system that spawns plain `Text2d` + `PunchScale` + `FadeOut` entities. Search for:
- Highlight-related `Text2d` spawn code in run/ domain
- Any highlight popup components or systems
- References to highlight text content strings in the existing system

Replace all references with the new GlitchText dispatch. Ensure no plain floating text remains for highlights.

## What NOT to Do

- Do NOT implement evolution-specific VFX — those are 5q-5t. The "EVOLVE." highlight here is the First Evolution highlight only.
- Do NOT implement bump grade feedback text — bump grades use pure visual feedback (flash + particles), not text labels. That was built in 5k.
- Do NOT create highlight text as UI nodes — use world-space Text2d entities with GlitchMaterial overlay. These are projected into the game world, not overlaid as UI.
- Do NOT implement new postprocess shaders — use existing triggers from rantzsoft_postprocess (5d)
- Do NOT implement a `SpawnGlitchText` message to a crate — spawn directly in game code using the GlitchMaterial from visuals/

## Dependencies

- **Requires**: 5e (visuals domain — GlitchMaterial, GlitchUniforms, PunchScale system, EntityGlowMaterial for ring/beam entities, modifier messages), 5j (screen effects — screen flash, chromatic aberration, vignette, screen shake lifecycle systems; modifier computation system for game element VFX), 5k (bump/failure VFX — screen flash patterns established there, cell death context detection for Mass Destruction)
- **Enhanced by**: 5f (bolt visuals — bolt trail for Combo King), 5g (breaker visuals — breaker entity for Perfect Streak/Untouchable), 5i (wall visuals — wall entity for Pinball Wizard), 5n (HUD — timer wall gauge for Nail Biter)
- **Independent of**: 5l (combat VFX), 5o (chip cards), 5p (screens)

## Verification

- All highlight types produce GlitchText labels at correct positions with correct text content
- GlitchText shader shows scanlines, chromatic split, and block jitter — text looks like projected neon, not plain text
- Labels punch-scale on appear (1.0 -> 1.3 -> 1.0 over ~0.15s)
- Labels dissolve with scan-line breakup after duration expires, then despawn cleanly
- Labels auto-despawn — no leaked entities accumulate
- Each highlight's game element VFX fires at the correct location and affects the correct game entity
- Close Save: barrier flashes
- Mass Destruction: cell field pulses with expanding ring
- Combo King: bolt trail intensifies
- Pinball Wizard: wall streak at impact point
- First Evolution: screen flash + chromatic aberration + grid glow
- Nail Biter: timer pulses
- APEX (Most Powerful Evolution) produces the most dramatic VFX of all highlights
- Node-end highlights display sequentially on chip select screen, not during gameplay
- No plain floating text remains for any highlight
- Labels are brief enough (~0.3s during gameplay) not to distract from active play
- Multiple highlights in quick succession do not overlap confusingly (queue or offset positions)
- All existing tests pass
- `cargo all-dclippy` clean
- `cargo all-dtest` clean

## Status: NEEDS DETAIL
