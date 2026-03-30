# Target Resolution

The `Target` enum is used in `On { target, then }` nodes. Resolution depends on context.

## Target Enum

| Target | Description |
|--------|------------|
| `Bolt` | A specific bolt (context-sensitive) |
| `AllBolts` | All bolt entities |
| `Breaker` | The breaker entity |
| `Cell` | A specific cell (context-sensitive) |
| `AllCells` | All cell entities |
| `Wall` | A specific wall (context-sensitive) |
| `AllWalls` | All wall entities |

## At Dispatch Time

| Target | Resolves to |
|--------|------------|
| `Bolt` | Primary bolt entity |
| `Breaker` | The breaker entity |
| `Cell` / `Wall` | No-op (entities may not exist yet) |
| `AllBolts` / `AllCells` / `AllWalls` | Desugared to `When(NodeStart, On(All*, permanent: true, ...))` pushed to Breaker BoundEffects. Re-fires every node start for the rest of the run. See [Dispatch](dispatch.md#all-target-desugaring) |

New bolts inherit the primary bolt's BoundEffects if spawned with `SpawnBolts(inherit: true)`.

## At Runtime (Inside Trigger Evaluation)

When a trigger system encounters `On(target, children)` while walking chains, it resolves `target` from its message data.

**Singular targets** (Bolt, Cell, Wall, Breaker) — context-sensitive, from the message that triggered evaluation:
- `BoltImpactCell { bolt, cell }` → Target::Bolt resolves to `bolt`, Target::Cell resolves to `cell`
- `BoltImpactBreaker { bolt, breaker }` → Target::Bolt resolves to `bolt`, Target::Breaker resolves to `breaker`

**Plural targets** (AllBolts, AllCells, AllWalls) — always resolve via query to all matching entities.

**Breaker** — always resolves via query (single entity), in both dispatch and runtime contexts.

Each trigger system knows how to resolve targets because it has the message data. No shared TriggerContext struct needed.
