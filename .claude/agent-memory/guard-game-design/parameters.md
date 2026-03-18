---
name: Parameters & Config Status
description: Current parameter values, review status, and data-driven config status per domain
type: reference
---

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

## Data-Driven Config Status
- bolt: RON + BoltDefaults + BoltConfig — COMPLETE
- breaker: RON + BreakerDefaults + BreakerConfig — COMPLETE
- cells: RON + CellDefaults + CellConfig — COMPLETE
- physics: RON + PhysicsDefaults + PhysicsConfig — COMPLETE
- playfield: RON + PlayfieldDefaults + PlayfieldConfig — COMPLETE
- mainmenu: RON + MainMenuDefaults + MainMenuConfig — COMPLETE
- timerui: RON + TimerUiDefaults + TimerUiConfig — COMPLETE
- archetype: RON + ArchetypeDefinition + ArchetypeRegistry — COMPLETE
