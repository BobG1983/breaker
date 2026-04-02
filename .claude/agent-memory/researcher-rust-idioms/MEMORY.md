## Stable Patterns

- [Typestate builder for Bevy component bundles](pattern_typestate_builder.md) — separate structs as state markers, multiple build() impls, no PhantomData, no trait abstraction
- [ScreenLifecycle trait pattern](pattern_screen_lifecycle_trait.md) — associated methods on States supertrait, no derive macro until 6+ impls, active phase excluded from trait
- [Declarative state routing](pattern_declarative_routing.md) — flat OnExit for static routes; SystemId HashMap + world.run_system for dynamic/cross-level routes; game crate owns handler fns

## Session History
See [ephemeral/](ephemeral/) — not committed.
