# Type Migration

Replace `RootEffect` with `RootNode` in every struct field listed below. The `effect/` domain is being deleted entirely — these are the types **outside** `effect/` that reference it.

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
