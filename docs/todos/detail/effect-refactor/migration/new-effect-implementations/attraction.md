# Name
Attraction

# Enum Variant
- `EffectType::Attraction(AttractionConfig)`
- `ReversibleEffectType::Attraction(AttractionConfig)`

# Config
`AttractionConfig { attraction_type: AttractionType, force: OrderedFloat<f32>, max_force: Option<OrderedFloat<f32>> }`

# Fire
1. Insert or get `ActiveAttractions` component on the target entity.
2. Push an `AttractionEntry { source, attraction_type, force, max_force }` onto the Vec.
3. Fire does NOT steer the bolt -- `apply_attraction` does.

Note: Attraction does not use `EffectStack` because it has non-f32 fields (`AttractionType`) and per-entry distinct behavior (different attraction types in the same stack). It uses its own `ActiveAttractions` component instead.

# Reverse
1. Find and remove the matching `AttractionEntry` from `ActiveAttractions` by `(source, config match)`.
2. The `ActiveAttractions` component is left in place even when empty (an empty Vec is functionally equivalent to no component — `apply_attraction` has nothing to iterate).
3. `reverse_all_by_source` uses `retain` to remove all entries matching the given source.

# Source Location
`src/effect_v3/effects/attraction/config.rs`

# New Types
- `ActiveAttractions` -- component containing `Vec<AttractionEntry>`. Each entry represents one active attraction effect on the entity.
- `AttractionEntry` -- struct. Fields: `source: String`, `attraction_type: AttractionType`, `force: f32`, `max_force: Option<f32>`.

# New Systems

## apply_attraction
- **What it does**: For each entity with `ActiveAttractions`, for each entry: query for the nearest entity matching `attraction_type` (using spatial queries). Calculate a steering force vector toward the nearest target. Apply the force to the entity's velocity. If `max_force` is set, clamp the per-tick steering delta to that cap.
- **What it does NOT do**: Does not modify `ActiveAttractions`. Does not handle fire/reverse. Does not add or remove entries.
- **Schedule**: FixedUpdate, after spatial queries are up to date.
