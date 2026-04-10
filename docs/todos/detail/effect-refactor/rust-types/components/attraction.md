# Name
ActiveAttractions, AttractionEntry

# Struct
```rust
/// Collection of active attraction forces applied to the bolt.
/// Not an EffectStack — uses custom storage with named entries.
#[derive(Component)]
pub struct ActiveAttractions(pub Vec<AttractionEntry>);

/// A single attraction force entry, keyed by source name.
pub struct AttractionEntry {
    /// Identifier for the source of this attraction (for removal by reverse).
    pub source: String,
    /// Type of attraction behavior.
    pub attraction_type: AttractionType,
    /// Base force magnitude.
    pub force: f32,
    /// Optional cap on the applied force.
    pub max_force: Option<f32>,
}
```

# Location
`src/effect/effects/attraction/`

# Description
`ActiveAttractions` is a component added to the bolt that accumulates attraction force entries from multiple sources.

- **Added by**: `AttractionConfig.fire()` pushes an `AttractionEntry` onto the `ActiveAttractions` vec (inserting the component if absent).
- **Tick**: The attraction system reads all entries each frame and applies the combined force to the bolt's velocity.
- **Removed by**: `AttractionConfig.reverse()` removes the entry matching the source name. If the vec becomes empty, the component is removed entirely.
- **Note**: This is not an `EffectStack` — it uses custom vec-based storage because each entry has distinct parameters (source, type, force, max_force) rather than uniform stacks.
