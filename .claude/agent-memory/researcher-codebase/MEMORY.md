# Memory

- [breaker-game cross-domain topology](project-cross-domain-topology.md) — Key coupling cycles, entity marker problem, and split feasibility tiers for breaker-game
- [bolt-cell collision architecture](bolt-cell-collision-architecture.md) — CCD Minkowski expansion, two independent scale systems (entity_scale vs compute_grid_scale), which components drive radius

- [bolt-lost mid-node respawn bypasses birthing](bolt-lost-respawn-gap.md) — bolt_lost mutates bolt in-place during Playing; begin_node_birthing only fires OnEnter(AnimateIn); tick_birthing already covers Playing so inserting Birthing in bolt_lost is the fix

- [Time<Virtual> pausing hazard](time-virtual-pausing-hazard.md) — Out transitions leave Time<Virtual> paused; FixedUpdate-dependent systems hang if reached while paused; affects run-to-menu and run-start flows

## Session History
See [ephemeral/](ephemeral/) — not committed.
