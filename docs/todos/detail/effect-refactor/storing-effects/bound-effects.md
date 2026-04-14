# Name
BoundEffects

# Struct
```rust
#[derive(Component, Clone, Default)]
pub struct BoundEffects(pub Vec<(String, Tree)>);
```

# Description
Permanent effect tree storage on an entity. Each entry is a `(source, tree)` pair.

- `source`: identifies which chip, breaker definition, or other origin installed the tree. For nested shapes, source strings use a scope-path suffix to create unique keys (see Nested Shape Storage below).
- `tree`: the effect tree.

Condition state (`was this During active last frame?`) is NOT stored inside `BoundEffects`. It is tracked in a separate component, `DuringActive`, on the same entity.

## DuringActive

```rust
#[derive(Component, Default)]
pub struct DuringActive(pub HashSet<String>);
```

`DuringActive` holds the set of source strings whose `During` condition is currently active. `evaluate_conditions` reads and writes this set to detect transitions:

- `source` is in `DuringActive` → condition was true last frame
- `source` is absent from `DuringActive` → condition was false last frame

On condition-becomes-true: fire the scoped tree and insert the source into `DuringActive`.
On condition-becomes-false: reverse the scoped tree and remove the source from `DuringActive`.

## How entries are added

- `stamp_effect(entity, source, tree)` appends a `(source, tree)` pair.
- `route_effect(entity, source, tree, RouteType::Bound)` appends the same way.
- During initial effect dispatch, `Stamp(target, tree)` resolves to entities and appends here.
- Nested shape install commands (`DuringInstallCommand`) append with idempotency check — they skip append if an entry with the same source already exists.

## How entries are removed

- By source: when a chip is unequipped or a definition is removed, all entries with that source string are removed from the Vec.

## Nested Shape Storage

Nested condition shapes install sub-trees into `BoundEffects` with scope-path-suffixed source strings to avoid key collisions.

### Shape A — `When(X, During(Cond, inner))`

When trigger X fires, the walker encounters the inner `During(Cond, inner)` and calls `DuringInstallCommand`. This appends to `BoundEffects` with source key `{original_source}#installed[0]`. The `evaluate_conditions` system then picks up this new During entry on the next frame and manages its condition lifecycle.

### Shape B — `Until(X, During(Cond, inner))`

Same as Shape A, but the install happens when the Until is first walked (immediately, not on a trigger). The source key is also `{original_source}#installed[0]`. When the Until's trigger fires, the installed During is reversed and removed.

### Shape C — `During(Cond, When(Trigger, Fire(reversible)))`

When the outer During condition becomes true, `evaluate_conditions` encounters the inner `When` and calls `install_armed_entry()`, which appends to `BoundEffects` (or `StagedEffects`) with source key `{original_source}#armed[0]`. When the outer During condition becomes false, `evaluate_conditions` removes this armed entry by source and calls `reverse_all_by_source_dispatch` to reverse any effects that already fired.

### Shape D — `During(Cond, On(Participant, Fire(reversible)))`

Same as Shape C, but targets a participant entity rather than self.

## Pairing with StagedEffects

BoundEffects and StagedEffects are always inserted as a pair. If an entity has one, it has both. Command extensions ensure this by inserting both when either is absent.
