# Memory

- [Example RON files block folder load](stable/example-ron-blocks-load.md) — `.example.ron` docs files in `bolts/` and `breakers/` cause MissingAssetLoader, failing the LoadedFolder, hanging all scenarios at frame 0
- [Explode RON field rename — RESOLVED](stable/explode-field-rename.md) — explode_chaos.scenario.ron updated to `damage: 15.0`; `damage_mult` no longer exists on Explode
- [PauseMenuSelection missing resource on FadeIn transition](stable/pause-menu-selection-missing-resource.md) — handle_pause_input panics when FadeIn transition pauses Time<Virtual> before PauseMenuSelection exists; fix: Option<ResMut<PauseMenuSelection>> or init_resource at plugin startup
