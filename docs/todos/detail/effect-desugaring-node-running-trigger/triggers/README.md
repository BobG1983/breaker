# Per-Trigger Behavior Specs

One file per trigger (or trigger group). Each file defines the bridge system that fires it, what messages it reads, locality, participants, and dispatch behavior.

## Trigger Categories

### Bump triggers
- [bump_local.md](bump_local.md) — PerfectBumped, EarlyBumped, LateBumped, Bumped (local, both participants)
- [bump_global.md](bump_global.md) — PerfectBumpOccurred, EarlyBumpOccurred, LateBumpOccurred, BumpOccurred, BumpWhiffOccurred, NoBumpOccurred (global)

### Impact triggers
- [impact_local.md](impact_local.md) — Impacted(ImpactTarget) (local, both participants)
- [impact_global.md](impact_global.md) — ImpactOccurred(ImpactTarget) (global)

### Death triggers
- [death.md](death.md) — Died (local, victim), Killed(KillTarget) (local, killer), DeathOccurred(DeathTarget) (global)

### Other triggers
- [bolt_lost.md](bolt_lost.md) — BoltLostOccurred (global)
- [node_lifecycle.md](node_lifecycle.md) — NodeStartOccurred, NodeEndOccurred, NodeTimerThresholdOccurred (global)
- [spawned.md](spawned.md) — Spawned(EntityType) (bridge, PostFixedUpdate)
- [time_expires.md](time_expires.md) — TimeExpires(f32) (self-consuming timer)

### Conditions (for During)
- [conditions.md](conditions.md) — NodeActive, ShieldActive, ComboActive(u32)
