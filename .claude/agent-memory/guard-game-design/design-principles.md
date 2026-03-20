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

## Open Issues (Ordered by Priority)
1. **HIGH** Test code in behaviors/ uses 0.8x early/late multiplier (should be 1.1x) — active.rs, bridges.rs, definition.rs
2. **HIGH** Prism archetype has no bolt-lost consequence — zero-penalty failure state
3. **HIGH** Run-end screen dead air (no timer/auto-advance) — still unfixed from prior review
4. **MEDIUM** Chip select timer 10s may be too generous — recommend 8s
5. **MEDIUM** Regen cells 2.0 HP/s may create stalemates — regen rate should NOT scale with hp_mult
6. **MEDIUM** Passive vs Active node types not behaviorally differentiated
7. **MEDIUM** All 3 layouts in Passive pool — no Active or Boss pool layouts yet
8. **MEDIUM** 150ms perfect window may be too generous post-rescale — validate Phase 4
9. **LOW** Run-end subtitle copy is weak/passive
10. **LOW** introduced_cells field in difficulty tiers is empty (content gap)

## Resolved (from prior reviews)
- ~~PLAN.md/README say bump "all grades boost" but 0.8x is penalty~~ FIXED in RON — but test code still uses 0.8x (issue #1)
- ~~Bolt-lost respawn straight up = no reaction required~~ FIXED — randomized within +/-30deg

## Future Design Notes
- Speed decay: recommend per-bounce/per-cell-hit decay, NOT passive time decay
- Piercing Amps will need "one cell per tick" limit revisited
- Phase 4: timer urgency should escalate to screen-level effects, not just text color
- Phase 7: introduce optional cells (not required_to_clear) for risk/reward with timer
- Phase 7: run rewards should differentiate on time remaining and nodes cleared
- If Phase 4 feels too easy: first knobs are perfect_window (toward 80-100ms) and dash_mult (toward 2.5-3x)
- Prism should eventually get LoseExtraBolts consequence type (Phase 4d)
- Chip authoring (4c.2): prioritize synergy pairs, don't ship 16 independent stat buffs
- Surge overclock needs visible juice when implemented — shockwave is the poster child for overclock feel
