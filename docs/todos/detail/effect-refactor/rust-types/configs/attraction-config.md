# Name
AttractionConfig

# Syntax
```rust
struct AttractionConfig {
    attraction_type: AttractionType,
    force: OrderedFloat<f32>,
    max_force: Option<OrderedFloat<f32>>,
}
```

# Description
- attraction_type: Which entity type the bolt steers toward. See [AttractionType](../enums/attraction-type.md)
- force: Attraction strength -- how aggressively the bolt curves toward the nearest target per tick
- max_force: Optional cap on the per-tick steering delta to prevent instant turns (None = uncapped)
