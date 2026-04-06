# Memory

- [Example RON files block folder load](stable/example-ron-blocks-load.md) — `.example.ron` docs files in `bolts/` and `breakers/` cause MissingAssetLoader, failing the LoadedFolder, hanging all scenarios at frame 0
- [Explode RON field rename — RESOLVED](stable/explode-field-rename.md) — explode_chaos.scenario.ron updated to `damage: 15.0`; `damage_mult` no longer exists on Explode
- [PauseMenuSelection missing resource — RESOLVED](stable/pause-menu-selection-missing-resource.md) — fixed via not_in_transition guard + any_with_component::<PauseMenuScreen> condition on handle_pause_input
- [ChainBolt tether corrupts bolt speed](stable/chain-bolt-speed-invariant.md) — enforce_distance_constraints averages axial velocity on taut tether; no apply_velocity_formula follows; BoltSpeedAccurate fires in tether_chain_bolt_stress at frames 3842–3846
