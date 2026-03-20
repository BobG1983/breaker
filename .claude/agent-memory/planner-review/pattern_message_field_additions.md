---
name: message_field_additions
description: When specs add fields to existing Message structs, they routinely miss test construction sites in the sender's file
type: feedback
---

When a spec adds a field to an existing Message struct (e.g., `BoltHitCell { bolt: Entity }`), the spec writer reliably lists the obvious update targets (the message file, the main consumer file) but misses:

1. The **sender's test module** — `bolt_cell_collision.rs` has its own tests that construct `BoltHitCell` directly.
2. Any **intermediate test helpers** (e.g., `collect_cell_hits` pushes `msg.cell` — field-access patterns also need updating).

**Why:** The sender and receiver files are mentally distinct when writing specs. The sender's tests feel like "internal" tests but they still use the struct literal.

**How to apply:** When reviewing a spec that adds a field to a Message, grep for ALL construction sites of that struct across the entire codebase (not just files named in the spec), then verify the spec lists each one.
