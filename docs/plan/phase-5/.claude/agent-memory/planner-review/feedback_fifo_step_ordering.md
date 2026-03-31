---
name: fifo_step_ordering
description: FIFO counter specs must explicitly state the counter increment precedes despawn and spawn, and the early-return guard must precede resource initialization
type: feedback
---

Two ordering constraints that FIFO despawn specs frequently omit:

1. **Counter must be incremented BEFORE spawning.** The new entity must get a value strictly greater than all existing owned entities. If the spec describes "get counter, despawn loop, spawn, then increment" — the increment is in the wrong place. The correct order is: get counter value, increment and store immediately, then despawn, then spawn with the pre-incremented value.

2. **Early-return guard (max == 0) must precede resource get-or-insert.** Test behavior for max=0 typically asserts the counter resource is NOT created. If the resource initialization happens before the early return, the resource will be created even when nothing spawns.

**Why:** Seen in wave1c-spawn-phantom-fifo-code.md. The spec steps split counter read (step 1) from counter use (step 5) from counter increment (step 7) across the despawn loop, creating ambiguity. Wave 1b (gravity_well) had a cleaner "increment before spawn" note that wave 1c omitted. The early-return-before-resource constraint was implicit in step ordering but not stated.

**How to apply:** For any FIFO spawn-order spec, verify: (a) increment happens before the spawn call, and (b) the zero-cap early return is the very first operation before any resource access. If not stated, flag as IMPORTANT.
