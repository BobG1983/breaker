# Adding a New Effect

1. **Create** `effect/effects/new_effect.rs`:
   - `pub(crate) fn fire(entity: Entity, /* params */, world: &mut World)`
   - `pub(crate) fn reverse(entity: Entity, /* params */, world: &mut World)`
   - `pub(crate) fn register(app: &mut App)` (if runtime systems needed)
   - Any per-effect components, runtime systems

2. **Add variant** to `EffectKind` enum in `effect/core/types/definitions/enums.rs`:
   ```rust
   FreezeBolt { duration: f32 },
   ```

3. **Add match arms** in `EffectKind::fire()` and `EffectKind::reverse()` in `effect/core/types/definitions/fire.rs` and `reverse.rs` — compiler will force this (exhaustive match). Add the new arm to the deepest helper that handles similar effects (`fire_breaker_effects`, `fire_utility_and_spawn`, etc.).

4. **Add** `new_effect::register(app)` call in `effect/effects/mod.rs` (if the effect has runtime systems).

5. **RON files** can immediately use `Do(FreezeBolt(duration: 2.0))`.
