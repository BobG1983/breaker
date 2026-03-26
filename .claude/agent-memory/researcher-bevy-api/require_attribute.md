---
name: "#[require] attribute syntax"
description: Exact syntax for #[require(...)] required component attribute in Bevy 0.18.1
type: reference
---

## #[require] Attribute (verified bevy_ecs macros source 0.18.0)

The `#[require(...)]` attribute is placed on a `#[derive(Component)]` struct/enum to declare that
other components must also be present whenever this component is inserted. Bevy auto-inserts the
required components using their default constructors if not already present.

### Supported forms

```rust
// 1. Component name only — uses Default::default()
#[require(MyComponent)]

// 2. Tuple-struct constructor
#[require(MyComponent(42.0))]

// 3. Named-field struct literal
#[require(MyComponent { field: value })]

// 4. Assignment expression (wraps in closure: || expr)
#[require(MyComponent = some_function())]

// 5. Enum variant
#[require(MyEnum::Variant)]
```

### Multiple requirements (comma-separated)

```rust
#[derive(Component)]
#[require(Transform, Visibility, InheritedVisibility)]
struct MyMarker;
```

### Custom default expression example

```rust
#[derive(Component)]
#[require(Speed(100.0), Health { max: 10, current: 10 })]
struct Enemy;
```

### Key rules

- Required components are ONLY inserted if not already present on the entity
- Requirements are transitive: if A requires B and B requires C, spawning A inserts B and C
- The `=` form and constructor forms are wrapped in closures internally — no access to outer generics
- Duplicate requirements on the same component are a compile error
- `#[require]` is NOT a proc-macro attribute that generates code — it's an attribute read by the
  `#[derive(Component)]` macro

### Import path

The `require` attribute is part of `#[derive(Component)]` — no separate import needed. Just use
`use bevy::prelude::*` or `use bevy::ecs::component::Component`.
