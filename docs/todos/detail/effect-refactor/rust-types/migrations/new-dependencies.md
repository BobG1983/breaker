# New Dependencies

## ordered-float

| Field | Value |
|-------|-------|
| Crate | `ordered-float` |
| Version | latest stable |
| Features | `serde` (for Serialize/Deserialize on OrderedFloat) |
| License | MIT |
| Why | All effect config structs use `OrderedFloat<f32>` instead of raw `f32`. This provides `Eq + Hash + Ord` on configs, enabling exact-match removal from EffectStack by (source, config) pair. Without it, f32 fields prevent deriving Eq on config structs. |
| Where | `breaker-game/Cargo.toml` |
| Serde | Transparent — `OrderedFloat<f32>` serializes/deserializes as a bare number. RON syntax unchanged. |
