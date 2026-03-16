---
name: Full Codebase Design Review 2026-03-16
description: Comprehensive review of all implemented mechanics through Phase 2c against design pillars
type: project
---

## Full Codebase Review (2026-03-16)

### Scope
Phase 1 (core mechanics) + Phase 2a (level loading) + Phase 2b (run/timer) + Phase 2c (archetype/Aegis)

### Verdict: STRONG FOUNDATION, several issues

### Issues Found

1. **HIGH -- Run-end screen dead air**: No timer, no auto-advance, static "Press Enter to continue." Violates Speed and Tension pillars. Location: screen/run_end/systems/spawn_run_end_screen.rs

2. **HIGH -- Doc inconsistency on bump multipliers**: PLAN.md and Phase 2 README say "all grades boost" but Aegis early/late is 0.8x (penalty). Implementation is CORRECT, docs are WRONG.

3. **MEDIUM -- Bolt-lost respawn too safe**: Straight up at base speed, zero reaction required. Should randomize respawn angle. Location: physics/systems/bolt_lost.rs

4. **MEDIUM -- Perfect window at 150ms may be too generous**: Was 50ms pre-rescale, now 150ms (9 frames). Combined with 4x dash mult covering 37.5% of playfield, game may be too easy. Validate in Phase 4.

5. **LOW -- Run-end subtitle text weak**: "All nodes cleared!" / "The clock ran out." -- both passive/obvious.

6. **LOW -- Main menu skips RunSetup**: Goes directly to Playing. Expected pre-2d but must not persist.

### Parameter Snapshot (Post-Rescale to 1920x1080)
- Playfield: 1440w x 1080h
- Breaker: width=216, height=36, max_speed=900, accel=5400, decel=4500
- Dash: mult=4.0x, dur=0.15s -> covers 540 units (37.5% of playfield)
- Bolt: base=720, min=360, max=1440, radius=14
- Bump: perfect_window=0.15s, early=0.15s, late=0.15s
- Bump cooldowns: perfect=0.0, weak=0.15s
- Bump multipliers (Aegis): perfect=1.5x, early/late=0.8x
- Tilt: dash=15deg, brake=25deg
- Max reflection: 75deg from vertical

### What's Excellent
- Bolt reflection (full overwrite with hit_fraction + tilt)
- Bump system (forward/retroactive paths, whiff punishment, dash-cancel)
- Archetype behavior binding (data-driven, composable, RON-extensible)
- Timer as tension engine (no grace period, visual escalation)
- System ordering (completion before timer, bump after collision)
- Data-driven parameters (all RON + Rust defaults)

### Escalation Curve (3-node run)
- Scatter: 60s, 34 HP, 0.57 HP/s
- Corridor: 75s, 54 HP, 0.72 HP/s
- Fortress: 70s, 70 HP, 1.00 HP/s
