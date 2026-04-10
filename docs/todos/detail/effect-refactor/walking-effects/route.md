# Route

## Receives
`Route(RouteType, Box<Tree>)` — a terminal that installs a tree on the current target entity.

## Behavior

1. Call `route_effect(entity, source, tree, route_type)` where entity is the current target (either the Owner, or a participant entity if inside On).

That's it. Route delegates to the command extension. The command extension handles inserting into BoundEffects or StagedEffects depending on the RouteType.

## Constraints

- DO use the current target entity, which may be a participant if Route appears inside On.
- DO NOT evaluate the tree contents. Route installs the tree for later evaluation — the tree will be walked when its own triggers fire.
- DO pass the source string through so the installed entry can be traced and removed by source.
