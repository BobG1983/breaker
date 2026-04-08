# Protocol Candidates -- Round 3

8 protocol candidates for designer review. Each passes the rule-change test: "Does this make me WANT to do something I normally wouldn't?"

Generated against the 13 approved protocols, 8 declined protocols (with reasons), the full chip catalog, all cell modifiers, all 16 hazards, and all 3 breaker archetypes.

---

## Candidate 1: Conductor

**One-line**: When you have 2+ bolts, only the bolt you bumped most recently deals full damage. All others deal 0.25x.

**The behavior change**: You WANT to choose which bolt to bump. Normally, you catch whatever bolt is falling toward you. With Conductor, you're reading the field and deliberately letting one bolt pass to bump the other -- because the bolt you bump becomes your damage dealer while the rest are scouts. You're choosing targets by choosing which bolt to empower.

**Mechanic**: On bump, the bumped bolt gains the "Conducted" state (full damage). All other bolts lose Conducted and deal 0.25x damage. Visual: the Conducted bolt glows bright, unconducted bolts dim to a faint trail. Unconducted bolts still destroy cells (slowly) and still trigger chip effects (shockwaves, chain lightning at reduced values). Only one bolt can be Conducted at a time.

**Why it passes the test**: Without Conductor, you catch every bolt indiscriminately. With it, you're making a split-second decision every time two bolts approach: "Which one do I bump? Which one is heading toward the cluster I need to kill?" You WANT to let bolts fall past you if they're heading the wrong direction -- and that means deliberately ignoring bolts you'd normally catch. In a game that screams "CATCH EVERYTHING," this protocol says "catch the RIGHT one."

**Synergies**:
- **Fission protocol** (cannot take both, but interesting contrast): Fission wants more bolts. Conductor wants fewer, better-aimed bolts. Perfect archetype tension.
- **Prism breaker**: Extra bolts on perfect bump become tactical -- you spawn more bolts but only one deals full damage. The unconducted extras become scouts that soften the field.
- **Piercing Shot chips**: The Conducted bolt with Piercing cuts through a column at full damage. Unconducted bolts with Piercing barely scratch cells, creating a clear "primary weapon + support" dynamic.
- **Tether chip**: Chain bolt inherits parent's Conducted status -- the tethered pair moves as a unit, both at full damage. Suddenly Tether is a Conductor-specific build-around.
- **Erosion hazard**: Smaller breaker makes it harder to choose WHICH bolt to catch -- you're scrambling to catch anything, which defeats the "deliberate choice" fantasy.
- **Overcharge hazard**: The Conducted bolt accumulates Overcharge speed while unconducted bolts reset on bump change. Managing which bolt has dangerous speed becomes part of the decision.

**Trap potential**: Looks great with multi-bolt builds. But if you can't track multiple bolts AND make bump decisions, you'll default to catching the nearest bolt every time -- and the nearest bolt is rarely the one aimed at the right cluster. Also punishing when bolts are falling simultaneously and you physically can't reach the "right" one. Three bolts, two falling at once, one aimed perfectly but on the wrong side -- you catch the wrong one and your best angle goes to waste at 0.25x. Players who take this with Prism and then panic-catch everything are playing a 0.25x damage game.

---

## Candidate 2: Scavenger

**One-line**: Destroying cells with modifiers (Volatile, Sequence, Armored, Phantom, Magnetic, Survival) grants a stacking 15% damage boost for the current node. Standard cells grant nothing.

**The behavior change**: You WANT to target modified cells first. Normally, you'd clear easy standard cells to thin the field and leave the annoying modified cells for later (or let AoE handle them). With Scavenger, you're threading the bolt into the Armored cell's weak point early, timing hits on Phantom cells during their solid phase, and hunting Magnetic cells for their bonus -- because every modified cell killed makes your bolt hit harder for the rest of the node.

**Mechanic**: On destroying a modified cell, all bolts gain a stacking DamageBoost. Stack is 15% per modified cell, multiplicative. Resets between nodes. Visual: bolt intensity ramps with each modified kill -- a visible power-up trail that grows. Counter displayed near bolt HUD.

Standard cells (no modifiers) give nothing. Cells spawned by hazards (Echo Cell ghosts, Fracture debris) count as unmodified and give nothing.

**Why it passes the test**: Without Scavenger, you clear the field efficiently -- hit what's easy, avoid what's hard until you have to. With it, you WANT to engage with the hardest cells in the node first. Armored cells, which you'd normally avoid until the field is clear, become your first target. Phantom cells, which you'd normally wait out, become time-sensitive prizes. The protocol flips target priority from "easiest first" to "hardest first."

**Synergies**:
- **Piercing Shot chips**: Thread through standard cells to reach the modified cell behind them. Piercing becomes a tool for reaching high-value targets, not just clearing fodder.
- **Anchor protocol** (cannot take both): Anchor commits to position. Scavenger commits to targets. Different axis of commitment. Good separation.
- **Chrono breaker**: Time penalty on bolt-lost. Missing a risky shot at an Armored cell's weak point costs time. But the damage boost from killing it speeds up the rest of the node.
- **Fracture hazard**: Debris cells are unmodified -- they dilute the field with cells that give nothing. More standard cells between you and your modified targets.
- **Echo Cells hazard**: Ghost cells are unmodified. You're forced to clear them (they're real cells) but they give no Scavenger stacks. Dead weight.
- **Volatility hazard**: Standard cells grow HP while you ignore them to hunt modifiers. By the time you circle back, they're beefy.

**Trap potential**: Terrible on nodes with few modified cells. Tier 1-2 nodes are almost entirely standard cells -- Scavenger gives nothing. Also terrible with Fracture and Echo Cells, which flood the field with zero-value targets. Looks universally good until you realize modifier density varies wildly by tier and hazard loadout. On a Fracture-heavy run, you're playing with a dead protocol.

---

## Candidate 3: Triage

**One-line**: The cell with the lowest remaining HP in the grid takes 3x damage from all sources. The cell with the highest remaining HP is immune to damage.

**The behavior change**: You WANT to soften multiple cells evenly, then burst them down one at a time. Normally, you focus fire on one cell until it dies. With Triage, focus fire is punished on full-HP cells (the highest is immune) and rewarded on damaged cells (the lowest takes 3x). You WANT to spread damage across the field first, then pick off the weakest cells in rapid succession -- each kill shifts the 3x bonus to the next-lowest cell, creating a cascade of focused kills.

**Mechanic**: Every frame, the system identifies the cell with the lowest HP (non-zero) and the cell with the highest HP. The lowest-HP cell has a visible glow and takes 3x damage from all sources (bolt, shockwave, chain lightning, explosion). The highest-HP cell has a visible shield effect and is completely immune to all damage. If only one cell remains, it is both lowest AND highest -- immune overrides (you can't damage the last cell directly). When only one cell remains, the immune flag lifts after 2 seconds (brief panic window).

Ties: if multiple cells share lowest HP, all of them take 3x. If multiple cells share highest HP, all are immune.

**Why it passes the test**: Without Triage, you aim for the same cell repeatedly until it dies. With it, you WANT to spread damage as widely as possible to create a chain of "lowest HP" targets. AoE becomes a setup tool, not a kill tool -- you spread chip damage across the field to create a cascade target list. You're playing a completely different pattern: spread, then execute.

**Synergies**:
- **Cascade / Shockwave chips**: AoE spreads damage across multiple cells, setting up a cascade of 3x targets. Cascade doesn't kill (it softens), and Triage rewards softening. Perfect synergy.
- **Chain Lightning (Voltchain evolution)**: Arc damage hits multiple cells, spreading damage evenly. With Triage, this becomes a setup tool that primes the cascade.
- **Siphon protocol** (cannot take both): Siphon rewards kill streaks. Triage creates kill streaks (lowest-HP cells die in rapid sequence once the cascade starts). Would be amazing together, which is why they're mutually exclusive.
- **Momentum hazard**: Non-lethal hits give cells HP. Triage's spread-damage phase involves a LOT of non-lethal hits. Momentum punishes exactly the play pattern Triage demands. Devastating trap synergy.
- **Diffusion hazard**: Damage shared to adjacent cells -- could be helpful (spreads damage wider) or harmful (feeds Momentum on neighbors). Complex interaction that rewards player knowledge.
- **Renewal hazard**: Cells regen to full on their timer. A cell you softened to "lowest HP" status can regen back to full, breaking your cascade setup. Forces faster execution.

**Trap potential**: The "highest HP is immune" rule is brutal. On nodes with one massive boss cell surrounded by chaff, the boss is permanently immune until every other cell is dead -- and then you get a 2-second panic window before you can even damage it. Also devastating with Momentum hazard: your spread-damage phase grows cells faster than you kill them. Players who don't understand the "soften then execute" pattern will bounce between immune cells and accomplish nothing.

---

## Candidate 4: Gauntlet Run

**One-line**: At the start of each node, a random column of cells is highlighted. Destroying all cells in that column within 5 seconds triggers a Purge -- all remaining cells in the field take massive damage (50% of their current HP). Failing the column timer does nothing.

**The behavior change**: You WANT to aim for a specific column at the start of every node instead of clearing the field generally. Normally, you take whatever angle the bolt gives you. With Gauntlet Run, the first 5 seconds of every node are a frantic race to clear one specific column. Your bump angles, dash positioning, everything orients toward that column. After the Purge (or failed attempt), you play normally -- but the next node, another column lights up.

**Mechanic**: On node start, one column (chosen by seed) highlights with a visible marker and a 5-second countdown. If all cells in that column reach 0 HP within the countdown, a Purge triggers: expanding shockwave from the column center dealing 50% of each cell's current HP as damage (not lethal -- softens the field dramatically). If the timer expires, the highlight fades and nothing happens. No penalty for failure, just missed opportunity.

**Why it passes the test**: Without Gauntlet Run, you clear the field however the bolt angle suggests. With it, you have a 5-second objective that overrides your normal play pattern. You WANT to thread the bolt directly into a specific column, using Piercing and precise angles to clear it fast. Every node starts with a puzzle: "How do I clear that column in 5 seconds from this bolt angle?" The optional nature is key -- you CAN ignore it, but you lose enormous value. The 5-second window creates the same "sprint then breathe" rhythm as Kickstart, but skill-gated (you have to AIM, not just exist).

**Synergies**:
- **Piercing Shot chips**: Core synergy. Piercing lets you clear a column in one or two passes. Without Piercing, clearing a column in 5 seconds requires multiple precise bumps.
- **Kickstart protocol** (cannot take both): Both are about node starts. Kickstart is raw power. Gauntlet Run is targeted precision. Good archetype separation.
- **Bolt Speed chips**: Faster bolt means more bounces in the 5-second window. More chances to thread the column.
- **Armored cells in the target column**: Armored cells face AWAY from the bolt's natural approach. You need wall bounces or piercing to get behind them in 5 seconds. This is where Ricochet Protocol would shine -- if you could take both (you can't).
- **Decay hazard**: Node timer ticks faster. The 5-second Gauntlet window doesn't change, but you have less time AFTER the window to clear the remaining field. Makes the Purge even more valuable -- you NEED the 50% HP shred to compensate for the shorter timer.
- **Fracture hazard**: Cleared column cells might fracture into adjacent cells -- but the column is already clear, so fractures appear in the column gaps. The Purge then damages the fractures. Interaction is neutral-to-positive.

**Trap potential**: Useless on nodes where the highlighted column is mostly empty (only 1-2 cells). The Purge triggers but barely matters because you cleared 2 cells and the rest of the field is untouched at 50% HP. Also terrible if you can't reliably aim -- failing the column timer every node means you're playing with a dead protocol. The 5-second window is generous for experts and impossible for beginners who can't thread bolt angles.

---

## Candidate 5: Afterimage

**One-line**: Every dash leaves a phantom breaker at your starting position for 2 seconds. Bolts that hit a phantom breaker are bumped as a Perfect Bump with the angle you dashed FROM.

**The behavior change**: You WANT to dash AWAY from where you'll bump the bolt. Normally, you dash TO where the bolt is going. With Afterimage, you dash FROM where the bolt will arrive -- because the phantom you leave behind performs a free Perfect Bump at your old position. You're placing phantom bumpers by dashing, turning every dash into two plays: repositioning yourself AND leaving a perfect-bump trap at your origin.

**Mechanic**: On dash start, spawn a translucent copy of the breaker at the pre-dash position. Duration: 2 seconds. The phantom has the same width as the breaker at spawn time. If a bolt contacts the phantom, it receives a Perfect Bump with the tilt angle the breaker had when the dash started. The phantom is consumed on first bump (one bolt per phantom). The real breaker can still bump normally. Max 1 phantom active (dashing while a phantom exists replaces it).

**Why it passes the test**: Without Afterimage, dash is pure repositioning -- get from A to B. With it, dash is a setup tool. You WANT to dash from a position where the bolt is about to arrive, because you leave a perfect-bump phantom there. The decision changes from "where do I need to be?" to "where do I need to leave a phantom, and where do I need to end up?" You're thinking two positions ahead: phantom position (where bolt is) and real position (where bolt will be next). Double the spatial planning. You also WANT to dash more often, because every dash is a potential phantom bump.

**Synergies**:
- **Breaker Speed chips**: Faster dash means more distance between phantom and real position. The phantom covers one area, you cover another. Wider field control.
- **Reckless Dash protocol** (cannot take both): Both make dash central, but Reckless Dash wants you dashing DURING impacts while Afterimage wants you dashing BEFORE impacts. Different timing axis.
- **Burnout protocol** (cannot take both): Burnout rewards standing still. Afterimage rewards dashing constantly. Direct opposition. Good archetype separation.
- **Wide Breaker / Augment chips**: Wider breaker = wider phantom = more forgiving phantom bump zone. The phantom inherits your width at dash time.
- **Erosion hazard**: Breaker shrinks. Phantom inherits the shrunken width. Smaller phantom = harder phantom bumps. The hazard degrades both your real AND phantom breaker.
- **Haste hazard**: Faster bolt gives you less time to set up phantom positions. The bolt arrives before your phantom is placed, or the phantom expires before the bolt reaches it.

**Trap potential**: Looks like free Perfect Bumps. But the phantom lasts only 2 seconds and only bumps one bolt. If you misread the bolt's trajectory, your phantom sits unused and you've dashed to a position you didn't need to be. Also, dashing away from the bolt's path means your real breaker isn't positioned to catch it if the phantom misses. One missed phantom = dash on cooldown + breaker out of position + bolt heading for the floor. The risk/reward is razor-thin: perfect phantom placement is devastating, bad phantom placement costs you a life.

---

## Candidate 6: Harvest

**One-line**: Destroying a cell within 1 second of it being damaged (first hit to kill) drops a pickup that grants a 3-second timed buff (random from a small pool). Cells that take longer than 1 second to kill drop nothing.

**The behavior change**: You WANT to one-shot cells. Normally, you don't care how long a cell takes to die -- damage is damage. With Harvest, speed of kill matters. You WANT to hit cells hard enough to destroy them in one impact (or within 1 second of first contact via chip AoE). This changes targeting: you skip high-HP cells you can't one-shot and hunt for cells you CAN kill instantly. It also changes build priorities: raw damage matters more than coverage because you need lethal hits, not chip damage.

**Mechanic**: When a cell is destroyed, check if time since first damage <= 1.0 second. If yes, spawn a pickup entity at the cell's position that drifts downward slowly (like a coin in an arcade game). If the bolt passes through the pickup, a random 3-second buff applies to that bolt: SpeedBoost(1.5x), DamageBoost(2.0x), Piercing(3), or SizeBoost(1.5x). Pickup despawns after 4 seconds if not collected by a bolt. Max 3 pickups on screen at once (oldest despawns if exceeded).

**Why it passes the test**: Without Harvest, you clear cells at whatever pace your damage allows. With it, you're planning lethal thresholds. "Can my bolt one-shot that cell? If not, can my shockwave finish it within 1 second of the bolt impact?" Every cell becomes a time-to-kill calculation. You WANT to prioritize cells you can kill fast and skip cells you can't -- which inverts the normal "clear the field evenly" approach. You also WANT to route the bolt through pickups after kills, adding a collection mini-game to every node.

**Synergies**:
- **Damage Boost chips**: Higher damage = more cells in one-shot range. Harvest directly rewards damage investment.
- **Greed protocol** (cannot take both): Both reward damage stacking. Greed gives permanent damage by skipping chips. Harvest gives timed buffs by one-shotting. Similar damage philosophy, different axis.
- **Cascade / Shockwave chips**: AoE damage after a kill can one-shot adjacent cells within the 1-second window, spawning more pickups. Chain kills = chain pickups.
- **Singularity chip (Rare)**: Tiny, fast, high-damage bolt. Perfect for one-shotting cells. The Harvest pickup collection is harder with a small bolt, though.
- **Volatility hazard**: Cells gain HP over time. Cells you didn't one-shot immediately start growing. The window to one-shot them shrinks as the node progresses. Forces aggressive early play.
- **Cascade hazard** (the hazard, not the chip): Destroyed cells heal neighbors. You one-shot a cell, the neighbors heal, now THEY'RE harder to one-shot. The heal actively fights your lethal threshold.

**Trap potential**: Useless against high-HP cells in later tiers. When cells have 500 HP and your bolt does 200 damage, nothing gets one-shot. The protocol goes dead as difficulty scales unless you're stacking massive damage. Also punishing with Fracture hazard -- debris cells are easy to one-shot (low HP), flooding you with pickups, but the debris blocks your bolt from reaching the real targets. Looks great on paper until tier 5+ when nothing dies in one hit without extreme investment.

---

## Candidate 7: Siege

**One-line**: Your breaker emits a forward-facing beam that deals low, constant damage to the nearest cell in its line of sight. Beam damage scales with how long you hold position (1x at 0s, 3x at 3s). Moving resets the scale timer.

**The behavior change**: You WANT to aim your breaker at specific cells and hold position. Normally, the breaker is purely reactive -- you chase the bolt. With Siege, your breaker is a weapon. You're actively aiming at cells and staying still to ramp up beam damage. The bolt is still your primary damage source, but between bumps, you're not just waiting -- you're pointing at the highest-value cell and burning it down. Every moment of "dead time" between bumps becomes productive aiming time.

**Mechanic**: Breaker emits a visible beam upward (narrow, extends to first cell in line of sight). Base damage: low (roughly 5% of a standard bump's damage per second). Damage multiplier ramps from 1x to 3x over 3 seconds of holding position. ANY movement (including dash) resets the ramp to 1x. Beam always targets the nearest cell in the breaker's forward arc. Beam has no width -- it's a precise line from breaker center upward.

**Why it passes the test**: Without Siege, the breaker does nothing between bumps. With it, you're actively aiming between bumps, choosing which cell to whittle down, and balancing "stay still to ramp damage" versus "move to catch the bolt." Every moment is a decision: beam or bolt. You WANT to stand still and aim, but the bolt doesn't wait. The tension between aiming your breaker-as-weapon and chasing the bolt-as-primary-weapon creates a completely new movement pattern.

**Synergies**:
- **Anchor protocol** (cannot take both): Both reward stillness, but Anchor gives better bumps while Siege gives beam damage. Different payoffs for the same positional commitment.
- **Burnout protocol** (cannot take both): Burnout alternates stillness and movement. Siege rewards sustained stillness. Direct archetype competition.
- **Wide Breaker / Augment chips**: Wider breaker doesn't affect beam targeting (beam is center-line) but makes catching bolts while aiming easier. Safety net for the "aim vs catch" tension.
- **Amp chip**: Ramping damage on the beam adds to Amp's ramping damage stacks. Double ramp while standing still.
- **Phantom cells**: Beam can target Phantom cells during their solid phase -- but the 3-second ramp conflicts with the ~3-second phase cycle. You ramp to 3x just as the cell goes ghost. Timing puzzle within a timing puzzle.
- **Erosion hazard**: Smaller breaker doesn't affect beam damage but makes bolt catching harder. The "stay still to beam" versus "move to catch" tension becomes desperate.
- **Decay hazard**: Timer ticking faster. Every second spent beaming is a second not spent clearing via bolt. The beam must be productive enough to justify the time investment.

**Trap potential**: The beam does low damage. Even at 3x ramp, it's supplementary -- not primary. Players who focus on beaming and neglect bolt-catching will clear nodes much slower than players who treat the beam as opportunistic bonus damage. Also terrible with Haste hazard: faster bolt means less time between bumps, which means less time to ramp the beam. At high Haste stacks, you never hold position long enough to reach 2x, let alone 3x. Looks like a powerful second weapon; actually a trap that tempts you to play passive.

---

## Candidate 8: Flashpoint

**One-line**: When you dash through a bolt (bolt passes through breaker during dash), the bolt teleports to a random cell and deals 5x damage on impact. You lose dash cooldown for 3 seconds.

**The behavior change**: You WANT to dash INTO the bolt's path, not away from it. Normally, you'd never dash into a bolt -- you dash to get ahead of it. With Flashpoint, intercepting the bolt mid-flight with your dash is the highest-damage play in the game. You're reading bolt trajectory, timing your dash to intersect it, and sending it into a random cell like a bolt of lightning. The 3-second cooldown loss means you can't dash again immediately -- you're committed to wherever you land.

**Mechanic**: During dash (active dash frames only), if the bolt passes through the breaker hitbox, the bolt teleports to a random cell's position and deals 5x damage on the first impact. After teleport, the bolt resumes normal physics (bounces off the cell normally). Your dash cooldown is set to 3 seconds (overrides normal cooldown). If you have multiple bolts, only the first bolt intercepted per dash triggers Flashpoint. Visual: bolt vanishes in a flash, reappears at the cell with a lightning-strike effect. Massive screen shake. The 5x damage applies only to the first cell impacted after teleport.

**Why it passes the test**: Without Flashpoint, you dodge the bolt or get ahead of it. Dashing through a bolt is a mistake -- it means you timed your movement wrong. With Flashpoint, dashing through the bolt is the single highest-damage action in the game. You WANT to intercept the bolt mid-flight, which means you're reading its trajectory and timing dashes to cross its path. The randomized target cell means you can't aim the teleport -- it's a gamble on which cell gets hit. But 5x damage to ANY cell is almost always worth it. The 3-second dash cooldown after use creates a vulnerability window where you can't reposition with dash.

**Synergies**:
- **Reckless Dash protocol** (cannot take both): Both make dash an offensive tool. Reckless Dash rewards dashing during impacts. Flashpoint rewards dashing through the bolt itself. Different timing targets.
- **Breaker Speed chips**: Faster movement helps you reach the bolt's path in time for the intercept. Also helps you recover position after the 3-second dash lockout.
- **Damage Boost chips**: 5x base damage, multiplicative with damage boosts. A bolt with 2x DamageBoost becomes 10x on Flashpoint. Devastating alpha strikes.
- **Aegis breaker**: Lives give you safety margin for the 3-second dashless window. If you lose the bolt while dash is locked, lives absorb the hit.
- **Drift hazard**: Wind changes bolt trajectory, making intercepts harder to time. The bolt goes where you don't expect, and you dash through empty space.
- **Haste hazard**: Faster bolt is harder to intercept. The dash window for intersection narrows. Mechanical ceiling rises dramatically.
- **Overcharge hazard**: Bolt speeds up per kill. Flashpoint kills make Overcharge more dangerous by pushing the bolt faster immediately after the teleport impact.

**Trap potential**: Random target cell means you can't aim. You might Flashpoint into a 1-HP cell that would have died anyway, wasting 5x damage. Also, the 3-second dash lockout is brutal -- one bad Flashpoint leaves you unable to dash with a bolt bouncing unpredictably after teleport. If the bolt teleports to a corner cell and ricochets erratically, you're stuck without dash trying to track it. The intercept timing is extremely tight with fast bolts -- attempting Flashpoint against a Haste-boosted bolt and missing means you've dashed through nothing and are out of position with a live bolt heading for the floor.

---

## Summary Table

| # | Name | Archetype | Behavior Change | Gap Filled |
|---|------|-----------|----------------|------------|
| 1 | Conductor | Bolt Selector | Choose WHICH bolt to bump | Multi-bolt behavior |
| 2 | Scavenger | Modifier Hunter | Target modified cells first | Cell interaction |
| 3 | Triage | Damage Spreader | Soften many, then cascade kills | Cell interaction / AoE planning |
| 4 | Gauntlet Run | Column Racer | Sprint-clear a target column at node start | Breaker positioning / precision aim |
| 5 | Afterimage | Phantom Placer | Dash FROM bolt, not TO bolt | Breaker positioning / dash rethink |
| 6 | Harvest | One-Shot Hunter | One-shot cells for timed pickups | Damage threshold targeting |
| 7 | Siege | Beam Aimer | Aim breaker between bumps | Breaker-as-weapon / dead time elimination |
| 8 | Flashpoint | Bolt Interceptor | Dash INTO the bolt's path | Dash-as-weapon / interception |

### Archetype Coverage vs Existing Protocols

| Gap (from brief) | Candidates that fill it |
|-------------------|------------------------|
| Multi-bolt behavior | Conductor (bolt selection) |
| Cell interaction | Scavenger (modifier targeting), Triage (HP-based targeting) |
| Breaker positioning | Gauntlet Run (column aim), Afterimage (phantom placement), Siege (beam aim) |

### Mutual Exclusion Notes

Protocols are one-per-run, so mutual exclusion is inherent. But some candidates are explicitly designed as archetype rivals:

- Conductor vs Fission: bolt quality vs bolt quantity
- Scavenger vs Greed: engage with modifiers vs skip chips for damage
- Siege vs Anchor vs Burnout: all reward stillness, different payoffs (beam damage vs better bumps vs heat cycle)
- Afterimage vs Reckless Dash: dash-as-setup vs dash-as-risk
- Flashpoint vs Reckless Dash: dash-through-bolt vs dash-during-impact
- Gauntlet Run vs Kickstart: targeted node opener vs raw power opener

### Declined During Generation (with reasons)

These ideas were considered and killed before reaching candidate status:

| Idea | Why Killed |
|------|-----------|
| "Bolt types" (fire/ice/lightning bolt) | Bolt doesn't have intrinsic types in the design. Would require a new system with no existing hooks. Also too close to generic RPG elements. |
| "Combo counter" (hit X cells without bumping for bonus) | You already want to hit cells without bumping -- this is just power for normal play. Fails the test. |
| "Cell bounty" (random cell marked, bonus for killing it) | Too similar to Gauntlet Run but worse -- one cell instead of a column, less interesting geometry. |
| "Mirror bolt" (bolt splits into a mirrored copy on bump) | Sounds like a rule change but you don't DO anything differently. You bump normally and get a free bolt. That's power (Fission already covers the "more bolts" space better). |
| "Overload" (take damage to power up bolt) | No player health resource to spend. Would need a new system. Also too close to Debt Collector (sacrifice something to gain damage). |
