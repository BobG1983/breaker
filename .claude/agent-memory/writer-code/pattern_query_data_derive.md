---
name: QueryData derive pattern
description: How to use #[derive(QueryData)] named structs in Bevy 0.18 — field access, Item types, lifetime params, and DerefMut through Mut<T>
type: feedback
---

When replacing tuple query type aliases with `#[derive(QueryData)]` named structs in Bevy 0.18:

**Why:** Named structs replace positional tuple destructuring with named field access, improving readability. The orchestrator chose this for the bolt domain queries refactor.

**How to apply:**

1. **Item types have two lifetime params**: `FooItem<'w, 's>` even for read-only QueryData. Always use `<'_, '_>`.

2. **Mutable fields are `Mut<'w, T>`**: Access via DerefMut. `bolt.position.0 = x` writes through Mut. `&mut bolt.position` coerces to `&mut Position2D` via DerefMut coercion.

3. **For secondary lookups (bolt_vel_params pattern)**: Keep a separate read-only QueryData struct with Optional fields if test entities don't spawn all required components. The `BoltVelocityParams` struct serves this role.

4. **`as_deref()` works on `Option<Mut<T>>`**: Since `Mut<T>: Deref<Target=T>`, `Option<Mut<T>>.as_deref()` returns `Option<&T>`.

5. **Partial moves from item struct**: You can move `Option<Mut<T>>` fields out of the item for pattern matching (e.g., `if let (Some(ref mut x), Some(y)) = (bolt.field_a, bolt.field_b)`), but only if the item isn't used afterward.

6. **`apply_velocity_formula_collision(&mut bolt)` borrows whole item**: Extract any needed field values before this call, or make this the last access.
