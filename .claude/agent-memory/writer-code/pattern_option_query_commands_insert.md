---
name: Option query + Commands insert pattern
description: Pattern for optional mutable component in Bevy query - mutate in place if Some, commands.insert if None
type: feedback
---

When a component may not exist on an entity yet but should be stamped during a system:

1. Add `Option<&'static mut Component>` to the query tuple
2. Add `mut commands: Commands` to the system signature
3. Use this pattern:
```rust
if let Some(ref mut comp) = optional_field {
    comp.field = new_value;
} else {
    commands.entity(entity).insert(Component { field: new_value });
}
```

**Why:** `commands.insert` is deferred — won't be visible to the query until next apply_deferred. So first insertion uses commands, subsequent mutations use the query field directly. In multi-bounce CCD loops within the same frame, multiple `commands.insert` calls on the same entity will overwrite (last wins), which is correct behavior.

**How to apply:** Use this whenever a component needs to be lazily added on first occurrence and mutated in place on subsequent occurrences within ECS queries.
