---
name: Counter resource state assertions missing from ordering specs
description: Specs that test FIFO/ordering counters often omit post-call resource state verification, weakening the RED test
type: feedback
---

When a spec tests a monotonic counter (e.g., PhantomSpawnOrder, GravityWellSpawnOrder), behavior 1 should verify BOTH:
- The spawned entity got the correct component value (e.g., order 0)
- The counter resource now holds the NEXT value (e.g., 1)

Without the second assertion, a broken implementation that stamps every entity with order 0 without incrementing passes behavior 1 and only fails at behavior 2.

**Why:** The counter is the mechanism under test. Verifying only the component value tests the output but not the state machine. The 1b gravity_well spec correctly verifies both; the 1c spawn_phantom spec omits the resource state check.

**How to apply:** In any spec that introduces a counter/ordering resource, require behavior 1 to verify the post-call resource state in its Then clause, not just the component on the spawned entity.
