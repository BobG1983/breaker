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

3. **Update the Impact and Impacted trigger systems** in `effect/triggers/` to listen for the new message. They already handle the four-trigger pattern (Impact global + Impacted targeted on both participants) — just add the new message reader.
