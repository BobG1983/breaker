# Phase 8: Roguelite Progression

**Goal**: The meta-layer that keeps players coming back. Permanent progression that expands the situation space, not just makes it easier.

## Run Modifiers (Ascension System)

Inspired by Slay the Spire's Ascension, Hades' Pact of Punishment, and Balatro's Stakes:

- **Selectable difficulty modifiers**: Each modifier changes the rules (shorter timers, more active nodes, restricted chip pool, etc.)
- **Modifiers are stackable**: Players can activate multiple modifiers for compounding challenge + reward
- **Reward multiplier**: Higher modifier stacks = more Flux earned per run
- **Modifiers as situation generators**: Each modifier combination creates a different meta-situation. "No common chips" forces different build paths. "Double boss HP" changes which evolutions matter. The modifier system multiplies run variety at the meta level.

## Meta-Progression

Progression must **expand the possibility space**, not just make the player stronger:

- **Unlockable chips added to the pool**: New chips earned through Flux spending or achievement. Each unlock increases the combinatorial space — the game literally gets more varied as you progress.
- **Unlockable breakers**: New archetypes with different physics and abilities. Each breaker multiplies the effective chip pool (same chips play differently on different breakers).
- **Permanent stat buffs**: Subtle, don't trivialize core. Small starting bonuses that reduce early-run friction without removing challenge.
- **Unlockable evolution recipes**: Some evolutions only enter the recipe book after being discovered or purchased. Knowledge-gated depth.

### What NOT to Unlock

Avoid progression that *reduces* variety:
- No "skip early nodes" — early nodes are where builds start. Skipping them removes decisions.
- No "start with chips" — the first few offerings define the run's identity. Pre-loaded chips homogenize early game.
- No "guaranteed rare offerings" — rarity is the scarcity that makes rare chips feel special.

## Alternate Breakers (3+ new archetypes)

Each breaker should feel like a different game:

- **Precision**: Narrow breaker, faster movement, tighter bump window, higher perfect multiplier. High skill floor, highest ceiling.
- **Berserker**: Wide breaker, slower movement, bump always triggers shockwave (small), no dash. Aggressive, forward-momentum playstyle.
- **Momentum**: Normal breaker, but bolt speed increases with consecutive hits (resets on bolt-lost). Rewards streaks, punishes mistakes exponentially.

Each archetype creates a different relationship with the chip pool — chips that are mediocre on Aegis might be build-defining on Momentum.

## Run Structure Expansion

- **Shop nodes**: Spend in-run currency to buy specific chips (agency within randomness)
- **Rest nodes**: Heal, remove a chip from inventory (deck thinning), or upgrade a chip's max stacks
- **Seed-based leaderboards**: Compare runs on the same seed. Same situation, different execution.
- **Daily challenge seeds**: One seed per day, everyone plays it, compare results. Community engagement driver.

## Save System

- Persist meta-progression (unlocks, Flux balance, modifier records) between sessions
- Run history: browseable log of past runs with seeds, builds, highlights, and outcomes
- Personal bests per archetype

## Acceptance Criteria

1. Flux is spendable on unlocks
2. At least 3 new breaker archetypes with distinct physics
3. Run modifier system with at least 5 stackable modifiers
4. Unlocking new chips visibly expands offering variety in subsequent runs
5. Daily challenge seed system functional
6. Run history with seed replay
