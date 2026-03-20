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

## Open Issues (Ordered by Priority)
1. **BLOCKING** Test code uses 0.8x weak multiplier (should be 1.1x) — bump.rs, apply_bump_velocity.rs, init_breaker_params.rs
2. **BLOCKING** Prism archetype bolt-lost penalty too soft (3s TimePenalty) — needs LoseExtraBolts or higher penalty
3. **IMPORTANT** Run-end screen dead air (no timer/auto-advance) — still unfixed from 2 prior reviews
4. **IMPORTANT** Run-end subtitle copy weak/passive — needs motivating tone
5. **IMPORTANT** Chip select timer 10s too generous — recommend 8s
6. **IMPORTANT** All 3 layouts in Passive pool — no Active or Boss pool layouts
7. **IMPORTANT** Passive vs Active node types not behaviorally differentiated — timer ticks on all nodes
8. **MINOR** RON type annotation mismatch: defaults.chipselect.ron says upgrade_select, should be chip_select
9. **MINOR** 150ms perfect window may be too generous post-rescale — validate Phase 4
10. **MINOR** introduced_cells field in difficulty tiers is empty (content gap)

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
- Prism should get LoseExtraBolts consequence type (Phase 4d) — the sharp, exciting fix
- Chip authoring (4c.2): prioritize synergy pairs, don't ship 16 independent stat buffs
- Surge overclock needs visible juice when implemented — shockwave is the poster child for overclock feel
- Run-end screen: consider randomized subtitle pools for variety + forward-looking tone
