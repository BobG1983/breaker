---
name: copy_derive_removal_cascade
description: Removing Copy from an enum breaks all pattern-match sites that implicitly relied on Copy for by-value binding
type: feedback
---

When a spec removes `Copy` from an enum (e.g., `ChipEffect`) because a new variant contains a non-Copy type (e.g., `Box<TriggerChain>`), ALL existing pattern-match sites on references to that enum silently change behavior:

- Before (with Copy): `let Enum::Variant(val) = &expr` binds `val` as the inner type (e.g., `u32`) via implicit copy.
- After (without Copy): match ergonomics still works but `val` becomes `&u32`, not `u32`.

This causes compilation errors anywhere the bound value is used by-value (passed to a function expecting `u32`, compared with `==`, etc.).

**Why:** The spec writer sees "remove Copy, add Clone" as a one-line change. They don't trace the cascade to every consumer that pattern-matches on a reference.

**How to apply:** When a spec removes Copy from any type, grep for ALL pattern-match sites on that type (not just the file where it's defined). Count how many bind inner values by-value. Each one needs updating.
