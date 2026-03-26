---
name: NodeTimer field names
description: NodeTimer uses 'remaining' and 'total' (not 'duration') — specs often get this wrong
type: feedback
---

NodeTimer in `breaker-game/src/run/node/resources.rs` has fields `remaining: f32` and `total: f32`.

**Why:** Specs referencing NodeTimer commonly say `duration` instead of `total`, causing compiler errors downstream.

**How to apply:** When reviewing any spec that calculates NodeTimer ratio, verify it references `timer.total` not `timer.duration`.
