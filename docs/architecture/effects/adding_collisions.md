# Adding a New Collision Type

1. **Add collision detection system** in the appropriate entity domain:
   - Bolt collisions → `bolt/`
   - Breaker collisions → `breaker/`
   - Cell collisions → `cells/`

2. **Define the impact message** in the detecting domain:
   ```rust
   #[derive(Message, Clone, Debug)]
   pub struct BreakerImpactCell {
       pub breaker: Entity,
       pub cell: Entity,
   }
   ```

3. **Update the impact bridge systems** in `effect_v3/triggers/impact/bridges/system.rs` to listen for the new message. The bridge functions already handle the four-trigger pattern per collision type (`ImpactOccurred(EntityKind)` global on each participant kind + `Impacted(EntityKind)` local on each participant entity) — add a new `on_impact_<kind>_<kind>` bridge function and register it in `effect_v3/triggers/impact/register.rs`.
