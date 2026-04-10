# Definitions

| Term | Definition |
|------|-----------|
| **Tree** | Any node: Fire, When, Once, During, Until, Sequence, On, Spawned. Can nest arbitrarily (subject to scoping rules). |
| **Scoped Tree** | A Tree inside During/Until. An immediate child Fire must be reversible. An immediate child Sequence must be reversible (all children reversible). A When child relaxes to any Tree (reversal removes the listener, not individual firings). |
| **Terminal** | A leaf operation: Fire(Effect), Stamp(Tree), or Route(Tree). Fire executes immediately. Stamp adds a Tree permanently to the target's BoundEffects. Route adds a Tree one-shot to the target's StagedEffects. |
| **Inner** | The Tree contained by a wrapper node — the thing inside When/Once/During/Until/Spawned. |
| **Owner** | The entity whose BoundEffects/StagedEffects is being walked. Fire always executes on the Owner. On() redirects to a different entity (a Participant). |
| **StampTarget** | First argument to Stamp(). Which entity or entities receive the Tree. See [valid-stamp-targets.md](valid-stamp-targets.md). |
| **Trigger** | A game event that gates evaluation. First argument to When, Once, and Until (e.g. `PerfectBumped`, `Impacted(Cell)`). See [triggers-list.md](triggers-list.md). |
| **Effect** | The action that Fire executes on the Owner (e.g. `SpeedBoost(1.5)`, `Shockwave(ShockwaveConfig(...))`). See [effects-list.md](effects-list.md). |
| **EntityKind** | Entity type filter used by Trigger parameters and Spawned (e.g. `Cell`, `Any`). See [entitykinds-list.md](entitykinds-list.md). |
| **Condition** | A state that gates During. Unlike a Trigger (one-time event), a Condition has a start and end and can cycle (e.g. `NodeActive`, `ComboActive(3)`). See [conditions-list.md](conditions-list.md). |
| **Participant** | A named role in a Trigger event, used by On() to redirect a Terminal to a non-Owner entity (e.g. `ImpactTarget::Impactee`, `DeathTarget::Killer`). See [participants/](participants/index.md). |
| **Local** | Trigger scope. Fires only on the entities involved in the event (e.g. the bolt and breaker that bumped). |
| **Global** | Trigger scope. Fires on all entities that have effect trees. |
| **Self** | Trigger scope. Fires only on the Owner entity. Used by TimeExpires. |
| **BoundEffects** | Permanent effect tree storage on an entity. Stamp adds here. When/Once entries live here and re-arm on each match. |
| **StagedEffects** | One-shot effect tree storage on an entity. Route adds here. Armed inner trees from When live here and are consumed when matched. |
