---
name: Parameters & Config Status
description: Current parameter values, review status, and data-driven config status per domain
type: reference
---

## Review Status
- Full codebase audit (2026-03-19): All implemented mechanics reviewed against 9 design pillars
- Prior review (2026-03-16): Phase 1 APPROVED, Phase 2b APPROVED, Phase 2c APPROVED
- 2 BLOCKING issues (test 0.8x values, Prism bolt-lost too soft)
- 5 IMPORTANT issues (run-end dead air, subtitle copy, chip timer, layout pools, passive/active differentiation)
- 3 MINOR issues (RON annotation, perfect window, introduced_cells)

## Key Parameter Values (Post-Rescale)
- Playfield: 1440w x 1080h
- Breaker: width=216, height=36, max_speed=1800, dash_mult=4.0x, dash_dur=0.15s
- Bolt: base=720, min=360, max=1440, radius=14
- Bump: perfect_window=0.15s, early=0.15s, late=0.15s
- Bump multipliers (Aegis/Chrono RON): perfect=1.5x, early/late=1.1x
- Bump multipliers (test code — WRONG): perfect=1.5x, early/late=0.8x
- Bump cooldowns: perfect=0.0, weak=0.15s
- Tilt: dash=15deg, brake=25deg
- Max reflection: 75deg from vertical
- Dash covers 540 units = 37.5% of playfield width
- BASE_BOLT_DAMAGE: 10.0
- Chip select timer: 10.0s (recommend 8.0s)
- Cell standard HP: 10, tough HP: 30, lock HP: 10, regen HP: 20
- Regen rate: 2.0 HP/s (confirmed NOT scaling with hp_mult — correct)
- Difficulty tiers: 5 (hp_mult 1.0->2.5, timer_mult 1.0->0.6, active_ratio 0.0->1.0)
- Boss HP mult: 3.0x, timer reduction per boss: 0.1
- Prism bolt-lost: TimePenalty(3.0) — too soft, recommend 7-8s or LoseExtraBolts

## Data-Driven Config Status
- bolt: RON + BoltDefaults + BoltConfig — COMPLETE
- breaker: RON + BreakerDefaults + BreakerConfig — COMPLETE
- cells: RON + CellDefaults + CellConfig — COMPLETE
- physics: RON + PhysicsDefaults + PhysicsConfig — COMPLETE
- playfield: RON + PlayfieldDefaults + PlayfieldConfig — COMPLETE
- mainmenu: RON + MainMenuDefaults + MainMenuConfig — COMPLETE
- timerui: RON + TimerUiDefaults + TimerUiConfig — COMPLETE
- archetype: RON + ArchetypeDefinition + ArchetypeRegistry — COMPLETE
- chipselect: RON + ChipSelectDefaults + ChipSelectConfig — COMPLETE (RON annotation stale)
- difficulty: RON + DifficultyCurveDefaults + DifficultyCurve — COMPLETE
- chips: RON + ChipDefinition + ChipRegistry + ChipInventory — COMPLETE
- cell types: RON + CellTypeDefinition + CellTypeRegistry — COMPLETE (standard, tough, lock, regen)

## Shockwave / Overclock Parameters
- Shockwave base range (Surge): 64.0, range_per_level: 32.0 — stacked range at 2 stacks = 96.0
- Shockwave damage: scales with DamageBoost — formula: BASE_BOLT_DAMAGE * (1.0 + boost) — IMPLEMENTED
- Surge trigger chain: OnPerfectBump(OnImpact(Cell, Shockwave))
- Surge rarity: Rare, max_stacks: 1 (note: range_per_level is dead weight until max_stacks > 1 or evolution adds stacks)
- Cell grid spacing: 133 horizontal (126w + 7pad), 50 vertical (43h + 7pad)
- At stacks=1 range 64: hits ~1-3 cells (vertical strip only)
- At stacks=2 range 96: hits ~3-6 cells (vertical + near-diagonal neighbors)
- At stacks=3 range 128: hits ~5-10 cells (covers ~2.5 cell widths in all directions)

## Offering Parameters (2026-03-22)
- Rarity weights: Common=100, Uncommon=50, Rare=15, Legendary=3 (code defaults — NOT in RON yet)
- Seen decay factor: 0.8 (code default — NOT in RON yet)
- Offers per node: 3 (code default — NOT in RON yet)
- Chip select timer: 8.0s (RON value, down from 10.0s default)
- Transition out duration: 0.5s, in duration: 0.3s (code defaults — no transition RON file)
- Flash color: white [1.0, 1.0, 1.0], Sweep color: neon cyan [0.0, 0.8, 1.0]

## Highlight Parameters (2026-03-23, RON: defaults.highlights.ron)
- clutch_clear_secs: 3.0 — appropriate
- fast_clear_fraction: 0.5 — appropriate
- perfect_streak_count: 5 — appropriate
- mass_destruction_count: 10, window: 2.0s — watch for over-triggering (window was 1.0s in original spec)
- combo_king_cells: 8 — appropriate
- pinball_wizard_bounces: 12 — high bar, appropriate for expert-only
- speed_demon_secs: 5.0 — depends on node timer, revisit in playtesting
- close_save_pixels: 20.0 — tight (bolt radius 14px), appropriate
- comeback_bolts_lost: 3 — maps to Aegis lives, thematic
- nail_biter_pixels: 30.0 — slightly more generous than CloseSave, appropriate
- untouchable_nodes: 2 — achievable but meaningful
- highlight_cap: 5 — up from spec's 3, good for story-telling (Pillar 9)
- FadeOut duration for juice popups: 2.0s (code default, not yet in RON)

## Shockwave VFX Parameters (2026-03-24)
- Annulus inner/outer: 0.85 / 1.0 (thin ring — max spectacle, zero confusion)
- Color: linear_rgba(0.0, 4.0, 4.0, 0.9) — HDR neon cyan
- Alpha: fades from 0.9 to 0.0 as wavefront expands (progress-based)
- Expansion speed: configured per TriggerChain::Shockwave `speed` field

## Chain Bolt Parameters (2026-03-24)
- Spawns at anchor position + spawn_offset_y
- Velocity: base_speed, randomized within respawn_angle_spread
- Tether: DistanceConstraint with configurable max_distance (tether_distance)
- Position correction: symmetric (half_correction each bolt)
- Velocity redistribution: momentum conservation along chain axis, only when NOT both converging
- Marked ExtraBolt — despawns on loss, never respawns

## Shield Cell Parameters (2026-03-24)
- ShieldBehavior: count (orbit children), radius, speed (rad/s), hp, color_rgb — all RON-configurable
- Orbit cell dim: 20.0 * scale
- Radius clamped: max(shield.radius * scale, orbit_half_diag + 1.0)
- Parent starts Locked with LockAdjacents(orbit_ids) — unlock when all orbits destroyed
- Orbit cells NOT required_to_clear (but parent IS, forcing engagement)
- Orbit cell HP scales with hp_mult from difficulty tier
- Valid shield speeds: 0.0 (stationary) to 2*PI (full rotation per second)

## Playtest Tuning Knobs (ordered by impact)
1. perfect_window: 0.15s -> try 0.10-0.12s if too easy
2. ~~chip_select_timer: 10.0s -> try 8.0s~~ DONE (RON at 8.0s)
3. dash_mult: 4.0x -> try 2.5-3.0x if positioning feels too forgiving
4. regen_rate: 2.0 -> lower if stalemates occur
5. timer_mult per tier: current 1.0->0.6 -> steeper curve if late game is too comfortable
6. prism_bolt_lost_penalty: 3.0s -> 7-8s or add LoseExtraBolts consequence
7. seen_decay_factor: 0.8 -> try 0.7 if offerings feel too samey across runs
