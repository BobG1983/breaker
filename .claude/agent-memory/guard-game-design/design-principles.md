---
name: Design Principles & Open Issues
description: Confirmed design principles, open issues, and future design notes
type: reference
---

## Design Principles Confirmed
- Mistimed bumps (1.1x) give small boost — all attempted bumps are rewarded
- Bolt reflection completely overwrites direction — every contact is skill expression
- Dash requires directional commitment — no stationary dashes
- Three control axes: position, tilt angle, bump timing
- All gameplay parameters are data-driven (RON configs + Rust defaults)
- Timer has no grace period — zero means you lose
- Archetype behaviors are data-driven via RON trigger/consequence bindings
- Chip stacking is flat per-stack with per-chip caps — Isaac-style pool depletion
- Chip select timeout = skip (no consolation prize) — maximum pressure
- Every archetype MUST have a bolt-lost consequence — no free respawns
- Regen rate must NOT scale with hp_mult (avoids late-game stalemates)
- ExtraBolt despawns on loss, never respawns — correct Prism behavior
- Shockwave damage MUST scale with DamageBoost — no "flat area damage" exceptions; synergy web requires it (IMPLEMENTED: shockwave.rs reads DamageBoost from bolt entity)
- Perfect bump requirement for Surge overclock is correct — do not weaken
- Global triggers (OnCellDestroyed, OnBoltLost) must not silently no-op when used with position-dependent effects (IMPLEMENTED: Option<Entity> on EffectFired.bolt, shockwave returns early on None)
- ChipKind removed — chip category is derived from ChipEffect variant, not a parallel enum. Enables multi-effect chips cleanly.
- TriggerChain leaf stacking formula: base + (stacks-1) * per_level — uniform across Shockwave/MultiBolt/Shield
- Surge shockwave range_per_level: 32.0 is well-calibrated (each stack adds ~0.6 cell widths of radius)
- Unified TriggerChain evaluation engine — archetypes and overclocks share the same grammar (2026-03-21)
- OnBumpSuccess should be reserved for defensive effects (Shield) — offensive power demands OnPerfectBump
- Bump-grade triggers (EarlyBump, LateBump, BumpWhiff) transform bumping from binary to spectrum — use aggressively in archetype/chip design
- Archetype root fields (on_bolt_lost, on_perfect_bump, etc.) get auto-wrapped into ActiveChains at init — no separate dispatch path
- SpeedBoost is a TriggerChain leaf (not special-cased) — fires through EffectFired like all other effects (2026-03-21)
- Bump SpeedBoost targets SPECIFIC bolt (SpeedBoostTarget::Bolt) not all bolts — per-bolt targeting rewards skill in multi-bolt (Prism) play
- BumpForceBoost (Augment) is conceptually DISTINCT from TriggerChain SpeedBoost: BumpForce = flat additive impulse at reflection, SpeedBoost = multiplicative scaling via triggered events
- Prism archetype has NO bump speed boost (intentional) — Prism trades velocity for quantity (SpawnBolt on perfect bump)
- Offering weights (Common=100, Uncommon=50, Rare=15, Legendary=3) produce 33:17:5:1 ratio — well-calibrated for 12-chip pool (2026-03-22)
- Pool depletion accelerates build-crafting: maxing chips concentrates future offerings toward remaining pieces (Pillar 2 alignment confirmed)
- Weight decay 0.8x per rejection is correct rate — creates "fresh options" without removing anything permanently
- Decay must ONLY apply at selection/timeout, NEVER at generation time — generation reads decay, does not write it
- Selected chip must NOT be decayed — player is actively building around it
- Timeout must decay all offered chips (player saw them and chose none)
- Transition timing 0.5s out + 0.3s in (0.8s total) is at fast end of spec — correct instinct for this game

## Open Issues (Ordered by Priority)
1. ~~**BLOCKING** Test code uses 0.8x weak multiplier (should be 1.1x)~~ RESOLVED — apply_bump_velocity.rs removed by unify-behaviors refactor; multipliers now expressed as TriggerChain SpeedBoost leaves in archetype RON (1.1x early/late, 1.5x perfect) (2026-03-21)
2. **BLOCKING** Prism archetype bolt-lost penalty too soft (7s TimePenalty, was 3s) — needs LoseExtraBolts leaf variant or higher penalty
2b. **BLOCKING** BumpForceBoost (Augment chip) is dead code — component gets stamped on Breaker but never read by any system. Was never wired up (pre-dates SpeedBoost refactor). Needs: flat additive speed bonus in reflect_top_hit (bolt_breaker_collision.rs), reading BumpForceBoost from breaker entity
2c. **BLOCKING** Double decay bug in chip offerings — generate_chip_offerings applies decay at generation AND handle_chip_input applies at confirmation. Selected chip also penalized. Fix: remove generation-time decay, add timeout decay to tick_chip_timer (2026-03-22)
3. **IMPORTANT** Run-end screen dead air (no timer/auto-advance) — unfixed from 4 prior reviews (2026-03-23: flagged again in Wave 4 review). Recommend 10s auto-advance timer, timeout = New Run.
4. ~~**IMPORTANT** Run-end subtitle copy weak/passive — needs motivating tone~~ RESOLVED — 5 variants per outcome, seed-deterministic, good tone (2026-03-23)
5. ~~**IMPORTANT** Chip select timer 10s too generous — recommend 8s~~ RESOLVED — RON now at 8.0s (2026-03-22)
6. **IMPORTANT** All 3 layouts in Passive pool — no Active or Boss pool layouts
7. **IMPORTANT** Passive vs Active node types not behaviorally differentiated — timer ticks on all nodes
8. **IMPORTANT** TransitionOut hardcoded to ChipSelect — needs TransitionTarget mechanism for 4h (evolution) branching (2026-03-22)
9. **IMPORTANT** RON config missing new offering/rarity weight fields — falls back to code defaults, defeats data-driven tuning (2026-03-22)
10. **MINOR** 150ms perfect window may be too generous post-rescale — validate Phase 4
12. **MINOR** introduced_cells field in difficulty tiers is empty (content gap)
13. **MINOR** Flash transition color white is off-brand for neon cyberpunk — consider hot neon color (Phase 5)
14. **MINOR** Sweep direction always left-to-right — consider seeded direction randomization
15. **BLOCKING** Evolution reward screen spec says "no timer" — violates Pillar 1. Must have timer (6s, timeout = skip). (2026-03-23)
16. **IMPORTANT** Evolution "no eligible" fallback undefined — recommend curated boss chip pool w/ higher rarity weights (2026-03-23)
17. **IMPORTANT** Evolved chip effects need design direction: must introduce new interaction points, not just stat boosts (2026-03-23)
18. **BLOCKING** spawn_highlight_text system is dead code — never registered in RunPlugin::build. Zero in-game juice from highlights (2026-03-23)
19. **BLOCKING** FadeOut { timer: 0.0 } in spawn_highlight_text — text despawns immediately (timer should equal duration: 2.0) (2026-03-23)
20. **BLOCKING** track_node_cleared_stats never emits HighlightTriggered — 8 of 15 highlight types have no in-game juice message (2026-03-23)
21. **IMPORTANT** All highlight popups spawn at same position (0,100,10) — simultaneous highlights stack unreadably (2026-03-23)
22. **IMPORTANT** track_node_cleared_stats has no per-kind dedup — same HighlightKind can fill entire cap (2026-03-23)

## Resolved (from prior reviews)
- ~~PLAN.md/README say bump "all grades boost" but 0.8x is penalty~~ FIXED in RON — but test code still uses 0.8x (issue #1)
- ~~Bolt-lost respawn straight up = no reaction required~~ FIXED — randomized within +/-30deg
- ~~Shockwave used flat BASE_BOLT_DAMAGE~~ FIXED — now routes through DamageCell with DamageBoost scaling (2026-03-20)
- ~~ChipKind redundant discriminator~~ REMOVED — ChipEffect enum is the sole category source (2026-03-20)
- ~~Entity::PLACEHOLDER in EffectFired.bolt (was OverclockEffectFired.bolt)~~ FIXED — replaced with Option<Entity> for proper null semantics (2026-03-20); event renamed EffectFired in refactor/unify-behaviors (2026-03-21)
- ~~RON type annotation mismatch: defaults.chipselect.ron says upgrade_select, should be chip_select~~ FIXED — verified 2026-03-22, file already says chip_select in type annotation

## Future Design Notes
- Speed decay: recommend per-bounce/per-cell-hit decay, NOT passive time decay
- Piercing Amps will need "one cell per tick" limit revisited
- Phase 4: timer urgency should escalate to screen-level effects, not just text color
- Phase 7: introduce optional cells (not required_to_clear) for risk/reward with timer
- Phase 7: run rewards should differentiate on time remaining and nodes cleared
- If Phase 4 feels too easy: first knobs are perfect_window (toward 80-100ms) and dash_mult (toward 2.5-3x)
- Prism should get LoseExtraBolts consequence type (Phase 4d) — the sharp, exciting fix
- Chip authoring (4c.2): prioritize synergy pairs, don't ship 16 independent stat buffs
- Surge overclock needs visible juice when implemented — shockwave is the poster child for overclock feel
- Run-end screen: consider randomized subtitle pools for variety + forward-looking tone
- Shockwave range 96.0 (not 64.0) — 64 only hits vertical neighbors; 96 catches near-diagonal, rewards positioning
- Piercing + armed trigger interaction needs explicit design intent: fire once or at each impact point?
- ChainHit + trigger evaluation: do chain hits resolve armed triggers? Phase 7+ decision but architecture must not prevent it
- Phase 7: consider OnCellDestroyed(Shockwave) using destroyed cell position for chain reaction mechanic
- Global trigger arming limitation: Arm results discarded for CellDestroyed/BoltLost/BumpWhiff (no bolt entity). Phase 7: decide if global-to-armed chains should target all bolts or require rethinking
- Risk/reward archetype design now possible: OnBumpWhiff penalties + higher OnPerfectBump rewards = sharp skill expression
- Archetype-overclock resonance: design overclocks that double up on archetype trigger kinds for build affinities (Phase 7+)
- SpeedBoost as TriggerChain leaf enables new chip archetypes: OnCellDestroyed(SpeedBoost) for velocity ramp, OnBumpWhiff(SpeedBoost(0.8)) for whiff penalties, etc.
- Amp SpeedBoost (flat, raises base/max) + TriggerChain SpeedBoost (multiplier) = multiplicative synergy — classic build-game power curve
- Prism SpeedBoost chip concept: "Photon Accelerator: OnPerfectBump(SpeedBoost(target: AllBolts, multiplier: 1.2))" — opt-in, not archetype baseline
- Evolution reward screen MUST have timer (6s recommended) — "no timer" violates Pillar 1 (2026-03-23)
- Evolution "no eligible" fallback: curated boss chip pool with higher rarity weights, same timer pressure
- Evolved chips must introduce new interaction points (new trigger conditions, cross-kind synergies) — not just stronger stat versions (2026-03-23)
- ~~Highlight moments (ClutchClear, PerfectStreak, etc.) should trigger in-game juice when detected, not just post-run display (2026-03-23)~~ IMPLEMENTED (spawn_highlight_text exists but has 3 BLOCKING bugs, see issues #18-20)
- ~~Run-end subtitles should be distinct per death type and ideally randomized from a pool (2026-03-23)~~ IMPLEMENTED — 5 variants per outcome, seed-deterministic selection
- Highlight popup system needs screen shake + particles in Phase 5+ (text-only is placeholder)
- Future: Flux bonus per highlight, or chip that scales with highlights earned (Phase 8)
- Flux formula too soft for final balance (bolts_lost * -3 is negligible) — flag for Phase 8 tuning, fine for vertical slice
