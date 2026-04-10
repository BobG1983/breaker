# Name
process_despawn_requests

# SystemSet
Runs in PostFixedUpdate — after all FixedUpdate systems including death handling, trigger evaluation, and death animations.

# Filepath
`src/shared/systems/process_despawn_requests.rs`

# Queries/Filters
No queries — reads `DespawnEntity` messages only. Uses Commands to despawn.

# Description
Read all `DespawnEntity` messages. For each, despawn the entity via `commands.entity(msg.entity).try_despawn()`.

Use `try_despawn` not `despawn` — the entity may have already been despawned by another system or a previous DespawnEntity message in the same frame.

This is the ONLY system that despawns entities in the death pipeline. No other system in the chain calls despawn.

DO run in PostFixedUpdate so the entity survives through all FixedUpdate processing.
DO NOT run in FixedUpdate — entities must be alive for trigger evaluation and data extraction.
