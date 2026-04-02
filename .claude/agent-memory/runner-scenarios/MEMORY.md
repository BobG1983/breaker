# Memory

- [Example RON files block folder load](stable/example-ron-blocks-load.md) — `.example.ron` docs files in `bolts/` and `breakers/` cause MissingAssetLoader, failing the LoadedFolder, hanging all scenarios at frame 0
- [Explode RON field rename](stable/explode-field-rename.md) — `explode_chaos.scenario.ron` uses `damage_mult` but the Explode struct now requires `damage`
