# Name
TriggerContext

# Derives
`Debug, Clone`

# Syntax
```rust
enum TriggerContext {
    Bump { bolt: Option<Entity>, breaker: Entity },
    Impact { impactor: Entity, impactee: Entity },
    Death { victim: Entity, killer: Option<Entity> },
    BoltLost { bolt: Entity, breaker: Entity },
    None,
}
```

# Description
Carries the entities involved in a trigger event so that On nodes can resolve ParticipantTargets during tree walking.

- Bump: Participants in a bump event. `On(Bump(Bolt))` resolves to `bolt` — if None (NoBump, BumpWhiff without a bolt), the On is skipped. `On(Bump(Breaker))` resolves to `breaker`.
- Impact: Both participants in a collision. `On(Impact(Impactor))` resolves to `impactor`. `On(Impact(Impactee))` resolves to `impactee`.
- Death: The victim and optionally the killer. `On(Death(Victim))` resolves to `victim`. `On(Death(Killer))` resolves to `killer` — if None (environmental death), the On is skipped.
- BoltLost: The bolt that was lost and the breaker that lost it. `On(BoltLost(Bolt))` resolves to `bolt`. `On(BoltLost(Breaker))` resolves to `breaker`.
- None: No participants. Used for global triggers with no event-specific entities (NodeStartOccurred, NodeEndOccurred, NodeTimerThresholdOccurred, TimeExpires). On nodes should never appear inside trees gated by these triggers — that is a RON authoring error caught by validation.

Created by the trigger dispatch system when a game event occurs. Passed through to the walking algorithm and forwarded to every node evaluation during the walk.

DO use the enum variant that matches the trigger category — a bump event always produces `Bump { ... }`, never `Impact { ... }`.
DO NOT construct a None context for triggers that have participants — use the correct variant even if you think no On nodes will be encountered.
DO NOT resolve a ParticipantTarget against the wrong context variant — `On(Impact(Impactee))` with a `Bump` context is a mismatch and should be skipped, not panicked.
