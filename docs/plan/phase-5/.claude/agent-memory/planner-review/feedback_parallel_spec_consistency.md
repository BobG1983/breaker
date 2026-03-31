---
name: parallel_spec_consistency
description: When two specs fix the same pattern in parallel waves, check both for inconsistent struct definitions and API call forms
type: feedback
---

When two features in the same wave apply the identical pattern (e.g., wave1b and wave1c both add FIFO spawn ordering to different effects), the implementation specs must be internally consistent with each other:

- **Struct field naming**: If wave 1b defines `GravityWellSpawnCounter { counters: HashMap<Entity, u64> }` (named field), wave 1c must not define `PhantomSpawnCounter(HashMap<Entity, u64>)` (newtype) without explicit justification.
- **API call forms**: If wave 1b uses `world.get_resource_or_insert_with(Type::default)` (function pointer), wave 1c must not use `world.get_resource_or_insert_with(|| Type::default())` (closure). Mixed forms compile but indicate the writer is not following the established pattern.
- **Step narrative structure**: If wave 1b's fire() steps follow a particular order, wave 1c should use the same narrative order for the same pattern.

**Why:** Seen in wave1c spec having two inconsistent forms of `get_resource_or_insert_with` (lines 16 and 28 of the code spec), and lacking an explicit struct body definition that wave 1b had.

**How to apply:** When reviewing a spec that is explicitly a parallel of another spec (referenced in Patterns to Follow), read both specs and spot inconsistencies in struct layout and API forms. Flag any divergence without justification as IMPORTANT.
