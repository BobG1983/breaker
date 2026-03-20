---
name: cross_system_component_preservation
description: When a spec says "do NOT touch component X" in a reset/init system, verify ALL components of that category are listed (e.g., all chip effects on bolt)
type: feedback
---

When a spec says a system should "not touch" certain components (e.g., "reset_bolt does NOT touch chip effect components"), the spec must enumerate ALL components in that category. In Phase 4b.2, the spec listed `Piercing, DamageBoost, BoltSpeedBoost, BoltSizeBoost` but missed `ChainHit` — another bolt chip effect component.

**Why:** Missing one component from the preservation list means the test won't catch if the implementation accidentally removes or overwrites it. Future chip effects added to the codebase will also be missed.

**How to apply:** When reviewing a spec that preserves a category of components, grep for ALL components in that category (e.g., all `#[derive(Component)]` types in `chips/components.rs` that are stamped on `Bolt` entities) and verify every one is listed.
