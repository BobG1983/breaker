# Game Design Guard Memory

## Review Status
- Full codebase review (2026-03-16): Phase 1 APPROVED, Phase 2b APPROVED, Phase 2c APPROVED
- Parameters rescaled to 1920x1080 canvas (all values differ from Phase 1 review)
- 2 HIGH issues fixed (run-end dead air, bump multipliers 0.8x->1.1x)
- 2 MEDIUM issues tracked (bolt-lost respawn, perfect window generosity)

## Key Parameter Values (Post-Rescale, 2026-03-16)
- Playfield: 1440w x 1080h
- Breaker: width=216, height=36, max_speed=900, dash_mult=4.0x, dash_dur=0.15s
- Bolt: base=720, min=360, max=1440, radius=14
- Bump: perfect_window=0.15s, early=0.15s, late=0.15s
- Bump multipliers (Aegis): perfect=1.5x, early/late=1.1x
- Bump cooldowns: perfect=0.0, weak=0.15s
- Tilt: dash=15deg, brake=25deg
- Max reflection: 75deg from vertical
- Dash covers 540 units = 37.5% of playfield width

## Design Principles Confirmed
- Mistimed bumps (1.1x) give small boost — all attempted bumps are rewarded
- Bolt reflection completely overwrites direction -- every contact is skill expression
- Dash requires directional commitment -- no stationary dashes
- Three control axes: position, tilt angle, bump timing
- All gameplay parameters are data-driven (RON configs + Rust defaults)
- Timer has no grace period -- zero means you lose
- Archetype behaviors are data-driven via RON trigger/consequence bindings

## Open Issues (Ordered by Priority)
1. Run-end screen dead air (no timer/auto-advance) -- HIGH
2. ~~PLAN.md/README say bump "all grades boost" but 0.8x is penalty~~ FIXED — code updated to 1.1x
3. ~~Bolt-lost respawn straight up = no reaction required~~ FIXED — randomized within ±30° via GameRng + BoltRespawnAngleSpread
4. 150ms perfect window may be too generous post-rescale -- MEDIUM (validate Phase 4)
5. Run-end subtitle copy is weak/passive -- LOW
6. Main menu skips RunSetup state -- LOW (expected pre-2d)

## Data-Driven Config Status
- bolt: RON + BoltDefaults + BoltConfig -- COMPLETE
- breaker: RON + BreakerDefaults + BreakerConfig -- COMPLETE
- cells: RON + CellDefaults + CellConfig -- COMPLETE
- physics: RON + PhysicsDefaults + PhysicsConfig -- COMPLETE
- playfield: RON + PlayfieldDefaults + PlayfieldConfig -- COMPLETE
- mainmenu: RON + MainMenuDefaults + MainMenuConfig -- COMPLETE
- timerui: RON + TimerUiDefaults + TimerUiConfig -- COMPLETE
- archetype: RON + ArchetypeDefinition + ArchetypeRegistry -- COMPLETE

## Future Design Notes
- Speed decay: recommend per-bounce/per-cell-hit decay, NOT passive time decay
- Bolt-lost respawn: randomize angle within +/-30deg to force reaction
- Piercing Amps will need "one cell per tick" limit revisited
- Phase 4: timer urgency should escalate to screen-level effects, not just text color
- Phase 7: introduce optional cells (not required_to_clear) for risk/reward with timer
- Phase 7: run rewards should differentiate on time remaining and nodes cleared
- If Phase 4 feels too easy: first knobs are perfect_window (toward 80-100ms) and dash_mult (toward 2.5-3x)

## Session History
See [ephemeral/](ephemeral/) — not committed.
