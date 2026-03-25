# Researcher-Codebase Memory

## Stable
- [rantzsoft-crate-exports.md](rantzsoft-crate-exports.md) -- Full public API for rantzsoft_spatial2d, rantzsoft_physics2d, rantzsoft_defaults, rantzsoft_defaults_derive: all types, traits, plugins, systems, SystemSets
- [defaults-config-pipeline.md](defaults-config-pipeline.md) -- End-to-end defaults/config pipeline: GameConfig derive macro, DefaultsCollection (14 handles), asset loading via bevy_asset_loader, 14 seed systems with iyes_progress tracking, hot-reload, and two gaps (HighlightConfig + TransitionConfig not seeded at runtime)
- [chip-select-flow.md](chip-select-flow.md) -- End-to-end chip offering + selection data flow map
- [bolt-boundary-and-bump-flow.md](bolt-boundary-and-bump-flow.md) -- Bolt-lost detection, bump grading chain, node completion chain, key coordinates
- [collision-message-flow.md](collision-message-flow.md) -- Physics collision message types, fields, ordering chain, and consumer map
- [spatial2d-propagation-flow.md](spatial2d-propagation-flow.md) -- Spatial2d propagation pipeline: save_previous, propagate, interpolation, Absolute/Relative hierarchy, orbit cells
- [highlight-detection-flow.md](highlight-detection-flow.md) -- Highlight detection, storage, cap, display, and juice flow map (15 HighlightKind variants, 6 detector systems)
- [effect-fired-pipeline.md](effect-fired-pipeline.md) -- EffectFired emission (bridges), observer handler pattern, 6 existing handlers, unhandled variants (MultiBolt, Shield)
- [damage-attribution-flow.md](damage-attribution-flow.md) -- source_chip threading from ActiveChains through EffectFired to DamageCell, bolt spawns, and incomplete evolution_damage pipeline

## Session History
See [ephemeral/](ephemeral/) -- not committed.
