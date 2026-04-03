---
name: WallRegistry data pattern
description: WallRegistry is a HashMap<String, WallDefinition> Resource loaded once at startup — no hot-path cost
type: project
---

WallRegistry wraps `HashMap<String, WallDefinition>` and implements `SeedableRegistry`.

Confirmed acceptable patterns:
- `seed()` clones both `def.name` (String) and `def` (WallDefinition including `Vec<RootEffect>`) per entry — this is startup-only, not per-frame
- `update_single()` similarly clones both name and definition — only called on hot-reload asset changes, not per frame in production
- HashMap has no pre-sized capacity; at 4 walls this is negligible (HashMap default capacity is 0, grows to handle the load)
- No systems, queries, or archetypes added in this wave — zero fragmentation risk
- `WallDefinition` stores `Vec<RootEffect>` — heap allocation per definition, but these are owned by the registry (a Resource), not attached to entities

**Why:** Walls is a fixed set of 4 entities. The registry is populated once at startup via `SeedableRegistry::seed()` and only touched again on hot-reload. No per-frame performance concern exists.

**How to apply:** Do not flag seed/update_single clone costs as hot-path issues for this registry. Only raise if wall count grows to hundreds or if seed() is called per-frame.
