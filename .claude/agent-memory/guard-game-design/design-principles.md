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

## Open Issues (Ordered by Priority)
1. Run-end screen dead air (no timer/auto-advance) — HIGH
2. ~~PLAN.md/README say bump "all grades boost" but 0.8x is penalty~~ FIXED — code updated to 1.1x
3. ~~Bolt-lost respawn straight up = no reaction required~~ FIXED — randomized within ±30°
4. 150ms perfect window may be too generous post-rescale — MEDIUM (validate Phase 4)
5. Run-end subtitle copy is weak/passive — LOW
6. Main menu skips RunSetup state — LOW (expected pre-2d)

## Future Design Notes
- Speed decay: recommend per-bounce/per-cell-hit decay, NOT passive time decay
- Bolt-lost respawn: randomize angle within +/-30deg to force reaction
- Piercing Amps will need "one cell per tick" limit revisited
- Phase 4: timer urgency should escalate to screen-level effects, not just text color
- Phase 7: introduce optional cells (not required_to_clear) for risk/reward with timer
- Phase 7: run rewards should differentiate on time remaining and nodes cleared
- If Phase 4 feels too easy: first knobs are perfect_window (toward 80-100ms) and dash_mult (toward 2.5-3x)
