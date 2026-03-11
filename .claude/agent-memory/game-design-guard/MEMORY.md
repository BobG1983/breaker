# Game Design Guard Memory

## Phase 1 Validation (2026-03-10)
- Phase 1 core mechanics APPROVED against design pillars
- See `phase1-review.md` for detailed findings

## Open Issues
- Perfect bump dash-cancel NOT YET IMPLEMENTED (deferred due to query conflicts) -- this is the crown jewel mechanic, must ship
- Bump grade detection uses bolt_velocity.y > 0 as collision proxy -- fragile, should use BoltHitBreaker message instead
- Bolt-lost respawn has no weight/penalty pre-Phase 2 -- respawn at min_speed recommended

## Key Parameter Values (Phase 1)
- Breaker: max_speed=500, dash_mult=2.0x, dash_dur=0.15s
- Bolt: base=400, min=200, max=800, min_angle=~10deg
- Bump: duration=0.3s, perfect_window=0.05s, early_window=0.15s
- Bump multipliers: perfect=1.5x, early/late=0.8x (penalty!), none=1.0x
- Tilt: dash=~15deg, brake=~25deg
- Max reflection: ~75deg from vertical

## Design Principles Confirmed
- Mistimed bumps (0.8x) are WORSE than no bump (1.0x) -- mashing is punished
- Bolt reflection completely overwrites direction -- every breaker contact is skill expression
- Dash requires directional commitment -- no stationary dashes
- Three control axes: position, tilt angle, bump timing
