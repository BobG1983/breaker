# Scenario Runner

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Scenario** | A named automated test run defined in a `.scenario.ron` file | `ScenarioDefinition`, `ScenarioConfig` |
| **Invariant** | A runtime assertion checked every frame during a scenario run; any violation fails the run | `InvariantKind`, `ViolationLog` |
| **Chaos** | Input strategy that injects random game actions each frame using a seeded RNG | `InputStrategy::Chaos`, `ChaosParams` |
| **Scripted** | Input strategy that plays back a deterministic list of frame-action pairs | `InputStrategy::Scripted`, `ScriptedParams` |
| **Hybrid** | Input strategy that runs scripted actions for N frames then switches to chaos | `InputStrategy::Hybrid`, `HybridParams` |
| **Recording** | Dev-only system that captures live `GameAction` inputs to a `.scripted.ron` file for later playback | `RecordingConfig`, `--record` flag |
| **FrameMutation** | A scripted mutation applied at a specific fixed-update frame during a scenario run | `FrameMutation`, `MutationKind` |
| **MutationKind** | Enum of mutation operations: `SetBreakerState`, `SetTimerRemaining`, `SpawnExtraEntities`, `MoveBolt`, `TogglePause`, `SetRunStat`, `DecrementRunStat`, `InjectOverStackedChip`, `InjectDuplicateOffers`, `InjectMaxedChipOffer`, `SpawnExtraSecondWindWalls`, `SpawnExtraShieldWalls`, `SpawnExtraPulseRings`, `SpawnExtraChainArcs`, `InjectMismatchedBoltAabb`, `SpawnExtraGravityWells` | `MutationKind` |
