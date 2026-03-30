# Feedback & Particles

## Bump Grade Feedback

| Element | Status | Readability | Juice | Style Guide | Current |
|---------|--------|-------------|-------|-------------|---------|
| Perfect bump | PLACEHOLDER | Critical | High | COVERED | Floating "PERFECT" text (HDR cyan, 65px). Style guide: gold/white flash on breaker + VFX burst + sparks + micro-shake. No text. |
| Early bump | PLACEHOLDER | Important | Medium | COVERED | Floating "EARLY" text (orange, 43px). Style guide: dim archetype-color flash + small sparks. No text. |
| Late bump | PLACEHOLDER | Important | Medium | COVERED | Floating "LATE" text (orange, 43px). Style guide: dim flash + minimal sparks. No text. |
| Whiff | PLACEHOLDER | Low | Low | COVERED | Floating "WHIFF" text (gray, 36px). Style guide: nothing. Silence IS feedback. |

## Failure States

| Element | Status | Readability | Juice | Style Guide | Current |
|---------|--------|-------------|-------|-------------|---------|
| Bolt lost | PLACEHOLDER | Critical | High | COVERED | "BOLT LOST" text (white, 86px). Style guide: slow-mo + desaturation + exit streak. No text. |
| Shield absorption | NONE | Critical | High | COVERED | Charge decrements silently. Style guide: barrier flash + cracks + bolt bounces. |
| Life lost | NONE | Critical | High | COVERED | `LivesCount` decrements silently. Style guide: longer slow-mo + danger vignette. |
| Run won | NONE | Important | High | COVERED | State transitions directly. Style guide: freeze-frame + flash + transition. |
| Run over (defeat) | NONE | Important | High | COVERED | State transitions directly. Style guide: extended slow-mo + full desaturation. |
| Time expired | NONE | Important | Medium | COVERED | Timer display shatters into shard particles. Red-orange pulse radiates from timer across screen edges. Dark wave sweeps downward. Desaturates from edges inward. |

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
| CRT/scanline overlay | NONE | Low | Low-Med | DECISION REQUIRED (DR-7) | Not implemented. Default on/off TBD. |
| Danger vignette | NONE | Medium | Medium | COVERED | Red-orange gradient inward from all edges (~15% screen width). Pulses at danger-scaled rhythm (slow→accelerating→heartbeat-synced). Timer <25%: 10-40% opacity. Last life: persistent 15%. Never >50% combined. |

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

## Particle Types

| Element | Status | Juice | Style Guide | Usage |
|---------|--------|-------|-------------|-------|
| Spark particles | NONE | High | COVERED | Cell destruction, impact effects, bump feedback |
| Trail particles | NONE | High | COVERED | Bolt wake, dash trail, beam afterimage |
| Shard particles | NONE | High | COVERED | Cell shatter (combo/chain), shield break |
| Glow mote particles | NONE | Low-Med | COVERED | Background sprites, gravity well ambient |
| Energy ring particles | NONE | High | COVERED | Shockwave, pulse, bump feedback rings |
| Electric arc particles | NONE | High | COVERED | Chain lightning, electric effects |
