# Game Design Guard Memory

## Phase 1 Validation (2026-03-11, updated)
- Phase 1 core mechanics APPROVED against design pillars
- See `phase1-review.md` for detailed findings
- Bump grade detection NOW USES BoltHitBreaker messages (fixed since last review)
- Perfect bump dash-cancel IS IMPLEMENTED -- schedule ordering VERIFIED CORRECT

## Open Issues
- apply_bump_velocity lacks explicit .after() ordering in bolt/plugin.rs -- works by accident, should fix
- Bolt-lost respawn has no weight/penalty pre-Phase 2 -- deferred, acceptable
- No bolt speed decay mechanism -- track for Phase 2 discussion
- MIN_PHYSICS_FPS (30.0) hardcoded in bolt_cell_collision -- needs comment about max_speed dependency

## Key Parameter Values (Phase 1)
- Breaker: max_speed=500, dash_mult=2.0x, dash_dur=0.15s
- Bolt: base=400, min=200, max=800, min_angle=~10deg
- Bump: duration=0.3s, perfect_window=0.05s, early_window=0.15s
- Bump multipliers: perfect=1.5x, early/late=0.8x (penalty!), none=1.0x
- Tilt: dash=~15deg, brake=~25deg
- Max reflection: ~75deg from vertical
- Bolt speed floored to base_speed (400) on every breaker contact

## Design Principles Confirmed
- Mistimed bumps (0.8x) are WORSE than no bump (1.0x) -- mashing is punished
- Bolt reflection completely overwrites direction -- every breaker contact is skill expression
- Dash requires directional commitment -- no stationary dashes
- Three control axes: position, tilt angle, bump timing
- All gameplay parameters are data-driven (RON configs + Rust defaults)

## Phase 2b Validation (2026-03-13, updated)
- Run structure and node timer APPROVED
- See `phase2b-review.md` for detailed findings
- Timer UI thresholds at 33%/15% -- IMPLEMENTED, correct
- Fortress retuned to 70s -- escalation now correct (HP/s: 0.57 -> 0.72 -> 1.0)
- Run-end screen still has dead air (no auto-advance) and weak copy ("The clock ran out." / "All nodes cleared!")

## Data-Driven Config Status
- bolt: RON + BoltDefaults + BoltConfig -- COMPLETE
- breaker: RON + BreakerDefaults + BreakerConfig -- COMPLETE
- cells: RON + CellDefaults + CellConfig -- COMPLETE
- physics: RON + PhysicsDefaults + PhysicsConfig -- COMPLETE
- playfield: RON + PlayfieldDefaults + PlayfieldConfig -- COMPLETE
- mainmenu: RON + MainMenuDefaults + MainMenuConfig -- COMPLETE
- timerui: RON + TimerUiDefaults + TimerUiConfig -- COMPLETE

## Future Design Notes
- Speed decay: recommend per-bounce/per-cell-hit decay, NOT passive time decay
- Bolt-lost respawn: consider random angle on respawn to force reaction
- Piercing Amps will need "one cell per tick" limit revisited
- Phase 4: timer urgency should escalate to screen-level effects, not just text color
- Phase 7: run rewards should differentiate on time remaining and nodes cleared
