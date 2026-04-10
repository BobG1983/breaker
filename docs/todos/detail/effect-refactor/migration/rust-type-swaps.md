# Type Migration

The `effect/` domain is being deleted entirely. These are the types **outside** `effect/` that reference it.

## Definition Structs (RON-deserialized)

| Struct | Field | Current Type | New Type | File |
|--------|-------|-------------|----------|------|
| BoltDefinition | effects | `Vec<RootEffect>` | `Vec<RootNode>` | `src/bolt/definition.rs` |
| BreakerDefinition | effects | `Vec<RootEffect>` | `Vec<RootNode>` | `src/breaker/definition/types.rs` |
| ChipDefinition | effects | `Vec<RootEffect>` | `Vec<RootNode>` | `src/chips/definition/types.rs` |
| CellTypeDefinition | effects | `Option<Vec<RootEffect>>` | `Option<Vec<RootNode>>` | `src/cells/definition/data.rs` |
| WallDefinition | effects | `Vec<RootEffect>` | `Vec<RootNode>` | `src/walls/definition.rs` |

## Builder Optional Data (runtime)

| Struct | Field | Current Type | New Type | File |
|--------|-------|-------------|----------|------|
| OptionalBreakerData | effects | `Option<Vec<RootEffect>>` | `Option<Vec<RootNode>>` | `src/breaker/builder/core/types.rs` |
| OptionalWallData | definition_effects | `Option<Vec<RootEffect>>` | `Option<Vec<RootNode>>` | `src/walls/builder/core/types.rs` |
| OptionalWallData | override_effects | `Option<Vec<RootEffect>>` | `Option<Vec<RootNode>>` | `src/walls/builder/core/types.rs` |
| CellDefinitionParams | effects | `Option<Vec<RootEffect>>` | `Option<Vec<RootNode>>` | `src/cells/builder/core/types.rs` |

## EffectStack Components (passive effects)

| Current Type | New Type | Notes |
|-------------|----------|-------|
| `ActiveSpeedBoosts` | `EffectStack<SpeedBoostConfig>` | Was `Vec<f32>`, now `Vec<(String, SpeedBoostConfig)>` with source tracking |
| `ActiveSizeBoosts` | `EffectStack<SizeBoostConfig>` | Was `Vec<f32>` |
| `ActiveDamageBoosts` | `EffectStack<DamageBoostConfig>` | Was `Vec<f32>` |
| `ActiveBumpForces` | `EffectStack<BumpForceConfig>` | Was `Vec<f32>` |
| `ActiveQuickStops` | `EffectStack<QuickStopConfig>` | Was `Vec<f32>` |
| `ActiveVulnerability` | `EffectStack<VulnerableConfig>` | Was `Vec<f32>` |
| `ActivePiercings` | `EffectStack<PiercingConfig>` | Was `Vec<u32>` |
| (none) | `EffectStack<RampingDamageConfig>` | New — previously a different accumulation pattern |

## Message Changes (non-effect domains)

| Struct | Change | File |
|--------|--------|------|
| `BoltImpactBreaker` | Add `bump_status: BumpStatus` field where `enum BumpStatus { Active, Inactive }` | `src/bolt/messages.rs` |
