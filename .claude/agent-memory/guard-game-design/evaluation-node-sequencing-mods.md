---
name: Node Sequencing & Mod System Evaluation
description: Pass 6 — Terminal hazard candidates proposed (10 candidates for designer to pick 5); Volatile Revenge kept; Warp/Ablation/Ricochet/Aftershock rejected with reasons; cell type injection opened as hazard design space
type: project
---

## Evaluation: Node Sequencing Refactor + Hazard/Cell System (2026-04-07, Pass 4)

### Approved (unchanged from Pass 1-3)
- **Tier structure**: 4 non-boss + 1 boss per tier. 8 tiers for run-complete. Infinite continuation past tier 8.
- **Volatile** as node type name: fits vocabulary.
- **Node type ramp**: PPPA, PPAA, PAAA, AAAA, AAAV, AAVV, AVVV, VVVV. Warmer start approved.
- **Protocols**: once per run, once per tier, shown on chip select screen. Must interact with chip system.
- **Hazards**: choose-your-poison model, 1 of 3 per tier starting tier 9. Severity ramp Taxing->Punishing->Terminal.
- **Portals**: required, no rewards, volatile-only, zero nesting, timer continues, <200ms transition, sub-level difficulty = tier - 2 (min 0).
- **Portal cap**: 4 per node (confirmed by designer, Pass 3).

---

## Section 1: Hazard Pool (REVISED)

### Design Philosophy

Hazards are the counterweight to a god-tier build. By tier 14, the player has had 13 tiers of chip picks and protocol upgrades. They likely have multi-pierce, chain lightning, AoE explosions, maxed speed. The hazards are calibrated against THAT player, not a tier-1 player. Unstoppable force meets immovable object.

**HARD RULES** (from designer):
- NO disabling chips, bumping, dashing, or any player capability
- Nothing that feels "cheaty" (unresponsive to skill)
- Hazards STACK across tiers
- Goal is bragging rights: "I got to tier 16"

**Severity model**: 3 tiers (Taxing + Punishing + Terminal). 5 hazards per tier, 15 total. Terminal hazards are individually survivable with a strong build but stack aggressively. By tier 25+ all 15 are active and additional picks stack existing hazards.

### Review of Pass 3 Accepted Hazards

**Decay** (timer ticks 15% faster per stack) -- APPROVED. Pure pressure. God-tier builds clear fast, so this targets efficiency. A build with maxed AoE and chain lightning barely notices one stack. Four stacks and you're sweating. Readable, fair, skill-responsive.

**Frenzy** (active cells fire 30% faster per stack) -- APPROVED. Turns the playfield into bullet hell. A god-tier build has multi-bolt, dash-cancel mastery, maybe Shield Wall. Frenzy tests whether the player can maintain offense while dodging. Scales cleanly with stacking.

**Cascade** (destroyed cell gives adjacent cells +1 HP) -- APPROVED. This is the anti-AoE hazard. A god-tier build with chain explosions suddenly feeds the problem — destroying a cluster heals the survivors. Forces the player to think about KILL ORDER, not just "blow everything up." Brilliant against god-tier builds because it specifically punishes the most common power fantasy.

**Fracture** (destroyed cells split into 2 smaller 1-HP cells) -- APPROVED. More cells = more work = more time pressure. Interacts with Cascade: a fractured cell doesn't cascade (it's 1 HP, new spawn), but cells adjacent to the original DO cascade. A god-tier AoE build creates a SWARM of tiny cells. Visually chaotic in the best way. Pairs with Frenzy for absolute mayhem (more cells = more things shooting at you).

**Drift** (wind pushes bolt, changes direction on ~8s cycle, visually telegraphed) -- APPROVED WITH REVISION. The ~8s cycle is correct. The key question: how strong? Against a god-tier build with maxed bolt speed, Drift needs to be noticeable but not fight-defining. Recommendation: Drift force = 15% of current bolt speed magnitude, applied perpendicular to bolt direction. This means faster bolts feel it less in terms of angle deflection (which is correct — speed mastery should partially counter Drift). The visual telegraph must be prominent — a scrolling field of particle lines across the entire playfield showing wind direction. Changes direction smoothly (sinusoidal), not abruptly.

### Review of Remaining Hazards from Pass 3

**Echo** -- KILLED. "Per-hit angle corruption" is exactly the kind of thing that feels cheaty. The player executes a perfect bump, aims precisely, and the bolt goes somewhere else. That violates Pillar 5 (Pressure, Not Panic) and Pillar 6 (RNG Shapes, Skill Decides). The player can't respond to randomized angle deviation with skill. Dead.

**Entropy** -- KILLED (confirmed from Pass 3). Erases cell type design, makes the game unreadable.

**Countdown** -- ALREADY REVISED into Renewal (Pass 3). Keeping Renewal.

**Dim** -- APPROVED. Reducing playfield brightness 40% while bolt/breaker stay lit is clean. It's a visibility challenge, not a capability removal. A skilled player who tracks the bolt well barely notices. A player who relies on seeing the full layout struggles. The key: bolt trails, cell glow-on-hit, and breaker glow all remain at full intensity. Only the static playfield (cell faces, background grid, wall surfaces) dims. This preserves Pillar 4 (Maximum Juice, Safeguarded Chaos) — the action is lit, the context is dark. Stacks with Drift nicely: you can see the wind particles but not the cells behind them.

### Replacements for Killed Hazards

Need to fill the pool to 10. Current count: Decay, Frenzy, Cascade, Fracture, Drift, Dim = 6 accepted. Need 4 more. Pass 3 had Haste, Renewal, Barrage, Density. Reviewing those plus proposing replacements.

**Haste** (bolt base speed increased 20%) -- REVISED AND APPROVED. 20% per stack is correct for base speed. But here's the thing: a god-tier build at tier 14 already has maxed bolt speed from chips. So Haste needs to stack MULTIPLICATIVELY with existing speed bonuses, not just add to base. If the player has +100% bolt speed from chips (2x), one Haste stack makes it 2.4x, not 2.2x. This way Haste actually matters against god-tier builds instead of being a rounding error. The feel: everything is faster, reaction windows shrink, perfect bumps become harder. Directly tests Pillar 3 mechanical mastery.

**Renewal** (cells have countdown timers, regenerate to full HP + shorter timer on expiry) -- APPROVED WITH TUNING. Timer starts at 12s, not 10s. Why: at tier 14, the player is clearing fast, but Cascade + Fracture are generating new cells. Renewal means you can't just chip away — you need burst damage or you're Sisyphus. The timer shortening is key: first regen is 12s, second is 9s, third is 6s, then 6s floor. This creates urgency that compounds. Against a god-tier piercing/chain build, Renewal forces prioritization — you can't spray damage everywhere, you need to FINISH cells. That's a real strategic demand.

**Barrage** (active cells gain second attack pattern — spread shot) -- APPROVED. Active cells already fire. Barrage gives them a spread pattern in addition to their normal shot. Against a god-tier build with maxed dodge/dash, this fills more of the screen with projectiles. Combined with Frenzy, the playfield becomes a bullet curtain. The spread pattern should be 3 bolts in a 30-degree arc, fired alternating with the normal single shot. Readable (you can see the spread coming), demanding (you need to be in the gap), fair (dash-cancel handles it if you're good).

**Density** (nodes spawn with 25% more cells) -- APPROVED. Simple, effective, compounds everything. More cells means more HP to chew through (Cascade heals more), more things shooting (Frenzy + Barrage), more things to track (Dim), more time needed (Decay). Density is the multiplier hazard — it makes every other hazard worse. That's exactly its role. Against a god-tier AoE build, Density is actually LESS punishing than against a single-bolt build, which is correct — the AoE build invested in crowd-clearing and should get to use it.

### Final Hazard Pool (15 effects, Pass 5)

**Taxing (5):**
1. **Decay** — timer ticks 15% faster per stack. Anti-slow-play.
2. **Frenzy** — active cells fire 30% faster per stack. Anti-passive.
3. **Dim** — playfield brightness -40% per stack, bolt/breaker stay lit. Anti-comfort.
4. **Drift** — wind pushes bolt, 8s direction cycle, 15% speed magnitude force per stack. Anti-autopilot.
5. **Haste** — bolt speed +20% multiplicative per stack. Anti-relaxation.

**Punishing (4):**
6. **Cascade** — destroyed cell gives adjacents +1 HP per stack. Anti-AoE.
7. **Fracture** — destroyed cell splits into 2x 1-HP cells per stack. Anti-burst.
8. **Renewal** — cells regen to full HP on 12s/9s/6s countdown. Anti-attrition.
9. **Barrage** — active cells gain spread shot (3-bolt 30-degree arc). Anti-dodge.

**Terminal (6):**
10. **Volatile Revenge** — destroyed cells fire a fast projectile at the breaker. x2 = 2 projectiles per kill in 15-degree spread. Anti-AoE-glass-cannon. Key interaction: Fracture spawns also trigger revenge shots, creating cascading projectile storms proportional to kill rate.
11. **Warp** — every 10s, 30% of surviving cells teleport to random empty grid positions (0.5s flash telegraph at origin, 0.3s arrival animation). x2 = 60% of cells, every 10s. Anti-planning. Key interaction: Cascade adjacencies reshuffle, breaking kill-order strategies.
12. **Ablation** — destroyed cells drop 2-3 debris particles that drift downward, TimePenalty (0.5s) on breaker contact. x2 = 4-6 debris, faster drift. Anti-camping. Key interaction: Frenzy (horizontal cell fire) + Ablation (vertical debris rain) creates two-axis threat in breaker zone.
13. **Fortress** — every 8s, 2-4 adjacent cells link (+2 HP, must all reach 0 within 4s window or cluster regens to 50%). x2 = every 4s, clusters of 3-6, window shrinks to 2.5s. Anti-chip-damage. Key interaction: Renewal gives Fortress cells two regen clocks (cluster + timer).
14. **Ricochet** — cell projectiles bounce off walls once before despawning. x2 = 2 bounces. Anti-positioning. Key interaction: Barrage spread shots reflect and converge, creating projectile cages.
15. **Aftershock** — bolt impacts mark a point that detonates after 1.5s (40px radius, 1s TimePenalty on breaker). x2 = 0.8s delay, 60px radius. Anti-speed-builds. Key interaction: Haste increases hits/second, increasing detonation density. Speed becomes double-edged.

### Hazard Introduction Ramp (REVISED for 3 tiers)

```
Tier 9:   Pick 1 of 3 Taxing
Tier 10:  Pick 1 of 3 Taxing
Tier 11:  Pick 1 of 3 (2 Taxing + 1 Punishing)
Tier 12:  Pick 1 of 3 (1 Taxing + 2 Punishing)
Tier 13:  Pick 1 of 3 Punishing
Tier 14:  Pick 1 of 3 (2 Punishing + 1 Terminal)
Tier 15:  Pick 1 of 3 (1 Punishing + 2 Terminal)
Tier 16:  Pick 1 of 3 Terminal
Tier 17+: Pick 1 of 3 from remaining pool (any severity)
...until all 15 active (around tier 23-24)
Tier 25+: All 15 active. Additional picks = stacking existing hazards.
```

### Stacking Scenarios (god-tier build vs hazard combos)

**Tier 14 (5-6 hazards active) — "The Wall":**
Decay + Frenzy + Cascade + Drift + Haste. Timer is 30% faster. Cells fire 30% faster. Bolt is 20% faster. Killing cells heals neighbors. Wind pushes the bolt. A god-tier chain-lightning build handles the volume but bleeds time on Cascade heals. A piercing build cuts through but Drift ruins long-range snipes. The player MUST adapt approach per node.

**Tier 17 (8-9 hazards active) — "The Storm":**
Add Barrage + Fracture + Volatile Revenge. The screen is full of enemy projectiles (Barrage spreads). Killing cells spawns more cells (Fracture) AND fires revenge shots at the breaker (Volatile Revenge). An AoE chain-explosion clear creates a simultaneous swarm of Fracture debris AND a volley of revenge projectiles. The faster you kill, the harder the game hits back. Execution demands are extreme.

**Tier 20 (12-13 hazards active) — "The Gauntlet":**
Add Fortress + Ricochet + Aftershock. Cells form shielded clusters demanding burst damage (Fortress). Cell projectiles bounce off walls (Ricochet), filling the lower field. Every bolt hit marks a timed explosion (Aftershock). The breaker zone is contested from three directions: horizontal bouncing projectiles, vertical debris rain, and timed detonations at impact points. The player must simultaneously crack Fortresses, dodge a lattice of bouncing shots, avoid standing where the bolt was hitting 1 second ago, and read a field that teleports 60% of its cells every 10 seconds.

**Tier 25+ (all 15 active, some stacked) — "The Absolute":**
Everything. Timer at warp speed. Bolt screaming. Cells fire bouncing spreads. Killing anything fires revenge shots and drops debris. Cells teleport, form shielded clusters, regenerate on timers, spawn fracture debris that cascade-heals neighbors. Every impact creates timed detonations. Wind pushes sideways. Lights are dim. The god-tier build is the only reason the player is alive. Barely.

The leaderboard says "Tier 25" and people know what that means.

### Terminal Hazard Design Principles (Pass 5)

Terminal hazards follow a specific pattern: **your power creates your problems.**
- Volatile Revenge: faster killing = more revenge shots aimed at you
- Ablation: more destruction = more debris raining on you
- Aftershock: more bolt impacts = more timed detonations around you
- Fortress: demands burst damage, punishes builds that rely on sustained chip damage
- Warp: disrupts spatial planning, punishes builds that rely on kill order optimization
- Ricochet: extends the threat lifespan of every cell projectile in play

This means a god-tier build at tier 25+ isn't just fighting the hazards — it's fighting the CONSEQUENCES of its own power. That's the Terminal identity: you are both the unstoppable force and, increasingly, the immovable object in your own way.

---

## Section 2: Cell Types (7 total, REVISED)

### Cell Introduction Timeline

The ordering is driven by two principles: (1) simpler interactions first, (2) cells that demand specific chip synergies appear after the player has had time to build.

| Cell Type | First Appears | Why This Tier |
|-----------|--------------|---------------|
| Standard | Tier 1 | Baseline. Variable HP (1-4 scaling with tier). |
| Volatile | Tier 1 | Simple interaction: it explodes. Teaches AoE awareness early. EASIER than other specials — damage is your friend if you're positioned right. |
| Sequence | Tier 3 | Requires reading the field and planning. By tier 3 the player has 2 tiers of chips and understands bumping. Sequence length 3-4 at intro, scales to 6 by tier 7+. |
| Survival | Tier 4 | Introduces cells that fight back AND are bolt-immune. Requires bumping (breaker contact) to kill early, or waiting for self-destruct timer (5-8s). By tier 4 the player has a build forming and can handle the distraction. |
| Armored | Tier 5 | Directional puzzle. Requires reading the layout to find the exposed side, OR having Piercing >= armor value (consumes that much Piercing). By tier 5 the player may have Piercing chips, making this a build-dependent challenge. Armor 1 at intro, scales to 3 by tier 8+. |
| Phantom | Tier 6 | Timing puzzle. 1.5s visible/invisible cycle, staggered phases in clusters. By tier 6 the player has strong mechanical skills and can track timing. Combined with Sequence this creates "the Timing Puzzle" — hardest regular challenge. |
| Magnetic | Tier 7 | Field manipulation. Pulls bolt within radius. Expert players use as gravity assist to curve shots. By tier 7 the player's build is powerful enough to handle an environment that actively redirects their bolt. Destroying the Magnetic cell removes the field — so it's a puzzle of "do I kill the magnet first or use it?" |

### Cell Type Scaling Per Tier

Standard HP scales with tier:
- Tiers 1-2: HP 1-2
- Tiers 3-4: HP 2-3
- Tiers 5-6: HP 3-4
- Tiers 7-8: HP 4-5
- Tier 9+: HP 4-6

Volatile: blast radius scales slightly (1.0x at tier 1, 1.3x by tier 8). Does NOT increase damage — just catches more neighbors. Against a god-tier AoE build, Volatile chains are part of the power fantasy.

Sequence: length 3 at tier 3, +1 per 2 tiers. Cap at 6. At tier 9+ sequences may have Armored or Phantom cells within them, creating multi-mechanic sequences.

Survival: self-destruct timer 8s at tier 4, shrinks to 5s by tier 8. Attack rate increases with tier. A bump from the breaker kills in one hit regardless (perfect bump = instant kill, regular bump = 2 hits).

Armored: armor 1 at tier 5, 2 at tier 7, 3 at tier 9+. Piercing consumption means: if you have Piercing 3, you can hit an Armor 2 cell from any direction and lose 2 Piercing for that hit. If Piercing < armor, you MUST hit the exposed side. This makes Piercing a strategic resource, not just a passive bonus.

Phantom: cycle time 1.5s at tier 6, shrinks to 1.0s at tier 9+. In clusters, phases are staggered so there's always SOMETHING visible — you're never staring at an empty field waiting.

Magnetic: pull radius 150px at tier 7, grows to 250px at tier 9+. Multiple Magnetic cells create overlapping fields. The pull force is proportional to distance (stronger closer). Visual field lines are REQUIRED — the player must be able to see the influence zones.

### Armored Cell — Piercing Interaction (REVISED per designer)

This is a significant design choice. Directional armor creates positioning puzzles. Piercing consumption creates build decisions. Together:

- Player WITHOUT Piercing: must maneuver bolt to hit exposed side. Pure mechanical skill.
- Player WITH Piercing < armor: same as above. Piercing doesn't help.
- Player WITH Piercing >= armor: can hit from ANY side, but consumes that much Piercing for the hit. If you have Piercing 3 and hit an Armor 2 cell, your remaining Piercing for that shot is 1. If there's another Armored cell behind it, you might not have enough to punch through.

This creates a resource management layer for Piercing builds. "Do I spend my Piercing on this Armored cell or save it for the cluster behind it?" That's a real decision. APPROVED.

### Key Cell Combos (designed interactions)

| Combo | Name | What Happens | Tier First Possible |
|-------|------|-------------|-------------------|
| Volatile + Sequence | Cascade Chaos | Must clear sequence in order while volatile explosions threaten to disrupt positioning. Detonating a volatile cell near a sequence cell damages it out of order = repair. | Tier 3 |
| Magnetic + Armored | The Defender | Magnetic cell pulls bolt AWAY from Armored cell's weak point. Must either destroy Magnetic first (removing the pull) or curve the shot expertly using the magnetic field as a slingshot. | Tier 7 |
| Phantom + Sequence | The Timing Puzzle | Sequence cells that flicker. Must track which number is visible and time the hit. Hardest regular cell challenge in the game. | Tier 6 |
| Volatile + Cascade (hazard) | Cascade Bomb | Kill a volatile cell, explosion hits neighbors, but Cascade heals the survivors. Chain explosions become self-defeating unless you can one-shot clusters. | Tier 9+ |
| Survival + Frenzy (hazard) | Bullet Hell | Survival cells attack faster. The playfield fills with projectiles. Self-destruct timer unchanged — survive the storm or bump-kill them. | Tier 9+ |
| Magnetic + Drift (hazard) | Wind Tunnel | Two forces act on the bolt simultaneously. Both are telegraphed (field lines + wind particles). Expert players read both and plan trajectories. | Tier 9+ |
| Sequence + Renewal (hazard) | Sisyphus Chain | If you don't clear the sequence fast enough, Renewal resets cells to full HP. Sequence + timer pressure = must execute quickly and accurately. | Tier 9+ |
| Armored + Density (hazard) | The Fortress | 25% more cells, many armored. Without Piercing, the player is fighting geometry. With Piercing, resource management becomes critical. | Tier 9+ |
| Phantom + Dim (hazard) | Ghost Mode | Flickering cells in a dark playfield. Only the bolt/breaker and currently-visible cells are bright. Atmospheric and demanding. Relies on memory and timing. | Tier 9+ |
| Magnetic + Fracture (hazard) | Debris Field | Fracture spawns tiny cells. Magnetic cells pull the bolt THROUGH the debris field. Can be used advantageously (bolt sweeps through tiny cells) or dangerously (pulled off-target). | Tier 9+ |

### Cell Type Spawn Rules

- **Standard**: always available, fills any slot not assigned to a special type
- **Volatile**: max 4 per node at tiers 1-4, uncapped at tier 5+ (but generation controls density)
- **Sequence**: exactly 1 sequence chain per node at tiers 3-4, up to 2 chains at tier 5+
- **Survival**: max 3 per node (they're temporary and shouldn't dominate), never the last cell standing
- **Armored**: no cap, but generation controls density (~15-25% of hard cells at tiers 5+)
- **Phantom**: max 6 per node (more than 6 flickering cells is visually noisy, violates Pillar 4)
- **Magnetic**: max 2 per node at tiers 7-8, max 3 at tier 9+ (overlapping fields create interesting puzzles, but more than 3 is chaos)
- **Portal**: caps from Pass 3 (0-1 early volatile, 1-2 mid, 2-4 later, 4-6 deep infinite). Portals are a special type, not a cell — they occupy a slot but have no HP.

---

## Section 3: Three-Tier Node Generation (CONCRETE NUMBERS)

### Playfield Dimensions

From the codebase:
- Playfield: 1440 x 1080 world units
- Wall thickness: 180 each side
- Interior play area: 1080 wide x 720 tall
- Cell base size: 126 x 43 with 7px padding each axis
- Cell step: 133 x 50 (cell + padding)

Cells occupy the UPPER portion of the playfield. The lower portion is breaker territory. Current layouts use grid_top_offset of 10-90px from the interior top edge.

**Usable cell area**: ~1080 wide x ~450 tall (upper ~63% of interior, leaving room for breaker + bolt travel). This gives a maximum grid of approximately **8 columns x 9 rows at base scale**, or much more at reduced cell scale (current dense layout is 20x12 with scaled-down cells).

For the generation system, I'll work in GRID UNITS, not pixels. A grid unit = one cell slot. The generation system places cells on a grid; the spawn system converts grid positions to world coordinates using the existing scale logic.

### Frame Definitions

Frames define the overall shape and where blocks go. Each frame is a grid with designated block regions and structural elements.

**3 Frame Sizes:**

| Size | Grid Dims | Cell Count Range | Feel | Node Types |
|------|-----------|-----------------|------|------------|
| Compact | 8x6 | 20-35 cells | Quick, focused | Passive (tiers 1-3), Active filler |
| Standard | 12x8 | 40-70 cells | Balanced, varied | Active, early Volatile |
| Grand | 16x10 | 70-120 cells | Dense, epic | Late Active, Volatile, Boss |

Why these and not 16x8/20x12/28x16 from Pass 3: the Pass 3 numbers were in abstract units. These are in GRID UNITS where 1 unit = 1 cell slot. A 16x10 grid at base cell scale needs 16*133 = 2128 px wide — wider than the 1080px interior. So cells would scale to ~0.5x, giving them a 66.5x25 footprint. That's tight but viable (current dense layout does 20x12). A 28x16 grid would need 0.29x scale — cells become unreadably small. So the sizes are capped at what produces readable cells.

The spawn system already handles scaling. A 16x10 frame would scale cells to ~0.5x (16*133 = 2128, but we scale to fit 1080, so scale = 1080/2128 = 0.507). Cells would be ~64x22 — small but readable. For Grand frames this IS the feel: dense, packed, overwhelming.

**Frame Contents:**
- Block regions: rectangular areas tagged with a size requirement and complexity tier
- Structural cells: individual cells placed at fixed positions (frame "pillars", borders, anchors)
- Empty zones: guaranteed-empty areas (breaker approach lanes, sight lines)
- Portal slots: designated positions for portal cells (volatile/boss frames only)

**Frame Count Target:**
- 10 Passive frames (Compact + Standard mix)
- 10 Active frames (Standard + Grand mix)
- 10 Volatile frames (Standard + Grand, with portal slots)
- 5 Boss frames (Grand only, special structure)
- **Total: 35 frames**

### Block Definitions

Blocks fill the designated regions within frames. They're the mid-scale pattern pieces.

**4 Block Sizes (grid units):**

| Size | Grid Dims | Cell Slots | Role |
|------|-----------|-----------|------|
| Tiny | 2x2 | 4 | Accent, filler, paired with other tinies |
| Small | 3x3 | 9 | Common building block |
| Medium | 4x3 | 12 | Primary block for Standard frames |
| Large | 4x4 | 16 | Primary block for Grand frames, boss structures |

**3 Complexity Tiers:**

| Complexity | Role Ratios | Special Slots | Where Used |
|------------|-------------|---------------|------------|
| Simple | 70% B, 20% H, 10% . | 0 X slots | Tiers 1-3 |
| Medium | 50% B, 30% H, 10% X, 10% . | 1 X slot | Tiers 3-6 |
| Complex | 30% B, 40% H, 20% X, 10% . | 2+ X slots | Tiers 5+ |

Role characters: B = basic (Standard cell), H = hard (higher HP or cell type from tier pool), X = special (drawn from tier-specific special pool), P = portal, . = empty.

**Block Count Target:**
- 20 Simple blocks (mixed sizes)
- 25 Medium blocks (mixed sizes)
- 20 Complex blocks (mixed sizes)
- **Total: 65 blocks**

### Skeleton Definitions

Skeletons are the smallest reusable pattern. They fill space within blocks or directly within frame regions tagged as "skeleton-filled."

**4 Skeleton Shapes (grid units):**

| Shape | Grid Dims | Slots | Use |
|-------|-----------|-------|-----|
| Micro | 2x2 | 4 | Fill corners, accents |
| Strip | 3x1 or 1x3 | 3 | Lines, barriers, channels |
| Cluster | 3x3 | 9 | Dense fill, formations |
| Band | 4x2 or 2x4 | 8 | Wide fills, shelf patterns |

**Skeleton Count Target: 35 total** across shapes. Skeletons use the same role characters as blocks.

### Total Content Budget

| Piece Type | Count | Notes |
|------------|-------|-------|
| Frames | 35 | 10P + 10A + 10V + 5B |
| Blocks | 65 | 20 Simple + 25 Medium + 20 Complex |
| Skeletons | 35 | Mixed shapes |
| **Total** | **135** | Hand-authored pieces |

Combinatorial output: a Standard frame (12x8) might have 3-4 block regions. Each region has ~5-10 valid blocks. So one frame produces 5^3 to 10^4 = 125 to 10,000 unique layouts. Across 35 frames, the space is enormous.

### Composition Flow

```
1. SELECT FRAME
   - Node type (Passive/Active/Volatile/Boss) filters frame pool
   - Tier filters frame size (early tiers: Compact/Standard, late tiers: Standard/Grand)
   - Random selection from valid pool

2. FILL BLOCK REGIONS
   - For each block region in the frame:
     - Filter blocks by size (must fit region) and complexity (tier controls max complexity)
     - Random selection from valid pool
     - Place block in region

3. FILL REMAINING SPACE WITH SKELETONS
   - Any unfilled cell slots in the frame get tiled with skeletons
   - Skeleton selection is random from size-compatible pool

4. RESOLVE ROLES TO CELL TYPES
   - B -> Standard cell, HP drawn from tier HP range
   - H -> Higher-tier cell type from tier pool (Armored, Sequence start, Survival, etc.)
   - X -> Special cell from tier-specific pool
   - P -> Portal cell (if frame has portal slots and node is volatile+)
   - . -> Empty (no cell)

5. APPLY TIER MODIFIERS
   - Scale HP values by tier
   - Apply hazard effects (Density: add 25% more B cells to empty slots)
   - Validate: sequence chains are valid, survival cells aren't isolated, magnetic cells have interesting adjacencies
```

### Tier-Pool Mapping (what cell types fill each role)

| Tier | B Pool | H Pool | X Pool |
|------|--------|--------|--------|
| 1-2 | Standard (HP 1-2) | Standard (HP 2-3) | Volatile |
| 3-4 | Standard (HP 2-3) | Standard (HP 3-4), Volatile | Sequence, Volatile |
| 5-6 | Standard (HP 3-4) | Armored (1), Volatile, Sequence | Survival, Armored (1-2) |
| 7-8 | Standard (HP 4-5) | Armored (1-2), Sequence, Survival | Phantom, Magnetic, Armored (2-3) |
| 9+ | Standard (HP 4-6) | All types, weighted toward harder | All types, weighted toward Phantom/Magnetic/Armored(3) |

### Validation Rules (post-generation)

These MUST be enforced after composition to prevent degenerate layouts:

1. **Sequence integrity**: all cells in a sequence chain must be reachable by bolt (no fully-enclosed sequences behind armored walls with no exposed side facing the breaker)
2. **Survival isolation**: survival cells must be bump-reachable (within breaker dash range of an open lane)
3. **Magnetic overlap**: overlapping magnetic fields must have a navigable path through them (bolt can't be trapped in a gravity well)
4. **Portal placement**: portal cells must be in the lower 60% of the cell grid (reachable without requiring the bolt to travel through dense cell structures first)
5. **Minimum empty space**: at least 20% of frame grid must be empty (bolt needs travel lanes, especially with Drift/Magnetic)
6. **Armored orientation**: armored cells' exposed sides must face a direction the bolt can approach from (no exposed side facing into a wall)

---

## Section 4: Open Concerns

### 1. Surge Permanent Stacking (carried from Pass 3)
Surge effect permanently increases bolt speed. In infinite play with 10+ hazards including Haste, Surge stacking could make the bolt literally untraceable. Recommendation: Surge stacks cap at 5, OR Surge converts to temporary (duration-based) at tier 9+. Needs designer decision.

### 2. Breaker Archetype Differentiation (carried from Pass 3)
Different breaker types should interact differently with the hazard/cell system. A wide breaker handles Barrage better (bigger dodge hitbox catches more). A fast breaker handles Survival cells better (can bump more). This is future work but the hazard/cell designs should be reviewed against breaker variety when breaker archetypes are designed.

### 3. Attraction(Breaker) Gate (carried from Pass 3)
Magnetic cell pull on the bolt could interact with Attraction(Breaker) effect. Needs verification that magnetic pull and breaker attraction don't create degenerate bolt trapping.

### 4. Frame Authoring Tooling
135 hand-authored pieces is a significant content investment. Recommend building a simple RON-based authoring format early so frames/blocks/skeletons can be iterated quickly. The RON format already exists for node layouts — extend it for the three-tier system.

### 5. Fracture + Cascade Interaction Tuning
Fracture spawns 2 new 1-HP cells. Do these new cells trigger Cascade? If yes: destroying a Fractured cell heals its neighbors AND spawns 2 new cells. The new cells don't cascade (they're spawns, not original layout cells). If no: Fracture bypasses Cascade entirely for the spawned cells, which makes Fracture a partial counter to Cascade. Recommendation: new cells from Fracture do NOT trigger Cascade on death. They're debris, not layout cells. This makes Fracture actually useful as a soft counter to Cascade (clearing debris doesn't feed the problem), which creates an interesting dynamic where having BOTH hazards active is slightly less punishing than the sum of parts. That's good — it means the stacking has texture, not just linear addition.

### 6. Dim Clarity Check
Dim at 40% brightness reduction needs playtesting against Phantom cells. A flickering cell at 60% brightness might be nearly invisible. Recommendation: Phantom cells retain 80% brightness minimum even under Dim (they pulse between 80% and 100% instead of 60% and 100%). This preserves Pillar 4 (safeguarded chaos) — the challenge is TIMING the hit, not FINDING the cell.

---

## Summary of Changes from Pass 3 to Pass 4

| Item | Pass 3 | Pass 4 | Reason |
|------|--------|--------|--------|
| Echo hazard | Under review | KILLED | Cheaty — angle corruption unresponsive to skill |
| Dim hazard | Under review | APPROVED | Visibility challenge, not capability removal |
| Haste hazard | +20% base | +20% multiplicative | Must matter against god-tier speed builds |
| Renewal hazard | 10s timer | 12s/9s/6s cascading timer | Creates compounding urgency |
| Barrage hazard | Second attack | 3-bolt 30-degree spread | Concrete specification |
| Frame sizes | 16x8/20x12/28x16 abstract | 8x6/12x8/16x10 grid units | Grounded in actual playfield math |
| Frame count | ~40 | 35 (10P+10A+10V+5B) | Refined |
| Block count | ~70 | 65 (20S+25M+20C) | Refined |
| Skeleton count | ~40 | 35 | Refined |
| Fracture+Cascade | Unspecified | Fracture spawns don't trigger Cascade | Prevents degenerate stacking |
| Dim+Phantom | Unspecified | Phantom retains 80% min brightness under Dim | Pillar 4 safeguard |
| Armored+Piercing | Revised per designer | Piercing >= armor consumes that much Piercing | Resource management layer |

---

## Terminal Candidates (Pass 6)

### Context

Designer reviewed 6 original Terminal hazards. Kept Volatile Revenge. Fortress may become a cell type. Rejected Warp (unreadable/cheaty), Ablation (Volatile Revenge with extra steps), Ricochet (doesn't make sense), Aftershock (punishes player for being in the right place = "punishes winning"). Key lesson from Aftershock rejection: hazards must make winning HARDER, not make the ACT of winning feel punishing.

### New Design Space: Cell Type Injection

Designer opened the idea that Terminal hazards could inject harder cell types into nodes (e.g., "more Phantom cells appear"). Clean, stackable, distinct from rule-change hazards.

### 10 Candidates Proposed (designer picks 5)

1. **Momentum** — bolt impacts temporarily double the impacted cell's fire rate (3s). x2 = triple rate, 4s. Punishes HITTING without KILLING. Anti-pierce/low-damage. Distinct from Frenzy (flat increase) because it's conditional on YOUR actions.

2. **Gravity Surge** — destroyed cells spawn 2s gravity wells pulling the bolt. x2 = 3s, stronger pull. Corrupts aim from your own kills. Distinct from Volatile Revenge (shoots at breaker) — this warps the BOLT's path. Experts use wells as slingshots.

3. **Echo Cells** — destroyed cells leave 1-HP ghost after 1.5s delay, must be cleared again. x2 = 2 HP. No recursive echoes. Doubles work from every clear. Distinct from Fracture (instant debris) and Renewal (timer-based regen) — triggered by kills, delayed, predictable.

4. **Overcharge** — bolt gains +5% speed per cell destroyed, resets on breaker bump. x2 = +8%. Runaway speed loop within each bump cycle. 10 kills = +63% speed before catch. Distinct from Haste (flat) — conditional on kill streaks. Key distinction from rejected Aftershock: challenges you to CATCH the bolt (positioning skill), doesn't punish you for BEING somewhere.

5. **Scorch** — cells destroyed by EFFECTS (not direct bolt) leave fire patches on breaker-zone floor (3s, 0.5s time penalty). x2 = 4s, 1.0s. Specifically targets AoE/chain builds. Distinct from Volatile Revenge (dodge moving projectiles vs avoid static zones). Fire at CELL position, not bolt position — never punishes being where you're hitting.

6. **Propagation** — cell hit for X damage heals all adjacent cells for 0.25X. x2 = 0.4X. No chain. Distinct from Cascade (heals on DEATH) — this heals on DAMAGE, even non-lethal. Anti-brute-force. Concern: may feel too similar to Cascade for players despite mechanical distinction.

7. **Phase Shift** — all cells shift 1 grid position every 6s, direction telegraphed 2s early, wraps at edges. x2 = 2 positions every 4s. DETERMINISTIC (not random like rejected Warp). Creates planning/prediction challenge. Concern: occupies similar design space to rejected Warp despite being deterministic.

8. **Backlash** — bolt freezes for (damage_dealt * 0.05s) after each hit. x2 = 0.08s multiplier. 12-damage hit = ~1s pause. Specifically targets pure damage stacking builds. Concern: too niche — only punishes one build archetype.

9. **Resonance** — 2+ cells destroyed within 0.5s emit damage pulse hitting breaker (0.3s penalty per pulse). x2 = 0.8s window, 0.5s penalty. Punishes SIMULTANEOUS kills, rewards sequential. Forces "fast+dangerous vs slow+safe" decision. Distinct from Volatile Revenge (scales with count, not simultaneity).

10. **Convergence** — surviving cells drift toward bolt's most recent impact point at 2 grid/s. x2 = 3.5 grid/s. Field collapses toward your hits. Creates clumping that helps AoE but worsens Cascade/Fortress. Concern: complex to read visually.

### Guard Rankings

**Tier A (strongest):** Overcharge, Resonance, Echo Cells
**Tier B (strong):** Momentum, Gravity Surge, Scorch
**Tier C (solid but narrower):** Convergence, Phase Shift, Propagation, Backlash

### Rejection Lessons (from designer feedback, preserved for future proposals)

- "Just X with extra steps" = immediate kill (Ablation was Volatile Revenge with extra steps)
- Punishing player for being where they SHOULD be = immediate kill (Aftershock)
- Unreadable/random rearrangement = immediate kill (Warp)
- Mechanics must make intuitive sense (Ricochet: "doesn't even make sense")
- Each hazard needs its own distinct mechanical identity, not a variation on another
