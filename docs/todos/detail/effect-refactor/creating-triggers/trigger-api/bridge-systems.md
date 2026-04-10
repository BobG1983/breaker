# Bridge Systems

A bridge system translates a game event message into a trigger dispatch. It is the ONLY place triggers are dispatched — game systems send messages, bridges translate them into triggers and call the walker.

## The pattern

Every bridge system follows the same structure:

1. **Read** a game event message (BoltImpactCell, BoltLost, Destroyed\<T\>, etc.).
2. **Determine scope** — which entities to walk. Local: the participant entities. Global: all entities with BoundEffects/StagedEffects. Self: the owner entity.
3. **Build TriggerContext** from the message fields. Use the variant matching the trigger category.
4. **Call the walking algorithm** on each entity in scope, passing the trigger and context.

## Scheduling

Bridge systems run in FixedUpdate, after the game systems that produce the events they read. A bridge for bolt-cell collision triggers runs after the collision system. A bridge for death triggers runs after the death detection system.

## One bridge per trigger category

Group related triggers into one bridge. A bump bridge dispatches PerfectBumped, EarlyBumped, LateBumped, Bumped (Local) and PerfectBumpOccurred, EarlyBumpOccurred, LateBumpOccurred, BumpOccurred, BumpWhiffOccurred, NoBumpOccurred (Global) — all from a single bump event message.

A death bridge dispatches Died (Local, on victim), Killed(EntityKind) (Local, on killer), and DeathOccurred(EntityKind) (Global) — all from a single Destroyed\<T\> message.

## What bridges must NOT do

- DO NOT implement game logic. Bridges translate, they don't decide. If a cell should take damage, the collision system sends DamageCell, not the bridge.
- DO NOT modify entities. Bridges read messages and dispatch triggers. They don't insert components, mutate state, or send other messages.
- DO NOT skip triggers based on entity state. If a bump happened, the bridge dispatches Bumped. Whether the entity's trees match it is the walker's problem.
- DO NOT call fire_effect or reverse_effect directly. Bridges call the walking algorithm. The walker calls the command extensions.
