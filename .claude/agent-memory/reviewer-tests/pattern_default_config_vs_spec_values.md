---
name: Default config values diverge from spec concrete values
description: writer-tests uses BoltConfig::default() (or similar) in spawn helpers, silently substituting a different concrete value than the one the spec named
type: feedback
---

When a spec names an explicit concrete value for a component (e.g., "bolt radius 5.0"), writer-tests may instead write a spawn helper that pulls the value from a config resource's Default impl (e.g., `BoltConfig::default().radius` which is 8.0). The test compiles and the geometric logic may still work, but the test no longer implements the spec's concrete scenario — it tests a different scenario.

**Why:** The writer-tests agent generalizes "use real config values" as a quality practice, but the spec's concrete values are chosen deliberately (to make the geometry unambiguous) and must be matched exactly.

**How to apply:** For every numeric constant in a spec's Given clause — positions, velocities, radii, half-extents, counts — verify the test uses that exact literal, not a value derived from a Default impl or config resource. Flag any divergence as BLOCKING if the spec behavior under test depends on that value (e.g., whether a geometric overlap condition is satisfied).
