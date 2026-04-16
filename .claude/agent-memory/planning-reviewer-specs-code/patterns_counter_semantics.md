---
name: Counter semantics pre vs post increment
description: Specs describing monotonic counters often get pre/post increment wrong, causing first-value off-by-one vs test behaviors
type: feedback
---

When a spec says "get counter (default 0), increment, store back, use as spawn order" — the new entity gets the incremented value (1 on first call), not 0. But test specs for FIFO ordering typically expect the first entity to have order 0.

**Why:** The natural English description "increment then use" implies the new entity gets the post-increment value. But the intended semantics are usually "use then increment" (post-increment in C terms). This caused a BLOCKING mismatch between the gravity_well FIFO code spec and its test spec.

**How to apply:** When reviewing any spec that describes a monotonic counter assigned to spawned entities, check: what value does the first spawn get? Cross-reference with the test spec's concrete expected values (e.g., "GravityWellSpawnOrder(0)" for the first well). If the step description says "increment first", flag it unless the starting value is -1 or equivalent. The spec should read: "read current value → assign to entity → increment stored value".
