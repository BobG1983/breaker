# Protocols — Design Catalog

Protocols are the positive upgrade system — per-tier offerings that **change how you play**, not just how strong you are. Each run, 1 protocol is offered per tier on the chip-select screen; the player picks EITHER a chip OR the protocol (picking either closes the screen). Each protocol can only be picked once per run. The protocol pool grows via meta-progression across runs (like unlocking new Jokers in Balatro).

## Chip / Protocol Distinction

- **Chips are power** — buffs that don't change how you WANT to play.
- **Protocols are rule changes** — fundamentally change how you WANT to play.
- **The test**: Does this make me play differently, or just play the same way but stronger?

Chips make your build stronger. Protocols make it *different*. A build with Burnout and Reckless Dash plays nothing like a build with Siphon and Greed — the buttons are the same, but the decisions you weigh are not.

## Catalog (15)

Organized by design category. Each entry links to its canonical per-mechanic design doc.

### Rhythm / Timing

| Protocol | One-liner |
|----------|-----------|
| [Burnout](burnout.md) | Move to build heat → stop to drain it into a speed boost → mega-bump the bolt for boosted damage + shockwave. |
| [Debt Collector](debt_collector.md) | Early/Late bumps build a damage multiplier — Perfect bumps cash it out on the next cell impact. |
| [Kickstart](kickstart.md) | Each node opens with 3s of 2× bolt speed + 2× damage + Piercing. Timer starts on the first Bump. |
| [Reckless Dash](reckless_dash.md) | Risky-zone-of-dash bumps grant a damage one-shot; bolt-loss during dash doubles the penalty. |

### Positional / Targeting

| Protocol | One-liner |
|----------|-----------|
| [Anchor](anchor.md) | Stand still → become planted: 2× bump force, wider Perfect window, and Piercing. Moving unplants. |
| [Echo Strike](echo_strike.md) | Perfect Bump → next cell impact becomes an echo (max 3). Subsequent Perfect Bumps damage every echoed cell. |
| [Ricochet](ricochet.md) | Wall rebounds boost the next cell impact's damage. |

### Multi-bolt / Replication

| Protocol | One-liner |
|----------|-----------|
| [Afterimage](afterimage.md) | Dash spawns a phantom Breaker; bumping the bolt on the phantom mutates the **real** bolt into a phantom temporarily. |
| [Conductor](conductor.md) | Perfect Bump on an extra bolt makes it the primary — effects swap with the role change. |
| [Fission](fission.md) | Every Nth cell kill splits a bolt into two. |

### Reactive / Penalty

| Protocol | One-liner |
|----------|-----------|
| [Deadline](deadline.md) | A timer runs while in-node; on expiry, a penalty fires. Beat the clock. |
| [Iron Curtain](iron_curtain.md) | Bolt-lost fires a damage wave with abs-symmetric distance falloff. |
| [Siphon](siphon.md) | Escalating time-refund reward per kill streak. |

### Meta / Offering

| Protocol | One-liner |
|----------|-----------|
| [Greed](greed.md) | Skip a chip offering to boost rarity weights on the next offering. |
| [Tier Regression](tier_regression.md) | Drop back 1 tier — replay an easier tier's nodes for extra chip offerings. Keeps hazards in infinite mode. |

## Meta-Progression

The initial protocol pool (15) is what the player sees in early runs. Across runs, players unlock new protocols — by completing specific achievements, reaching tier milestones, or succeeding with specific builds. The Phase 7 target is **30 protocols**; the remaining 15 unlock via meta-progression.

Unlocks are persistent across runs (stored in the save layer) and drive long-tail replayability — "what build would Tier Regression + Echo Strike make possible?" is the kind of question a player only asks once both protocols are unlocked.

## Related

- [Chips](../chips.md) — the power upgrade system.
- [Hazards](../hazards/index.md) — the negative upgrade system (tier 9+ infinite mode).
- [Effects](../effects/index.md) — the action primitives that protocols and chips compose from.
- [Terminology: Core](../terminology/core.md) — Breaker, Bolt, Cell, Bump, Rebound.
