# Definitions

| Term | Definition |
|------|-----------|
| **RootNode** | Top-level entry in an `effects: []` list. Either Stamp(StampTarget, Tree) or Spawn(EntityKind, Tree). |
| **Tree** | Any node: Fire, When, Once, During, Until, Sequence, On. Can nest arbitrarily (subject to scoping rules). |
| **Scoped Tree** | A Tree inside During/Until. An immediate child Fire must be a reversible Effect. An immediate child Sequence must have all reversible children. A When child relaxes this rule (reversal removes the listener, not individual firings). |
| **Terminal** | A leaf operation: Fire(Effect) or Route(RouteType, Tree). Fire executes immediately. Route installs a tree on another entity. |
| **RouteType** | How Route installs a tree: Bound (permanent → BoundEffects) or Staged (one-shot → StagedEffects). |
| **Inner** | The Tree contained by a wrapper node — the thing inside When/Once/During/Until. |
| **Owner** | The entity whose BoundEffects/StagedEffects is being walked. Fire always executes on the Owner. On() redirects to a different entity (a Participant). |
| **StampTarget** | First argument to Stamp(). Which entity or entities receive the Tree. See [stamp-targets/](stamp-targets/index.md). |
| **Trigger** | A game event that gates evaluation. First argument to When, Once, and Until (e.g. `PerfectBumped`, `Impacted(Cell)`). See [triggers/](triggers/index.md). |
| **Effect** | The action that Fire executes on the Owner (e.g. `SpeedBoost(1.5)`, `Shockwave(ShockwaveConfig(...))`). See [effects/](effects/index.md). |
| **EntityKind** | Entity type filter used by Trigger parameters and Spawn (e.g. `Cell`, `Any`). See [entitykinds-list.md](entitykinds-list.md). |
| **Condition** | A state that gates During. Unlike a Trigger (one-time event), a Condition has a start and end and can cycle (e.g. `NodeActive`, `ComboActive(3)`). See [conditions/](conditions/index.md). |
| **Participant** | A named role in a Trigger event, used by On() to redirect a Terminal to a non-Owner entity (e.g. `Impact(Impactee)`, `Death(Killer)`). See [participants/](participants/index.md). |
| **Local** | Trigger scope. Fires only on the Participant entities involved in the event (e.g. the bolt and breaker that bumped). |
| **Global** | Trigger scope. Fires on all entities that have effect trees. |
| **Self** | Trigger scope. Fires only on the Owner entity. Used by TimeExpires. |
| **BoundEffects** | Permanent effect tree storage on an entity. Route(Bound) adds here. When/Once entries live here and re-arm on each match. |
| **StagedEffects** | One-shot effect tree storage on an entity. Route(Staged) adds here. Armed inner trees from When live here and are consumed when matched. |
