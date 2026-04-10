# Definitions

| Term | Definition |
|------|-----------|
| **GameEntity** | Marker trait for entity types that participate in the death pipeline. Impl'd on Bolt, Cell, Wall, Breaker. |
| **Hp** | Unified health component. Replaces `CellHealth`. Used by all damageable entity types. |
| **KilledBy** | Component set on the killing blow only. Tracks which entity dealt the final damage. |
| **DamageDealt\<T\>** | Generic damage message per victim type. One Bevy message queue per T. Replaces `DamageCell`. |
| **KillYourself\<T\>** | Message sent when an entity should die. Replaces `RequestCellDestroyed`, `RequestBoltDestroyed`. |
| **Destroyed\<T\>** | Message sent after domain-specific death handling confirms the kill. Replaces `CellDestroyedAt`. |
| **DespawnEntity** | Message requesting deferred despawn. Processed in PostFixedUpdate. |
| **apply_damage\<T\>** | System that processes `DamageDealt<T>`, decrements Hp, sets KilledBy on the killing blow. |
| **detect_deaths\<T\>** | Per-domain system that detects Hp ≤ 0 and sends `KillYourself<T>`. |
| **on_destroyed\<T\>** | Bridge system that receives `Destroyed<T>` and dispatches Died/Killed/DeathOccurred triggers to the effect system. |
| **process_despawn_requests** | System that reads `DespawnEntity` and despawns entities. Runs in PostFixedUpdate. |
| **Killing blow** | The damage message that crosses Hp from positive to zero. Only this message sets KilledBy. |
| **Dealer** | The entity that originated the damage. Propagated through effect chains (shockwave inherits bolt, explosion inherits killer). |
| **Environmental death** | A death with no dealer (timer expiry, lifespan, etc.). KilledBy.dealer is None. Killed trigger is not fired. |
