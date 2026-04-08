# Protocol Brainstorm -- 15 Initial Designs

## Summary Table

| # | Name | One-liner | Strategy Shift | Unlock Tier |
|---|------|-----------|---------------|-------------|
| 1 | Deadline | Timer pressure = bolt power | Play on the edge of failure | Early |
| 2 | Ricochet Protocol | Wall-bank shots deal 3x damage | Aim for walls, not cells | Early |
| 3 | Convergence Engine | Cell kills pull nearby cells toward the impact point | Group, then nuke | Mid |
| 4 | Undertow | Every bump reverses bolt Y-velocity (it comes back to you) | Rapid-fire bump machine | Mid |
| 5 | Overburn | Bolt trails fire behind it, damaging cells it passes near | Navigate, don't aim | Mid |
| 6 | Debt Collector | Damage stacks on whiffs, cashes out on next cell hit | Controlled losing | Early |
| 7 | Bloodline | Bolt inherits a fraction of every killed cell's max HP as damage | Snowball through hard cells first | Late |
| 8 | Fission | Every Nth cell destroyed splits the bolt permanently | Build toward critical mass | Mid |
| 9 | Iron Curtain | Bolt-lost spawns a timed shield wall + damages all cells once | Turn disaster into offense | Early |
| 10 | Kickstart | Node starts with 3s of 2x bolt speed + 2x damage, then returns to normal | Explosive openers | Early |
| 11 | Echo Strike | Every perfect bump records bolt position; next cell impact replays damage at all recorded positions | Phantom geometry | Late |
| 12 | Gravity Lens | Bolt curves toward the densest cell cluster, strength scales with speed | Fast bolt = strong pull | Mid |
| 13 | Siphon | Cells killed by AoE effects (shockwave, chain lightning, explode) add time to the node timer | AoE = survival | Mid |
| 14 | Overclock Protocol | Perfect bumps grant a stacking speed multiplier that resets on whiff -- but each stack also increases bolt SIZE | Risk tower that changes physics | Late |
| 15 | Tier Regression | Drop back 1 tier of difficulty; gain an extra tier of chip offerings before escalation resumes | Strategic retreat for power | Late |

---

## Protocol 1: Deadline

### One-line description
When node timer drops below 25%, bolt speed doubles and bolt damage doubles.

### Detailed mechanic
Game-rule modifier. Watches `NodeTimerThreshold(0.25)`. When threshold is crossed:
- `SpeedBoost(multiplier: 2.0)` applied to all active bolts
- `DamageBoost(2.0)` applied to all active bolts
- Visual: bolt trails intensify, timer pulses red, screen edges glow with urgency
- Effect persists until node ends

This is a direct promotion of the existing legendary chip. As a protocol, it no longer competes with chip slots -- it shapes the entire run identity.

### Strategy shift
You stop trying to clear nodes fast. You *want* to be in the danger zone. Smart players will intentionally slow-play the first 75% of the timer, positioning bolts and softening cells, then detonate the node in a frenzy when Deadline activates. The entire pacing of your run inverts.

### Synergies
- **Overclock / Feedback Loop chips**: Speed stacks multiplicatively during Deadline window -- bolt becomes a missile
- **Piercing Shot chips**: Double damage + piercing = cells melt in sequence during the final push
- **Chrono breaker**: Time penalty on bolt-lost is devastating when you're already in the danger zone. High risk, high reward pairing.
- **Decay hazard**: Timer ticks faster, so Deadline activates sooner -- sounds good until you realize you have less time to soften cells before the window opens, AND less time in the window itself
- **Haste hazard**: Stacks with Deadline's speed boost multiplicatively -- bolt becomes nearly uncontrollable

### Trap potential
Looks amazing. But if you can't track a 2x-speed bolt for sustained play, you'll lose it during the exact window you need it most. Players who can't execute tight bumps at high speed will lose more bolts in the Deadline window than they gain from the damage. Also a trap with Erosion hazard -- your breaker is smallest when you need to catch the fastest bolt.

### Meta-progression unlock tier
**Early** -- this is the introductory protocol. It's the simplest to understand ("play on the edge"), it's dramatic when it fires, and it teaches the player that protocols fundamentally change your strategy. First protocol every player should see.

---

## Protocol 2: Ricochet Protocol

### One-line description
After a wall bounce, bolt deals 3x damage until its next cell impact.

### Detailed mechanic
Direct promotion of existing legendary chip. Expressible as:
`On(AllBolts) -> When(Impacted(Wall)) -> Until(Impacted(Cell)) -> Do(DamageBoost(3.0))`

Visual: bolt glows bright on wall bounce, trails intensify. Glow fades on cell impact.

As a protocol (not a chip), this effect applies globally to all bolts without consuming a chip slot. The opportunity cost is giving up a chip pick at that tier's offering.

### Strategy shift
You aim for walls, not cells. Every shot becomes a bank shot. The optimal play is to angle the bolt into a wall first, THEN into cells. This completely changes breaker positioning -- you're not trying to aim directly at cell clusters, you're trying to set up ricochet angles. Players who master wall-bank geometry get 3x damage on every cell hit.

### Synergies
- **Aftershock chips**: Wall bounces already trigger shockwaves; now they also charge 3x damage. Wall-bounce builds become a real archetype.
- **Bolt Speed chips**: Faster bolt = more wall bounces per second = more opportunities to maintain the 3x buff
- **Piercing Shot**: 3x damage + piercing = one wall-bounce sequence can clear a column
- **Drift hazard**: Wind pushes bolt sideways -- can accidentally give you wall bounces you didn't plan, or rob you of the angle you needed
- **Gauntlet chip** (if kept as legendary): Big slow bolt bouncing off walls at 3x damage with 0.5x base = 1.5x effective. Interesting math.

### Trap potential
Terrible in dense layouts where the bolt immediately hits a cell after spawning. If the cell grid is close to walls, you can't get a "clean" wall bounce without hitting a cell first. Also bad with Magnetism chips -- attraction pulls bolt toward cells, fighting the wall-bank strategy. Looks like free damage until you realize your build works against it.

### Meta-progression unlock tier
**Early** -- simple mechanic, dramatic effect, teaches the player that protocols change your aiming strategy.

---

## Protocol 3: Convergence Engine

### One-line description
When a cell is destroyed, surviving cells within range drift toward the destruction point.

### Detailed mechanic
New game-rule modifier. On `CellDestroyed`:
- All cells within 96px of the destroyed cell's position begin drifting toward the destruction point
- Drift speed: 8px/s (slow enough to be readable, fast enough to matter over a node)
- Cells stop drifting when they reach the destruction point or collide with another cell
- Cells that cluster together become vulnerable to AoE
- Visual: cells have a subtle pull-trail when drifting, impact point has a brief gravity-lens distortion

This was killed as a hazard ("too chaotic"), but as a positive protocol it becomes a tool the player actively exploits.

### Strategy shift
You become a sculptor. Kill cells on the edges of clusters to pull the rest together, then hit the dense cluster with AoE. The order you destroy cells matters enormously -- kill from the outside in to compress, or kill from the center to scatter. Every node becomes a spatial puzzle: "where do I detonate first to create the best cluster for my AoE?"

### Synergies
- **Cascade / Shockwave chips**: The entire point. Group cells, then shockwave them. Devastating.
- **Powder Keg legendary**: Cells clump together, then explode on death. Chain explosions become trivial to set up.
- **Chain Lightning chips**: Arcs jump between nearby cells -- Convergence brings them into range.
- **Diffusion hazard**: Normally annoying (damage shared with neighbors). With Convergence, cells drift together so Diffusion spreads damage to more targets -- which means more cells taking damage simultaneously, which with enough AoE becomes a positive.
- **Fracture hazard**: Splits create fragments in adjacent cells... which then drift toward the next kill. Debris clumps up instead of spreading out. Turns a hazard into a minor advantage.

### Trap potential
Terrible with single-target builds (pure damage boost, no AoE). If you don't have area damage, clumped cells are just harder to reach individually. Also bad with Tether hazard -- linked pairs drifting together can cause cascading heal loops with Cascade hazard. Looks amazing with AoE until you realize your AoE also triggers Cascade heals on the now-adjacent cells.

### Meta-progression unlock tier
**Mid** -- requires understanding of AoE synergies and spatial reasoning. New players won't know what to do with it.

---

## Protocol 4: Undertow

### One-line description
Every non-whiff bump reverses the bolt's vertical velocity component (it comes back to you).

### Detailed mechanic
New game-rule modifier. On `Bump` (any grade):
- The bumped bolt's Y-velocity is negated (inverted). If it was going up, it now comes down. If it was going down, it already gets bumped up -- so this fires AFTER the bump, meaning the bolt goes up, then this reversal sends it... wait. Let me be precise.

Actually: the bolt gets bumped upward normally. Undertow adds a delayed reversal -- 0.8s after the bump, the bolt's Y-velocity inverts. The bolt fires up, hits cells for 0.8s, then curves back down toward the breaker for another bump. The player becomes a rapid-fire bump machine.

- Delay: 0.8s after bump (tunable)
- Visual: bolt leaves a "boomerang arc" trail, brief flash when reversal fires
- Only affects primary bolt (not spawned bolts, chain bolts, etc.)
- Reversal does NOT fire if bolt has already been bumped again (prevents double-inversion)

### Strategy shift
The entire game rhythm changes. Instead of bump-wait-bump with long intervals, you're bumping every 1-2 seconds. The bolt keeps coming back to you. This massively increases the value of perfect bumps (more opportunities per node) and makes bump-triggered chips fire constantly. The downside: the bolt spends less time in the cell field because it keeps returning.

### Synergies
- **Tempo legendary**: Speed ramps on consecutive bumps -- Undertow gives you 3-4x more bumps per node. Tempo becomes insane.
- **Feedback Loop legendary**: Perfect bump chains fire constantly with more bump opportunities
- **Overclock chips**: Timed speed burst refreshes on every perfect bump. With Undertow, you're bumping every 1.5s -- permanent Overclock uptime.
- **Surge chips**: Permanent speed stacking per perfect bump. Undertow gives you more bumps, so speed escalates faster. (Potential balance concern -- see open design concerns about Surge.)
- **Erosion hazard**: Breaker shrinks, but you're bumping constantly, which restores width. Undertow neutralizes Erosion. This is a GOOD synergy to know about.
- **Prism breaker**: Extra bolts on perfect bump + constant bumps = bolt army

### Trap potential
Looks great for bump-heavy builds. But the bolt spends HALF its time returning to you instead of hitting cells. If your build relies on sustained cell contact (Amp, Piercing Shot chains, cell-death triggers), you're cutting your DPS in half because the bolt is in transit downward instead of hitting things. Terrible with Overcharge hazard -- speed resets on bump, but you're bumping every 1.5s, so the bolt never accelerates through kills.

### Meta-progression unlock tier
**Mid** -- fundamentally changes the game rhythm. Needs experience to evaluate the trade-off.

---

## Protocol 5: Overburn

### One-line description
The bolt damages all cells within a small radius of its path, not just cells it directly hits.

### Detailed mechanic
New game-rule modifier. Continuous effect:
- Bolt has an "aura" radius of 12px (about 40% of a cell width) that damages cells as it passes
- Aura damage = 25% of bolt's current damage per tick (ticks at physics rate)
- Visual: bolt leaves a bright, wide trail. Cells in the trail glow and take damage visibly.
- Does NOT trigger on-impact effects (Impacted triggers) -- only direct hits do
- DOES count as "being hit" for cell HP purposes

### Strategy shift
You stop aiming precisely and start navigating. The optimal path is the one that passes NEAR the most cells, not the one that hits a specific cell. Piercing becomes less valuable (you're already damaging multiple cells per pass). Positioning and angle become about maximizing trail coverage.

### Synergies
- **Bolt Speed chips**: Faster bolt = longer trail per second = more cells damaged per pass
- **Bolt Size chips**: Larger bolt has a proportionally larger aura radius (16px at 1.5x size)
- **Singularity legendary** (if kept as chip): Tiny fast bolt leaves a devastating trail -- 0.6x size but 2.5x damage means 0.625x aura damage at 1.4x speed. The needle becomes a scalpel.
- **Volatility hazard**: Cells gain HP when not being hit -- but Overburn's trail hits everything the bolt passes, suppressing Volatility growth across the board
- **Momentum hazard**: Non-lethal hits grow cells. Overburn's 25% damage ticks are almost always non-lethal. Every pass grows every cell in the trail. DEVASTATING trap synergy.

### Trap potential
**Momentum hazard is the killer.** Overburn ticks are small, non-lethal hits. Momentum punishes non-lethal hits. With both active, every bolt pass grows every cell it doesn't kill. The player who took Overburn early gets crushed by Momentum at tier 10. Also bad with Diffusion hazard -- trail damage bleeds to adjacent cells as non-lethal ticks, growing Momentum further. Looks incredible in the early game; becomes a liability if the wrong hazards stack.

### Meta-progression unlock tier
**Mid** -- the mechanic is simple to understand, but the hazard trap (Momentum) requires meta-knowledge to avoid.

---

## Protocol 6: Debt Collector

### One-line description
Every whiff and bolt-lost stacks damage. Next cell impact cashes out all stacked damage as a single hit.

### Detailed mechanic
New game-rule modifier:
- On `BumpWhiff`: add 2.0x base bolt damage to the debt counter
- On `BoltLost`: add 5.0x base bolt damage to the debt counter
- On `Impacted(Cell)`: the next cell hit deals normal damage + entire debt counter as bonus damage. Counter resets to 0.
- Visual: bolt glows progressively brighter/redder as debt accumulates. Cashout hit has a massive flash + screen shake proportional to debt.
- Debt counter is displayed as a visible number near the bolt (small, unobtrusive)
- Debt persists across nodes within a run

### Strategy shift
Failure becomes fuel. You stop dreading whiffs and bolt-loss and start calculating: "how much debt can I accumulate before the cashout?" Aggressive play is rewarded -- intentionally risky positioning that might cause whiffs becomes a way to charge up devastating single hits. The run becomes a series of controlled disasters followed by nuclear payoffs.

### Synergies
- **Whiplash legendary** (if kept as chip): Already gives bonus damage on whiff -> next impact. Stacks with Debt Collector for absurd single-hit damage.
- **Glass Cannon legendary** (if kept as chip): Bolt-lost = lose life, but also stacks 5.0x debt. The risk/reward is extreme.
- **Desperation legendary** (if kept as chip): Bolt-lost = speed boost + debt stack. Speed helps you reach cells for cashout. Builds feed each other.
- **Aegis breaker**: Lives-based. Bolt-lost costs a life but stacks debt. Deliberate bolt sacrifice becomes a strategy: lose life, gain massive damage.
- **Chrono breaker**: Time penalty on bolt-lost + debt stacking. You're trading time for damage. High-skill players can manage the time budget.

### Trap potential
Looks great for aggressive players. But debt only pays off on the NEXT cell hit -- if you whiff 5 times and then lose the bolt before hitting a cell, the debt keeps growing but you never cash out. On nodes with sparse cell layouts, the debt sits uselessly. Also terrible with Erosion hazard -- whiffing more (to build debt) accelerates breaker shrink, making it harder to bump, causing more whiffs... the spiral is real, but so is the payoff if you survive.

### Meta-progression unlock tier
**Early** -- the concept is immediately compelling ("failure = power") and it teaches a core roguelite lesson: sometimes the weird choice is the right one.

---

## Protocol 7: Bloodline

### One-line description
When a cell is destroyed, the bolt permanently gains bonus damage equal to a fraction of that cell's max HP.

### Detailed mechanic
New game-rule modifier. On `CellDestroyed` (by bolt impact):
- The destroying bolt gains permanent `DamageBoost` equal to `cell_max_hp * 0.05` (5% of cell's max HP added as flat bonus)
- This is FLAT bonus damage, not multiplicative -- it adds to base, then multiplicative chips scale it
- Visual: bolt gains a subtle glow layer per stack. After 20+ stacks, the bolt visibly crackles with absorbed energy.
- Only applies to the bolt that dealt the killing blow (not AoE kills, not chain lightning kills)
- Resets on bolt-lost (the bolt that absorbed the power is gone)

### Strategy shift
Target priority inverts. You want to kill the HARDEST cells first because they give the most damage. In a normal run, you soften easy cells first. With Bloodline, you aim for the high-HP cells because they make your bolt stronger for everything after. Also creates a strong incentive to protect your primary bolt -- that bolt has been accumulating power all run. Losing it is devastating.

### Synergies
- **Damage Boost chips**: Multiplicative chips scale the flat bonus from Bloodline. A bolt with 50 flat bonus damage * 3x DamageBoost = 150 bonus. Bloodline is the BASE that makes damage chips more valuable.
- **Prism breaker**: Multiple bolts, but only the killing bolt gains the bonus. You want ONE primary bolt doing the killing. Anti-synergy with multi-bolt strategies -- GOOD design, creates tension.
- **Echo Cells hazard**: Ghost cells have 1 HP. Killing them gives minimal Bloodline stacks. The hazard wastes your time without feeding your power.
- **Cascade hazard**: Cells healed by Cascade still had their original max HP. Killing a healed cell gives the same Bloodline bonus. Cascade is annoying but doesn't neuter your scaling.

### Trap potential
Terrible with multi-bolt builds. If you have Split Decision, Splinter, or Prism spawning bolts everywhere, kills are distributed across many bolts. No single bolt accumulates meaningful power. Also bad with AoE-heavy builds -- Shockwave and Chain Lightning kills don't grant Bloodline stacks, so your damage investment goes into effects that don't feed the loop. Best with single-bolt, high-damage, precision builds. Worst with chaos builds.

### Meta-progression unlock tier
**Late** -- requires understanding of damage calculation (flat vs multiplicative), target prioritization, and bolt preservation. Expert-only protocol.

---

## Protocol 8: Fission

### One-line description
Every 8th cell destroyed permanently splits one of your bolts into two.

### Detailed mechanic
New game-rule modifier:
- Global destruction counter. Every 8 cell destructions (any source -- direct hit, AoE, chain), trigger a fission event.
- Fission targets the bolt with the fewest active splits (even distribution)
- New bolt inherits parent's current effects (like Split Decision's inherit)
- New bolt launches at 45-degree offset from parent's velocity
- Counter displayed as a subtle ring around the bolt (fills like a charge meter)
- No cap on splits -- this is the "infinite bolts" protocol
- Visual: satisfying fission animation -- bolt briefly expands, splits with a bright flash

### Strategy shift
You become a cell-destruction engine. Every system that kills cells (AoE, chain lightning, explosions) is feeding the fission counter. The early game is patient -- 8 kills per split is slow. But by tier 5, if you've been efficiently destroying cells, you might have 6-8 bolts, each with inherited effects, all feeding more destructions that trigger more splits. The exponential curve comes online if you play fast enough.

### Synergies
- **Chain Reaction legendary** (if kept as chip): Recursive destruction spawns bolts -- which kill more cells -- which feed Fission faster. Exponential bolt growth.
- **Cascade / Shockwave chips**: AoE kills count toward the counter. More AoE = faster splits.
- **Entropy Engine evolution**: Random effects on cell destruction, some of which spawn more bolts or cause more destruction. Feeds the counter.
- **Convergence Engine protocol** (if both taken... wait, one protocol per run. But future design space.): Would be absurd together. Good to note for 30-protocol phase.
- **Overcharge hazard**: Bolt gains speed per cell destroyed within a bump cycle. With 8 bolts, cells die fast, speed ramps insanely between bumps. Sounds like a buff until the bolts become untraceable.
- **Fracture hazard**: Splits create fragments = more cells to destroy = faster Fission counter. Hazard feeds the protocol.

### Trap potential
8 cells per split is slow for precision-damage builds that one-shot cells efficiently with few targets. A build focused on killing 20 cells per node cleanly only gets 2-3 splits per node. Meanwhile, a build that causes chain destruction through 60 cells gets 7 splits. Fission punishes clean, efficient play and rewards chaotic destruction volume. Looks great for everyone; only delivers for AoE builds.

### Meta-progression unlock tier
**Mid** -- simple to understand, but evaluating whether your build has enough destruction volume requires experience.

---

## Protocol 9: Iron Curtain

### One-line description
When a bolt is lost, a shield wall spawns for 3 seconds AND all cells take a flat damage pulse.

### Detailed mechanic
New game-rule modifier. On `BoltLost`:
- Shield wall spawns (visible, blocks bolt passage) for 3.0 seconds
- ALL cells on the field take flat damage equal to 1.0x base bolt damage
- Visual: wall slams up with sparks, damage pulse ripples outward from the bottom of the screen
- Shield wall is destructible -- cells that fire projectiles (future feature) can break it
- Does NOT prevent the bolt-lost penalty (life loss, time penalty, etc.) -- it's a consolation, not a save

### Strategy shift
Bolt-loss becomes a tactical event. The 3s shield wall gives breathing room to reposition, and the damage pulse softens all cells. Aggressive play near the bottom (risking bolt-loss for better angles) becomes viable because losing the bolt isn't pure downside. You're not TRYING to lose bolts, but you're less afraid of it.

### Synergies
- **Aegis breaker**: Lives-based. Bolt-loss costs a life but triggers Iron Curtain. With 3 lives, you get 3 damage pulses across the run. Deliberate sacrifice builds.
- **Desperation legendary** (if kept as chip): Bolt-lost = 2x breaker speed + Iron Curtain shield wall + damage pulse. The comeback kit.
- **Last Stand chip**: Speed boost on bolt-lost stacks with Iron Curtain's shield wall. You're faster AND protected.
- **Decay hazard**: Timer is ticking fast. Iron Curtain's shield gives 3s of safety, which matters more when every second counts.
- **Cascade hazard**: Damage pulse hits all cells but probably doesn't kill them. Cascade heals survivors. The pulse feeds the heal loop. Trap.

### Trap potential
The damage pulse is 1.0x base bolt damage -- weak against high-HP cells in later tiers. By tier 6+, cells have 5-10x base HP, so the pulse is negligible. Iron Curtain's defensive value (shield wall) stays useful, but its offensive value (damage pulse) falls off hard. Players who took it for the damage will be disappointed. Players who took it for the shield will do fine.

### Meta-progression unlock tier
**Early** -- immediately useful, easy to understand, safety net that doesn't feel passive because it DOES something on trigger.

---

## Protocol 10: Kickstart

### One-line description
Every node begins with 3 seconds of 2x bolt speed and 2x bolt damage.

### Detailed mechanic
New game-rule modifier. On `NodeStart`:
- All bolts get `SpeedBoost(2.0)` and `DamageBoost(2.0)` for 3.0 seconds
- Timer starts on first bump (bolt must be launched first)
- Visual: bolt is on fire for 3s. Intense glow, particle trail, screen edges pulse. When the buff expires, visual snap -- bolt dims, particles cut, brief "power down" sound.
- The contrast between powered and unpowered state is the point -- you FEEL the power leave.

### Strategy shift
Openers become everything. You plan every node around the first 3 seconds: where is the bolt going? What cells can you reach in the opening burst? Positioning at node start matters more than anywhere else. Expert players will angle their first bump to send the bolt through the densest cluster while Kickstart is active. The rest of the node is cleanup.

### Synergies
- **Piercing Shot chips**: 3s of 2x damage + piercing = devastating opening salvo through cell columns
- **Feedback Loop legendary**: If you can land a perfect bump -> cell kill in the first 3s, the speed burst stacks with Kickstart's speed. Opening becomes nuclear.
- **Singularity legendary** (if kept as chip): Tiny fast bolt + Kickstart = 2.8x speed, 5.0x damage for 3s. Absurd alpha strike.
- **Decay hazard**: Timer ticks faster, making the 3s opening window a larger percentage of your total time. Kickstart becomes proportionally more important.
- **Renewal hazard**: Cells regen on timer. If you can kill cells in the 3s Kickstart window, they don't have time to regen. Kickstart directly counters Renewal's early-node pressure.

### Trap potential
3 seconds is short. If your first bump sends the bolt into a wall instead of cells, you've wasted the entire Kickstart window. Bad for builds that rely on setup time (Convergence grouping, Magnetism positioning). Also bad with Undertow -- bolt comes back in 0.8s, so you spend 2 of your 3 Kickstart seconds bumping instead of hitting cells. Looks universally good; punishes slow starts.

### Meta-progression unlock tier
**Early** -- immediate, dramatic effect. Every player can feel the difference. Simple to use, hard to optimize.

---

## Protocol 11: Echo Strike

### One-line description
Every perfect bump records the bolt's position. On next cell impact, phantom damage replays at all recorded positions.

### Detailed mechanic
New game-rule modifier:
- On `PerfectBump`: record bolt's current position in a position buffer (max 5 positions)
- On `Impacted(Cell)`: fire phantom damage at every recorded position (damage = 50% of bolt's current damage). Clear the buffer.
- Phantom damage uses `Shockwave(base_range: 24)` at each recorded position -- small AoE, not a full-screen event
- Visual: ghost bolt markers appear at recorded positions (translucent, pulsing). On cashout, shockwave rings expand from each marker simultaneously.
- Buffer persists across bumps until a cell impact triggers cashout.
- If buffer is full (5) and another perfect bump fires, oldest position is replaced.

### Strategy shift
Perfect bumps become spatial planning. You're not just timing bumps -- you're choosing WHERE to bump perfectly, because those positions become future damage sources. Expert players will bump perfectly near cell clusters, knowing the phantom damage will hit them on the next cell impact. It's predictive geometry: "where will the bolt be when I perfect-bump, and where will those phantom hits be useful?"

### Synergies
- **Overclock / Surge chips**: Speed boosts on perfect bump keep the bolt moving fast between recorded positions. More speed = more distance = more spread in phantom placement.
- **Prism breaker**: Extra bolt on perfect bump. More bolts = more perfect bump opportunities = more positions recorded. But only ONE buffer, so extra bolts competing for the same buffer.
- **Tether chip**: Chain bolt spawns on perfect bump. Tethered bolt and primary bolt both count -- but only primary bolt position is recorded. Synergy is indirect.
- **Gravity Surge hazard**: Gravity wells from kills mess with bolt position. If your phantom positions are in gravity wells, the bolt gets pulled away from optimal bump zones. Disrupts the spatial planning.

### Trap potential
Requires consistent perfect bumps. If you can't land perfect bumps reliably, the buffer stays empty and the protocol does nothing. Also requires spatial awareness -- recording positions randomly is worthless. You need to bump perfectly near cells. This protocol separates experts from everyone else. Beginners will take it and never trigger it. Experts will use it to clear half the node with phantom geometry.

### Meta-progression unlock tier
**Late** -- extreme skill requirement. The protocol that makes speedrunners drool and beginners cry.

---

## Protocol 12: Gravity Lens

### One-line description
The bolt curves toward the densest cell cluster. Curvature strength scales with bolt speed.

### Detailed mechanic
New game-rule modifier. Continuous effect:
- Every physics tick, calculate the center-of-mass of all living cells
- Apply a steering force toward that center, proportional to: `bolt_speed * 0.15`
- At base speed (~400), this is a gentle curve. At 2x speed (Overclock, Deadline), it's a sharp pull.
- Visual: faint gravitational lens distortion around the bolt, trail curves visibly
- The force caps at a maximum curvature to prevent orbit loops (bolt can't get stuck circling)
- Force deactivates for 0.3s after a cell impact (prevents immediate re-collision with the same cluster)

### Strategy shift
Aiming becomes about speed management. Slow bolt = subtle curve, you still aim normally. Fast bolt = aggressive curve, it hones in on cell clusters. The player's control over bolt trajectory becomes: (1) bump angle sets initial direction, (2) bolt speed determines how much the gravity lens corrects. Skilled players will bump at "wrong" angles knowing the Gravity Lens will bend the bolt toward cells.

### Synergies
- **Bolt Speed chips / Surge / Overclock**: More speed = stronger gravitational pull. Speed builds become accuracy builds.
- **Deadline protocol**: Cannot be taken with Gravity Lens (one protocol per run), but this interaction validates that they occupy different niches.
- **Haste hazard**: Bolt speed increase = stronger Gravity Lens pull. Haste actually helps you. A hazard that feeds your protocol.
- **Overcharge hazard**: Per-kill speed ramp within a bump cycle. As speed increases, gravity lens strengthens, pulling bolt into more cells, generating more kills, more speed, stronger gravity... positive feedback loop that eventually makes the bolt uncontrollable. Beautiful chaos.
- **Echo Cells hazard**: Ghost cells count as living cells for center-of-mass calculation. More targets = stronger/different pull direction. Could help or hurt.

### Trap potential
Terrible on nodes where cells are spread out evenly -- center-of-mass is in the middle of nothing, so the bolt curves toward empty space. Also bad in the late game when cells are clustered in corners -- the bolt might curve away from the angle you intended toward a cluster you didn't want to hit first. Loss of precise control is the trade-off. Expert players who rely on exact bank-shot angles will hate this protocol.

### Meta-progression unlock tier
**Mid** -- the concept is clear, but understanding when it's good vs. when it fights your positioning requires experience.

---

## Protocol 13: Siphon

### One-line description
Cells killed by area-of-effect damage (shockwave, chain lightning, explode) add time to the node timer.

### Detailed mechanic
New game-rule modifier. On `CellDestroyed` (by AoE source):
- Add 0.5 seconds to the node timer per cell killed by AoE
- Only AoE-sourced kills count: Shockwave, ChainLightning, Explode, Pulse. NOT direct bolt impact, NOT bolt trail (Overburn).
- Visual: time bonus pops up as floating text near the killed cell (+0.5s), timer briefly flashes green
- No cap on time gained per node (but diminishing returns naturally as cells run out)

### Strategy shift
AoE becomes survival. Every shockwave chip, every chain lightning, every explosion is now a time extension. Build construction prioritizes AoE coverage over single-target damage. The timer stops being a threat and starts being a resource you farm. This flips the Escalation pillar -- the timer is still ticking, but you're actively fighting it with AoE kills rather than passively watching it drain.

### Synergies
- **Cascade / Shockwave chips**: Core synergy. Every shockwave kill extends the timer. Shockwave chains that kill 3-4 cells add 1.5-2s. Sustain builds become real.
- **Powder Keg legendary** (if kept as chip): Cell death -> explosion -> kills adjacent cells -> each kill adds time. Chain explosions become a timer refill.
- **Chain Reaction legendary** (if kept as chip): Recursive destruction spawning bolts -> which cause more AoE -> which kill more cells -> which add more time. Infinite scaling potential.
- **Entropy Engine evolution**: Random effects on cell destruction include shockwaves. Siphon turns Entropy Engine's chaos into timer sustain.
- **Decay hazard**: Timer ticks faster. Siphon lets you fight back. The tension between Decay's drain and Siphon's refill is excellent -- you're constantly on the edge.
- **Fracture hazard**: Fragments create more cells to kill with AoE = more time added. Hazard feeds the protocol.
- **Renewal hazard**: Cells regen, which means more cells to kill again for more time. Eternal-node scenario if the math works out.

### Trap potential
Useless without AoE. If your build is pure damage + piercing, no AoE kills happen, no time is added. The protocol is dead weight. Also, Siphon can create false security -- "I have infinite time" leads to sloppy play, more bolt-losses, and eventual death from life/time penalties rather than the timer. The timer was keeping you focused. Remove the time pressure and some players will play worse.

### Meta-progression unlock tier
**Mid** -- requires understanding which effects count as AoE and building around them. Rewards catalog knowledge.

---

## Protocol 14: Overclock Protocol

### One-line description
Perfect bumps grant a stacking speed multiplier that resets on whiff -- but each stack also increases bolt SIZE.

### Detailed mechanic
New game-rule modifier. On `PerfectBumped`:
- Add one Overclock stack to the bolt
- Each stack: `SpeedBoost(1.15)` + `SizeBoost(1.08)`
- On `BumpWhiff`: all stacks on all bolts reset to 0
- Visual: bolt grows and accelerates per stack. At 5+ stacks, it's visibly larger and trailing bright particles. At 10+ stacks, screen shake on cell impacts. Whiff = visible "deflate" with a sad trombone-style visual (bolt shrinks, particles cut).

The twist: speed makes the bolt harder to track. Size makes it easier to hit cells... but also easier to hit the breaker on return bounces (less time to reposition). And the bigger bolt has a larger collision box, which can mean hitting cells you didn't intend to hit, disrupting aiming.

### Strategy shift
This is a tower you're building. Each perfect bump adds a floor. Whiff and the whole tower collapses. The decision becomes: "do I go for another perfect bump to add a stack, or do I play safe to protect my existing stacks?" At 8+ stacks, the bolt is huge and fast -- exciting but volatile. The size increase means your aiming becomes less precise (big bolt hits everything nearby), which can be good (more cell contact) or bad (can't thread gaps).

### Synergies
- **Tempo legendary** (if kept as chip): Already does speed-on-bump, whiff resets. Overclock Protocol adds the size dimension. Together, they create an extreme risk tower.
- **Feedback Loop legendary**: Deep trigger chain fires off the perfect bump that's building stacks. Double reward per perfect bump.
- **Gauntlet legendary** (if kept as chip): Already large bolt. With Overclock Protocol's size scaling on top, the bolt becomes comically large at high stacks. Visual spectacle.
- **Erosion hazard**: Breaker shrinks, making bumps harder. Harder bumps = more likely whiff = stack collapse. Direct tension.
- **Haste hazard**: Bolt already faster from hazard. Overclock Protocol stacks more speed on top. At high stacks + Haste, the bolt is nearly invisible from speed.

### Trap potential
The size growth is a trap within a trap. At low stacks, size helps. At high stacks (8+), the bolt is so large it clips cells you didn't aim for, disrupting precision strategies. Players chasing big numbers on the stack counter will build a bolt too big to control. Also, whiff reset is brutal with Erosion -- smaller breaker means more whiffs means can never build stacks. This protocol is incredible for consistent players and worthless for inconsistent ones.

### Meta-progression unlock tier
**Late** -- the dual-scaling (speed + size) creates complex decision space. The "size is both good and bad" realization only comes with experience.

---

## Protocol 15: Tier Regression

### One-line description
Drop back 1 tier of difficulty. Gain an extra tier's worth of nodes to earn chip offerings before escalation resumes.

### Detailed mechanic
New game-rule modifier. On pickup:
- Current tier index decreases by 1 (e.g., tier 5 -> tier 4)
- The player plays through the "easier" tier's node sequence again, getting chip offerings at each node
- After completing the regressed tier, progression resumes normally from the tier they were at
- Can only appear as a protocol offering once per run
- Visual: tier counter visibly rewinds. Nodes are marked with a distinct "regression" indicator.
- In infinite mode (tier 9+): regresses to tier 8 difficulty but KEEPS existing hazard stack. You get easier cells with the same hazards.
- Tier 0 regression variant: hardcoded simple layouts, super easy. Only appears if Tier Regression is taken at tier 1.

### Strategy shift
This is the strategic retreat. You're trading forward progress for build investment. 5 extra nodes means 5 more chip offerings, which might be the difference between a mediocre build and a broken one. The decision is: "is my build strong enough to push forward, or do I need more chips to handle what's coming?" In infinite mode, it's even more interesting: you get easier cells but the same hazards. More time to farm, but the hazards still stack.

### Synergies
- **Every chip in the game**: More nodes = more chip offerings = more chances to find the chip you need. This protocol is a universal build-enabler.
- **Fission protocol** (cannot take both, one per run): If Fission needs destruction volume, regression gives more nodes to accumulate splits. But you can't take both. Good design space.
- **Bloodline protocol** (cannot take both): More nodes = more cells to kill = more damage absorbed. But you can't take both.
- **Hazard stacking (infinite mode)**: Regression doesn't remove hazards. If you've stacked Decay + Erosion, regression just means easier cells with the same brutal hazards. Not a free pass.

### Trap potential
Looks like free value. But the opportunity cost is enormous: you gave up a chip slot AND a protocol that actively changes your mechanics. The extra nodes might give you chips you don't need. If you're already strong enough to push forward, regression is wasted tempo. Also, in infinite mode, regression without hazard removal means you're still dealing with everything you've accumulated. The cells are easier but the hazards aren't. This protocol is for builders, not for fighters.

### Meta-progression unlock tier
**Late** -- understanding when to retreat vs. push forward requires deep run experience. New players will always take it (free stuff!). Experts will know when it's actually worth the protocol slot.

---

## Legendary-to-Protocol Migration Analysis

### Promote to Protocol (remove from chip pool)

| Legendary | Why Promote | Protocol Modifications |
|-----------|-------------|----------------------|
| **Deadline** | Already designed as Protocol 1. Run-defining effect that changes pacing strategy. Too powerful and too identity-altering to compete with chip slots. | Timer threshold tuning (25% is correct). Add damage boost (original only had speed). |
| **Ricochet Protocol** | Already designed as Protocol 2. Changes aiming strategy fundamentally. The wall-bank playstyle is a run identity, not just a power boost. | No changes needed. Already perfect as a protocol. |
| **Tempo** | Consecutive-bump speed ramp with whiff reset is a game-rhythm alteration, not a stat buff. It changes HOW you play every single bump. | Consider merging with Overclock Protocol (Protocol 14) design or keeping both. If keeping Tempo as protocol, it should have an additional twist beyond what Protocol 14 already does. **Recommendation**: keep Tempo as a legendary chip. Protocol 14 (Overclock Protocol) is the evolved version of this concept with the size dimension added. |

### Keep as Legendary Chips

| Legendary | Why Keep | Notes |
|-----------|----------|-------|
| **Glass Cannon** | Pure risk/reward stat trade. Changes your math, not your mechanics. Chip territory. | Good as-is. |
| **Desperation** | Triggered recovery effect. Bolt-lost response, not a strategy shift. | Good as-is. Synergizes well with Iron Curtain protocol. |
| **Whiplash** | Whiff redemption combo. Single-interaction modifier, not a run identity. | Good as-is. Synergizes well with Debt Collector protocol. |
| **Singularity** | Stat bundle (small, fast, strong). Changes feel but not mechanics. | Good as-is. |
| **Gauntlet** | Stat bundle (big, fast, weak). Changes feel but not mechanics. | Good as-is. |
| **Chain Reaction** | Recursive destruction trigger. Powerful synergy enabler but doesn't change how you play moment-to-moment. | Keep as chip. It synergizes with protocols (Fission, Siphon, Convergence) as an ingredient, not a strategy shift. |
| **Feedback Loop** | Deep trigger chain. Rewards precision sequences but doesn't alter your fundamental approach. | Keep as chip. Good synergy with Undertow and Overclock protocols. |
| **Parry** | Perfect bump -> shield + shockwave. Defensive triggered ability. | Keep as chip. Competes with Iron Curtain protocol for the "safety" build slot conceptually, but at different trigger conditions. |
| **Powder Keg** | Cell-death explosion. AoE effect, not a strategy shift. | Keep as chip. Core synergy ingredient for Siphon and Convergence protocols. |
| **Death Lightning** | Chain lightning on cell death. AoE triggered effect. | Keep as chip. Same reasoning as Powder Keg. |
| **Tempo** | See above. Keep as chip; Protocol 14 is the evolved version of this concept. | The speed-on-bump, whiff-resets mechanic is strong but doesn't need to be a protocol. Protocol 14 adds the size dimension that makes it protocol-worthy. |

### Summary

**Promote 2 legendaries to protocols**: Deadline, Ricochet Protocol
**Keep 11 legendaries as chips**: Glass Cannon, Desperation, Whiplash, Singularity, Gauntlet, Chain Reaction, Feedback Loop, Parry, Powder Keg, Death Lightning, Tempo
**13 new protocol designs**: Convergence Engine, Undertow, Overburn, Debt Collector, Bloodline, Fission, Iron Curtain, Kickstart, Echo Strike, Gravity Lens, Siphon, Overclock Protocol, Tier Regression

---

## Design Notes

### Protocol Archetypes

The 15 protocols cluster into recognizable playstyle archetypes:

| Archetype | Protocols | Chip Builds That Want These |
|-----------|-----------|----------------------------|
| **Timer Manipulator** | Deadline, Siphon, Kickstart | AoE builds, speed builds |
| **Angle Master** | Ricochet Protocol, Gravity Lens | Wall-bounce builds, speed builds |
| **Controlled Chaos** | Convergence Engine, Fission | AoE + destruction volume builds |
| **Failure Alchemist** | Debt Collector, Iron Curtain | Risk-reward builds, Aegis breaker |
| **Rhythm Player** | Undertow, Overclock Protocol | Bump-heavy builds, precision builds |
| **Scaling Engine** | Bloodline, Echo Strike | Single-bolt damage builds, precision builds |
| **Strategic** | Tier Regression | Any build that needs more chips |

### Hazard Counter-Play

Every protocol has at least one hazard that synergizes with it and at least one that punishes it:

| Protocol | Best Hazard Pairing | Worst Hazard Pairing |
|----------|--------------------|-----------------------|
| Deadline | Haste (more speed in window) | Erosion (small breaker for fast bolt) |
| Ricochet Protocol | Drift (accidental wall bounces) | Gravity Surge (disrupts angles) |
| Convergence Engine | Fracture (fragments cluster) | Momentum (non-lethal AoE grows cells) |
| Undertow | Erosion (constant bumps restore width) | Overcharge (speed resets on bump) |
| Overburn | Volatility (trail suppresses growth) | Momentum (trail ticks grow cells) |
| Debt Collector | Haste (faster bolt = more whiffs for debt) | Erosion (more whiffs = shrink spiral) |
| Bloodline | Cascade (doesn't reduce bonus) | Echo Cells (1 HP ghosts waste effort) |
| Fission | Fracture (more cells to destroy) | Echo Cells (ghosts pad counter cheaply) |
| Iron Curtain | Decay (shield window matters more) | Cascade (pulse feeds heals) |
| Kickstart | Renewal (kill before regen) | Drift (opening angle disrupted) |
| Echo Strike | Gravity Surge (disrupts bump positions) | Erosion (harder to perfect bump) |
| Gravity Lens | Haste (stronger pull) | Drift (competing forces) |
| Siphon | Fracture (more AoE targets) | Momentum (AoE non-lethal grows cells) |
| Overclock Protocol | Haste (speed + speed + size) | Erosion (whiff = collapse + shrink) |
| Tier Regression | All hazards (more time to deal with them) | Stacked hazards (easier cells don't help) |

### Breaker Preferences

| Protocol | Best Breaker | Worst Breaker | Why |
|----------|-------------|---------------|-----|
| Deadline | Chrono | Aegis | Chrono's time penalty is devastating in the danger zone, but the time-pressure identity matches |
| Undertow | Prism | Chrono | Prism spawns extra bolts on perfect bump; more bumps = more spawns. Chrono's time penalty on bolt-loss hurts when rapid bumping risks drops |
| Debt Collector | Aegis | Prism | Aegis loses a life on bolt-lost = debt stacks. Prism spawns an extra bolt = less bolt-lost = less debt |
| Bloodline | Aegis | Prism | Single-bolt focus. Prism's multi-bolt dilutes kill attribution |
| Echo Strike | Prism | Aegis | More bolts = more perfect bump opportunities. Aegis has no multi-bolt synergy |
| Iron Curtain | Aegis | Chrono | Life-loss triggers the shield. Time penalty triggers it too but Aegis's finite lives make it more dramatic |
| Tier Regression | Any | Any | Universal. Breaker choice is irrelevant to regression value |
