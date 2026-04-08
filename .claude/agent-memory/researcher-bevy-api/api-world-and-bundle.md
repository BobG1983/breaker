---
name: Bevy 0.18.1 World Access and Bundle
description: One-shot systems, Commands::run_system, Bundle/DynamicBundle traits, BundleInfo inspection
type: reference
---

# World Access — One-shot systems and Commands::run_system (Bevy 0.18.1)

Verified against docs.rs/bevy/0.18.1, github.com/bevyengine/bevy/tree/v0.18.1.

## One-shot system registration and execution

```rust
// Register at app build time — returns SystemId (Copy + Send + Sync)
let id: SystemId = app.world_mut().register_system(my_fn);

// Or from &mut World directly
let id: SystemId = world.register_system(my_fn);

// Run from Commands in a normal system (deferred — executes at ApplyDeferred)
fn my_system(mut commands: Commands, ids: Res<RouteSystemIds>) {
    commands.run_system(ids.some_route);       // no input
    commands.run_system_with(ids.other, val);  // with input
}

// Run immediately from &mut World (exclusive context only)
world.run_system(id)
world.run_system_with(id, input)
```

- `SystemId<I = (), O = ()>` is `Copy + Send + Sync` — safe to store in Resources
- `Commands::run_system` is deferred (runs at next ApplyDeferred sync point, same frame)
- No return value from `Commands::run_system` — one-shot system writes to resources/components directly
- One-shot systems can read any `SystemParam` (Res, ResMut, Query, etc.) — no need for &mut World

---

# Bundle trait — introspection and iteration (Bevy 0.18.1)

Verified against docs.rs/bevy/0.18.1/bevy/ecs/bundle/.

## Trait definition

```rust
pub unsafe trait Bundle: DynamicBundle + Send + Sync + 'static {
    // Required method — returns None for each component not yet registered
    fn get_component_ids(
        components: &Components,
    ) -> impl Iterator<Item = Option<ComponentId>>;
}
```

- `unsafe trait` — manual impls are unsupported; always use `#[derive(Bundle)]`
- NOT dyn-compatible (cannot use as trait object)

## DynamicBundle supertrait

```rust
pub trait DynamicBundle: Sized {
    type Effect;

    // Low-level: moves component pointers out, calling func per component
    // Requires unsafe, raw pointer work
    unsafe fn get_components(
        ptr: MovingPtr<'_, Self>,
        func: &mut impl FnMut(StorageType, OwningPtr<'_>),
    );

    // Runs post-insertion effects on the entity
    unsafe fn apply_effect(
        ptr: MovingPtr<'_, MaybeUninit<Self>>,
        entity: &mut EntityWorldMut<'_>,
    );
}
```

## Tuple impls

- `()` implements Bundle (empty set)
- Tuples of up to 16 items where each item: Bundle → impl Bundle
- Nest tuples for >15 components: `((A, B, C, ..., O), P, Q)`

## BundleInfo — inspection after World registration

Obtained only via `World::bundles()` — cannot be constructed directly.

```rust
let bundles: &Bundles = world.bundles();
let bundle_id: Option<BundleId> = bundles.get_id(TypeId::of::<MyBundle>());
let info: Option<&BundleInfo> = bundle_id.and_then(|id| bundles.get(id));

// BundleInfo methods:
info.id() -> BundleId
info.explicit_components() -> &[ComponentId]   // defined in the bundle struct
info.required_components() -> &[ComponentId]   // pulled in by #[require(...)]
info.contributed_components() -> &[ComponentId] // explicit + required combined
info.iter_explicit_components() -> impl Iterator<Item = ComponentId>
info.iter_contributed_components() -> impl Iterator<Item = ComponentId>
info.iter_required_components() -> impl Iterator<Item = ComponentId>
```

BundleInfo is only populated after the bundle type has been registered (i.e., spawned or
explicitly registered). Before that, `bundles.get_id(TypeId::of::<MyBundle>())` returns `None`.

## Common questions

1. **Can you destructure `impl Bundle`?**
   No. `impl Bundle` is an opaque return type — you cannot pattern-match or destructure it.

2. **Can you iterate over components in a Bundle?**
   Not safely outside the ECS. `DynamicBundle::get_components` consumes the bundle via
   `MovingPtr` (raw pointers) — ECS internals only.
   The only user-facing iteration is `BundleInfo::iter_explicit_components()`, which gives
   `ComponentId`s only (not component values), and requires a World.

3. **Inspect bundle contents without spawning into a World?**
   Very limited. `Components::default()` can be constructed; call `register_component::<T>()`
   on it, then `Bundle::get_component_ids`. But there is NO way to get actual component VALUES
   without spawning into a World.

4. **How to test bundle contains right components?**
   The idiomatic approach IS to use a World. Create a minimal `World::default()`, spawn
   the bundle, then query for expected components.

## Testing bundles — the correct Bevy pattern

```rust
#[test]
fn my_bundle_has_expected_components() {
    let mut world = World::default();
    let entity = world.spawn(MyBundle { ... }).id();

    assert!(world.get::<ComponentA>(entity).is_some());
    assert!(world.get::<ComponentB>(entity).is_some());
    assert_eq!(*world.get::<ComponentA>(entity).unwrap(), expected_value);
}
```
