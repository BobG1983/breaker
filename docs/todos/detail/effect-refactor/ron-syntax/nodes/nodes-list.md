# Effect Tree Nodes

## Root Nodes

| Node | Syntax | Description |
|------|--------|-------------|
| [Stamp](stamp.md) | `Stamp(StampTarget, Tree)` | Root node. Declares which entity receives the effect tree. Required at the top level of every `effects: []` list. |
| [Spawn](spawned.md) | `Spawn(EntityKind, Tree)` | Root node. Watches for new entities of a given kind and applies the tree to each one. |

## Tree Nodes

| Node | Syntax | Description |
|------|--------|-------------|
| [Fire](fire.md) | `Fire(Effect)` | Execute an effect immediately on the Owner. Leaf — nothing nests inside. |
| [When](when.md) | `When(Trigger, Tree)` | Repeating gate. Every time the trigger matches, evaluate the inner tree. Re-arms after each match. |
| [Once](once.md) | `Once(Trigger, Tree)` | One-shot gate. Evaluates inner tree on first trigger match, then removes itself. |
| [During](during.md) | `During(Condition, Scoped Tree)` | State-scoped. Applies inner effects while a condition is true, reverses them when it becomes false. Can cycle. |
| [Until](until.md) | `Until(Trigger, Scoped Tree)` | Event-scoped. Applies inner effects immediately, reverses them when the trigger fires. One-shot. |
| [Sequence](sequence.md) | `Sequence([Terminal, ...])` | Ordered multi-execute. Runs children left to right. Reversible if all children are reversible. |
| [On](on.md) | `On(Participant, Terminal)` | Redirects a terminal to a different entity involved in the trigger event instead of the Owner. |
| [Route](route.md) | `Route(RouteType, Tree)` | Terminal. Installs a tree on another entity. Bound = permanent, Staged = one-shot. Only appears inside On(). |
