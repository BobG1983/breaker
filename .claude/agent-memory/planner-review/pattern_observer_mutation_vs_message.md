---
name: Observer mutation vs message pattern
description: Specs referencing "follow shockwave pattern" for observers that mutate components (not write messages) cause query signature mismatch
type: feedback
---

When a new observer needs to MUTATE entity components (e.g., BoltVelocity), referencing an observer that WRITES messages (e.g., shockwave writes DamageCell) as the pattern leads to wrong query signatures. The test infrastructure (test_app, tick, trigger helpers) can follow the reference, but the actual observer query and logic must reference a different pattern.

**Why:** shockwave uses `Query<(&Transform, Option<&DamageBoost>)>` (read-only) while a velocity-mutating observer needs `Query<(&mut BoltVelocity, ...)>` (mutable). Writer-code will follow the referenced pattern literally.

**How to apply:** When reviewing specs for new observers, check whether the observer reads or writes entity data. If it writes (mutates components), ensure the impl spec specifies the exact query signature rather than just pointing to a read-only observer as the pattern.
