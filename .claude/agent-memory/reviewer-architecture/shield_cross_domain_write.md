---
name: ShieldActive cross-domain write exception
description: bolt and cells domains are authorized to directly mutate ShieldActive (effect domain component) per plugins.md exception
type: project
---

`ShieldActive` (effect domain component, defined in `effect/effects/shield.rs`) is written by two non-effect domains as a documented architectural exception in `docs/architecture/plugins.md`:

- **bolt** (`bolt_lost` system): reads/writes ShieldActive on the breaker entity to absorb bolt losses
- **cells** (`handle_cell_hit` system): reads/writes ShieldActive on cell entities to absorb damage hits

Both systems decrement `charges` directly and use `commands.entity(...).remove::<ShieldActive>()` when charges reach zero.

**Why:** Damage absorption must short-circuit within the same frame as the hit — a message round-trip would be too late.

**How to apply:** If a new domain needs to write ShieldActive, it must be added to the exception list in plugins.md. Do not silently extend this exception.
