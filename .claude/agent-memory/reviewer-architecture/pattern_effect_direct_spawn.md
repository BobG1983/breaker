---
name: Effect domain direct entity spawning pattern
description: Effect fire() functions spawn full game entities (bolts, walls) directly via World access rather than using messages to owning domains
type: project
---

Effect `fire()` functions that need to create new game entities (chain bolts, phantom bolts, second wind walls) do so by directly spawning entities with `&mut World` rather than sending messages to the owning domain.

**Why:** The `fire()` function has `&mut World` access by design (see `docs/architecture/effects/commands.md`). Direct spawning is simpler than adding message round-trips. Established by `chain_bolt.rs`, `spawn_phantom.rs`, `second_wind.rs`, `gravity_well.rs`, and `shockwave.rs`.

**How to apply:** When reviewing new effects that spawn entities, accept direct spawning in `fire()` as the established pattern. However, flag it if the spawned entity is missing a cleanup marker (`CleanupOnNodeExit` or equivalent via `#[require]`). Note: `SpawnChainBolt` was removed from both code and docs (`docs/architecture/messages.md`) — this doc/code inconsistency is resolved.
