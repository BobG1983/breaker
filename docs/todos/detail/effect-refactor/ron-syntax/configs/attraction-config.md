# Name
AttractionConfig

# Parameters
- attraction_type: AttractionType (`Breaker`, `Bolt`, `Cell`, `Wall`)
- force: f32
- max_force: Option<f32>

# Description
- attraction_type: Which entity type the bolt steers toward. One of: Breaker, Bolt, Cell, Wall.
- force: Attraction strength -- how aggressively the bolt curves toward the nearest target per tick
- max_force: Optional cap on the per-tick steering delta to prevent instant turns (None = uncapped)
