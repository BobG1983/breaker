# Feedback & Particles

## Bump Grade Feedback

| Element | Status | Readability | Juice | Style Guide | Current |
|---------|--------|-------------|-------|-------------|---------|
| Perfect bump | PLACEHOLDER | Critical | High | COVERED | Floating "PERFECT" text (HDR cyan, 65px). Style guide: gold/white flash on breaker + VFX burst + sparks + micro-shake. No text. |
| Early bump | PLACEHOLDER | Important | Medium | COVERED | Floating "EARLY" text (orange, 43px). Style guide: dim archetype-color flash + small sparks. No text. |
| Late bump | PLACEHOLDER | Important | Medium | COVERED | Floating "LATE" text (orange, 43px). Style guide: dim flash + minimal sparks. No text. |
| Whiff | PLACEHOLDER | Low | Low | COVERED | Floating "WHIFF" text (gray, 36px). Style guide: nothing. Silence IS feedback. |

### Implementation

**Perfect bump:** Recipe `"bump_perfect"`: `ExpandingRing(speed: 300.0, max_radius: 20.0, thickness: 2.0, hdr: 1.5, color: Gold, lifetime: 0.15) + SparkBurst(count: 12, velocity: 200.0, hdr: 1.2, color: Gold, gravity: 40.0, lifetime: 0.2) + ScreenShake(tier: Micro) + ScreenFlash(color: Gold, intensity: 0.4, duration_frames: 2)`. Fired by `breaker/` on `BumpPerformed { grade: Perfect }` at contact position.

**Early bump:** Recipe `"bump_early"`: `SparkBurst(count: 4, velocity: 120.0, hdr: 0.5, color: White, gravity: 30.0, lifetime: 0.15)`. Subtle.

**Late bump:** Recipe `"bump_late"`: `SparkBurst(count: 2, velocity: 80.0, hdr: 0.3, color: White, gravity: 20.0, lifetime: 0.1)`. Barely visible.

**Whiff:** No recipe. No VFX. No modifier. Nothing fires. Silence IS feedback.

---

## Failure States

| Element | Status | Readability | Juice | Style Guide | Current |
|---------|--------|-------------|-------|-------------|---------|
| Bolt lost | PLACEHOLDER | Critical | High | COVERED | "BOLT LOST" text (white, 86px). Style guide: slow-mo + desaturation + exit streak. No text. |
| Shield absorption | NONE | Critical | High | COVERED | Charge decrements silently. Style guide: barrier flash + cracks + bolt bounces. |
| Life lost | NONE | Critical | High | COVERED | `LivesCount` decrements silently. Style guide: longer slow-mo + danger vignette. |
| Run won | NONE | Important | High | COVERED | State transitions directly. Style guide: freeze-frame + flash + transition. |
| Run over (defeat) | NONE | Important | High | COVERED | State transitions directly. Style guide: extended slow-mo + full desaturation. |
| Time expired | NONE | Important | Medium | COVERED | Timer display shatters into shard particles. Red-orange pulse radiates from timer across screen edges. Dark wave sweeps downward. Desaturates from edges inward. |

### Implementation

**Bolt lost:** Recipe `"bolt_lost"` for the exit streak: `TrailBurst(count: 8, length: 30.0, hdr: 0.8, color: White, fade_distance: 60.0)`. Fired at bolt exit position with direction downward. Plus direct primitives from `bolt/`: `TriggerSlowMotion { factor: 0.3, duration: 0.3, ramp_in: 0.05, ramp_out: 0.1 }` + `TriggerDesaturation { camera, target_factor: 0.7, duration: 0.3 }`.

**Shield absorption:** Not a recipe — shield barrier entity handles it. On charge consumed: effect/ adds a crack seed to the barrier's `ShieldMaterial` uniforms. Plus recipe `"shield_absorb"`: `SparkBurst(count: 6, velocity: 150.0, hdr: 0.8, color: White, gravity: 20.0, lifetime: 0.15)` at barrier position. No slow-mo.

**Life lost:** No recipe. Direct primitives from `effect/`: `TriggerSlowMotion { factor: 0.2, duration: 0.5, ramp_in: 0.05, ramp_out: 0.15 }` + `TriggerVignettePulse { camera, color: OrangeRed, intensity: 0.4, duration: 0.5 }`. Life orb dissolve recipe fires from HUD system (see UI catalog).

**Run won:** No recipe. Direct primitives from `run/`: `TriggerSlowMotion { factor: 0.0, duration: 1.0 }` (freeze-frame) + `TriggerScreenFlash { camera, color: White, intensity: 2.0, duration_frames: 5 }`. Then transition fires.

**Run over:** No recipe. Direct primitives from `run/`: `TriggerSlowMotion { factor: 0.15, duration: 2.0, ramp_in: 0.1, ramp_out: 0.3 }` + `TriggerDesaturation { camera, target_factor: 1.0, duration: 2.0 }`.

**Time expired:** Recipe `"time_expired"`: `ShardBurst(count: 12, velocity: 200.0, rotation_speed: 4.0, hdr: 1.0, color: OrangeRed, lifetime: 0.4) + VignettePulse(color: OrangeRed, intensity: 0.5, duration: 0.5) + Desaturation(target_factor: 0.8, duration: 1.0)`. Fired at timer wall position.

---

## Screen Effects

| Element | Status | Readability | Juice | Style Guide | Current |
|---------|--------|-------------|-------|-------------|---------|
| Screen shake system | NONE | Important | High | COVERED | Not implemented. Four tiers: Micro/Small/Medium/Heavy with directional decay. |
| Chromatic aberration pulse | NONE | Low | High | COVERED | Not implemented. Brief RGB offset on events. |
| Screen distortion (radial) | NONE | Important | High | COVERED | Not implemented. For shockwave, gravity well, explosion. |
| Screen flash | NONE | Low | Medium-High | COVERED | Full-screen additive HDR spike (1-3 frames). Gold/white for skill, white for triumphs, red-orange for danger, temperature-tinted for escalation. Intensity tiers match shake tiers. |
| Desaturation | NONE | Important | High | COVERED | Not implemented. For bolt lost, run defeat. |
| Slow-motion / time dilation | NONE | Critical | High | COVERED | Not implemented. Game clock slowdown for failure states. |
| HDR Bloom (tunable) | PARTIAL | Important | High | COVERED | Camera has `Bloom::default()`. No per-entity control, no intensity tuning. |
| CRT/scanline overlay | NONE | Low | Low-Med | COVERED | Not implemented. Off by default, configurable. |
| Danger vignette | NONE | Medium | Medium | COVERED | Red-orange gradient inward from all edges (~15% screen width). Pulses at danger-scaled rhythm (slow→accelerating→heartbeat-synced). Timer <25%: 10-40% opacity. Last life: persistent 15%. Never >50% combined. |

### Implementation

All screen effects are **direct primitive messages** — they don't use recipes. They are FullscreenMaterial components on the camera entity with trigger/lifecycle systems.

| Effect | Message | Step |
|--------|---------|------|
| Screen shake | `TriggerScreenShake { camera, tier, direction }` | 5k |
| Chromatic aberration | `TriggerChromaticAberration { camera, intensity, duration }` | 5k |
| Radial distortion | `TriggerRadialDistortion { camera, origin, intensity, duration }` | 5k |
| Screen flash | `TriggerScreenFlash { camera, color, intensity, duration_frames }` | 5k |
| Desaturation | `TriggerDesaturation { camera, target_factor, duration }` | 5k |
| Slow motion | `TriggerSlowMotion { factor, duration, ramp_in, ramp_out }` | 5k |
| Vignette pulse | `TriggerVignettePulse { camera, color, intensity, duration }` | 5k |

**Bloom:** Configured via `VfxConfig.bloom_intensity`. Per-entity bloom via HDR material values >1.0. Step 5d.

**CRT:** `crt.wgsl` FullscreenMaterial. Toggled via `VfxConfig.crt_enabled`. Step 5d.

**Danger vignette:** Continuous system in `run/danger_vignette.rs`. Reads timer and lives, sends `TriggerVignettePulse` with varying intensity. Step 5k.

---

## Highlight Moment Popups

| Element | Status | Readability | Juice | Style Guide | Current |
|---------|--------|-------------|-------|-------------|---------|
| Highlight system (shared) | PLACEHOLDER | Important | Medium | COVERED | Floating text + PunchScale + FadeOut. Style guide: stylized glitch text labels. |
| "SAVE." (Close Save) | PLACEHOLDER | Low | Medium | COVERED | "CLOSE SAVE!" text cyan. Style guide: glitch text at bottom edge. |
| "OBLITERATE." (Mass Destruction) | PLACEHOLDER | Low | High | COVERED | "MASS DESTRUCTION!" text orange. Style guide: glitch text center-screen. |
| "COMBO." (Combo King) | PLACEHOLDER | Low | Medium | COVERED | "COMBO KING!" text orange. Style guide: glitch text near bolt. |
| "RICOCHET." (Pinball Wizard) | PLACEHOLDER | Low | Medium | COVERED | "PINBALL WIZARD!" text orange. Style guide: glitch text at wall. |
| "EVOLVE." (First Evolution) | PLACEHOLDER | Low | High | COVERED | "FIRST EVOLUTION!" text yellow. Style guide: glitch text + evo-tier glow. |
| "CLUTCH." (Nail Biter) | PLACEHOLDER | Low | Medium | COVERED | "NAIL BITER!" text green. Style guide: glitch text near timer. |
| Other highlights (9 more) | PLACEHOLDER | Low | Medium | COVERED | PerfectStreak, FastClear, NoDamageNode, CloseSave, SpeedDemon, Untouchable, Comeback, PerfectNode, MostPowerfulEvolution — all text-only. |

### Implementation

Each highlight is a recipe containing a `GlitchText` step + per-highlight screen effect steps. Fired by `run/` on `HighlightTriggered { kind }`.

| Highlight | Recipe Name | GlitchText | Extra Steps |
|-----------|-------------|------------|-------------|
| Close Save | `"highlight_save"` | `GlitchText(text: "SAVE.", size: 48.0, color: White, duration: 0.8)` | `ScreenFlash(color: White, intensity: 0.3, duration_frames: 2)` |
| Mass Destruction | `"highlight_obliterate"` | `GlitchText(text: "OBLITERATE.", size: 56.0, color: OrangeRed, duration: 1.0)` | `ExpandingRing(speed: 300.0, max_radius: 60.0, thickness: 1.0, hdr: 0.5, color: OrangeRed, lifetime: 0.3)` |
| Combo King | `"highlight_combo"` | `GlitchText(text: "COMBO.", size: 48.0, color: Gold, duration: 0.8)` | `SparkBurst(count: 8, velocity: 150.0, hdr: 0.6, color: Gold, gravity: 30.0, lifetime: 0.2)` |
| Pinball Wizard | `"highlight_ricochet"` | `GlitchText(text: "RICOCHET.", size: 48.0, color: White, duration: 0.8)` | `SparkBurst(count: 6, velocity: 120.0, hdr: 0.5, color: White, gravity: 20.0, lifetime: 0.15)` |
| First Evolution | `"highlight_evolve"` | `GlitchText(text: "EVOLVE.", size: 56.0, color: Gold, duration: 1.0)` | `ScreenFlash(color: Gold, intensity: 0.5, duration_frames: 3) + ChromaticAberration(intensity: 0.2, duration: 0.3)` |
| Nail Biter | `"highlight_clutch"` | `GlitchText(text: "CLUTCH.", size: 48.0, color: LimeGreen, duration: 0.8)` | `VignettePulse(color: LimeGreen, intensity: 0.2, duration: 0.3)` |
| Perfect Streak | `"highlight_streak"` | `GlitchText(text: "STREAK.", size: 48.0, color: Gold, duration: 0.8)` | `SparkBurst(count: 8, ..., color: Gold)` |
| Fast Clear | `"highlight_blitz"` | `GlitchText(text: "BLITZ.", size: 48.0, color: White, duration: 0.8)` | `ScreenFlash(color: White, intensity: 0.3, duration_frames: 2)` |
| No Damage Node | `"highlight_flawless"` | `GlitchText(text: "FLAWLESS.", size: 56.0, color: White, duration: 1.0)` | `ExpandingRing(speed: 200.0, max_radius: 80.0, thickness: 1.5, hdr: 0.6, color: White, lifetime: 0.4)` |
| Speed Demon | `"highlight_demon"` | `GlitchText(text: "DEMON.", size: 48.0, color: OrangeRed, duration: 0.8)` | `SparkBurst(count: 10, ..., color: OrangeRed)` |
| Untouchable | `"highlight_ghost"` | `GlitchText(text: "GHOST.", size: 48.0, color: MediumPurple, duration: 0.8)` | `GlowMotes(count: 8, drift_speed: 20.0, radius: 30.0, hdr: 0.4, color: MediumPurple, lifetime: 0.5)` |
| Comeback | `"highlight_surge"` | `GlitchText(text: "SURGE.", size: 48.0, color: LimeGreen, duration: 0.8)` | `ScreenFlash(color: LimeGreen, intensity: 0.3, duration_frames: 2)` |
| Perfect Node | `"highlight_perfect"` | `GlitchText(text: "PERFECT.", size: 64.0, color: Gold, duration: 1.2)` | `ScreenFlash(color: Gold, intensity: 0.6, duration_frames: 3) + ScreenShake(tier: Medium)` |
| Most Powerful Evo | `"highlight_apex"` | `GlitchText(text: "APEX.", size: 64.0, color: Gold, duration: 1.2)` | `ScreenFlash(color: Gold, intensity: 0.8, duration_frames: 4) + ChromaticAberration(intensity: 0.3, duration: 0.4) + ScreenShake(tier: Heavy)` |

---

## Particle Types

| Element | Status | Juice | Style Guide | Usage |
|---------|--------|-------|-------------|-------|
| Spark particles | NONE | High | COVERED | Cell destruction, impact effects, bump feedback |
| Trail particles | NONE | High | COVERED | Bolt wake, dash trail, beam afterimage |
| Shard particles | NONE | High | COVERED | Cell shatter (combo/chain), shield break |
| Glow mote particles | NONE | Low-Med | COVERED | Background sprites, gravity well ambient |
| Energy ring particles | NONE | High | COVERED | Shockwave, pulse, bump feedback rings |
| Electric arc particles | NONE | High | COVERED | Chain lightning, electric effects |

### Implementation

Particles are not recipes — they are the primitives that recipes use. Each is a `PrimitiveStep` variant / direct message type implemented in step 5e. See `docs/architecture/rendering/particles.md` for the emitter architecture and `docs/architecture/rendering/recipes.md` for how recipes reference them.
