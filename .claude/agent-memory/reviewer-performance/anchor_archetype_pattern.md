---
name: AnchorPlanted dynamic insert/remove archetype churn
description: AnchorPlanted added/removed every time breaker starts/stops moving — archetype invalidation per-tick
type: project
---

`detect_breaker_movement` removes `AnchorPlanted` when the breaker moves (horizontal velocity
above epsilon). `tick_anchor` inserts `AnchorPlanted` when the timer expires.

This means `AnchorPlanted` is inserted/removed on every plant/uproot cycle. In Bevy 0.18, this
moves the entity between archetypes, which invalidates archetype caches for any query that
includes or excludes `AnchorPlanted`.

**`tick_anchor` uses `Without<AnchorPlanted>`** — this query is already filtered correctly,
so it only runs on un-planted entities. The `Without` filter is the right approach.

**At current scale:** 1 Breaker entity. Archetype invalidation on a single entity's add/remove
is negligible. The engine handles this efficiently for small entity counts.

**Phase 3 concern:** If more entities ever get AnchorActive (not planned), or if AnchorPlanted
is checked by many systems (currently just tick_anchor via Without<>), this stays cheap.
Marker component with proper Without<> filtering is the correct Bevy pattern here.
