---
name: Validation Rules
description: Bevy 0.18.1 API facts relevant to test validation and known build issues
type: reference
---

## Bevy 0.18.1 API Facts
- MessageWriter uses `.write()` method, not `.send()`
- Fixed across: bump.rs, bolt_breaker_collision.rs, bolt_cell_collision.rs, bolt_lost.rs
- Camera: `hdr` field removed — use `Camera::default()` without hdr setting
- App resource access: use `app.world_mut().resource_mut::<T>()`, not `app.world_resource_mut::<T>()`
- Bundle tuple limit: max 15 elements per tuple. A 16-element spawn tuple triggers `E0277 "not a Bundle"`. Fix by nesting sub-tuples or extracting a named `#[derive(Bundle)]` struct.
