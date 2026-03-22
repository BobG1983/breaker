---
name: pattern_onenter_deferred_resource_chain
description: Chained OnEnter systems using commands.insert_resource() need apply_deferred between producer and consumer
type: feedback
---

When two systems are chained in OnEnter (e.g., `(generate_data, consume_data).chain()`), the first system's `commands.insert_resource(X)` is deferred and NOT visible to the second system in the same call. Bevy's `.chain()` does not automatically apply deferred commands between systems.

**Why:** This caused a runtime panic in the chip offering system spec where `generate_chip_offerings` inserted `ChipOffers` via commands and `spawn_chip_select` tried to read `Res<ChipOffers>` in the same OnEnter chain.

**How to apply:** Whenever a spec chains two OnEnter systems where the first inserts a resource and the second reads it, require one of:
1. `(system_a, apply_deferred, system_b).chain()`
2. Exclusive system access for the producer
3. Restructuring so the resource exists before both systems run
