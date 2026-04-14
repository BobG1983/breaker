# Name
Trigger

# Derives
`Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize`

# Syntax
```rust
enum Trigger {
    PerfectBumped,
    EarlyBumped,
    LateBumped,
    Bumped,
    PerfectBumpOccurred,
    EarlyBumpOccurred,
    LateBumpOccurred,
    BumpOccurred,
    BumpWhiffOccurred,
    NoBumpOccurred,
    Impacted(EntityKind),
    ImpactOccurred(EntityKind),
    Died,
    Killed(EntityKind),
    DeathOccurred(EntityKind),
    BoltLostOccurred,
    NodeStartOccurred,
    NodeEndOccurred,
    NodeTimerThresholdOccurred(f32),
    TimeExpires(f32),
}
```

# Description
- PerfectBumped: Local. Bolt bumped with perfect timing. Fires on bolt + breaker. See [perfect-bumped](../ron-syntax/triggers/perfect-bumped.md)
- EarlyBumped: Local. Bolt bumped with early timing. Fires on bolt + breaker. See [early-bumped](../ron-syntax/triggers/early-bumped.md)
- LateBumped: Local. Bolt bumped with late timing. Fires on bolt + breaker. See [late-bumped](../ron-syntax/triggers/late-bumped.md)
- Bumped: Local. Bolt bumped with any successful timing. Fires on bolt + breaker. See [bumped](../ron-syntax/triggers/bumped.md)
- PerfectBumpOccurred: Global. A perfect bump happened somewhere. Fires on all entities. See [perfect-bump-occurred](../ron-syntax/triggers/perfect-bump-occurred.md)
- EarlyBumpOccurred: Global. An early bump happened somewhere. Fires on all entities. See [early-bump-occurred](../ron-syntax/triggers/early-bump-occurred.md)
- LateBumpOccurred: Global. A late bump happened somewhere. Fires on all entities. See [late-bump-occurred](../ron-syntax/triggers/late-bump-occurred.md)
- BumpOccurred: Global. Any successful bump happened somewhere. Fires on all entities. See [bump-occurred](../ron-syntax/triggers/bump-occurred.md)
- BumpWhiffOccurred: Global. Bump timing window expired without contact. Fires on all entities. See [bump-whiff-occurred](../ron-syntax/triggers/bump-whiff-occurred.md)
- NoBumpOccurred: Global. Bolt hit breaker with no bump input. Fires on all entities. See [no-bump-occurred](../ron-syntax/triggers/no-bump-occurred.md)
- Impacted: Local. This entity collided with an entity of the given kind. Fires on both collision participants. See [impacted](../ron-syntax/triggers/impacted.md), [EntityKind](entity-kind.md)
- ImpactOccurred: Global. A collision involving the given entity kind happened somewhere. Fires on all entities. See [impact-occurred](../ron-syntax/triggers/impact-occurred.md), [EntityKind](entity-kind.md)
- Died: Local. This entity died. Fires on victim only. See [died](../ron-syntax/triggers/died.md)
- Killed: Local. This entity killed an entity of the given kind. Fires on killer only. See [killed](../ron-syntax/triggers/killed.md), [EntityKind](entity-kind.md)
- DeathOccurred: Global. An entity of the given kind died somewhere. Fires on all entities. See [death-occurred](../ron-syntax/triggers/death-occurred.md), [EntityKind](entity-kind.md)
- BoltLostOccurred: Global. A bolt fell off the bottom. Fires on all entities. See [bolt-lost-occurred](../ron-syntax/triggers/bolt-lost-occurred.md)
- NodeStartOccurred: Global. A new node started. Fires on all entities. See [node-start-occurred](../ron-syntax/triggers/node-start-occurred.md)
- NodeEndOccurred: Global. The current node ended. Fires on `OnEnter(NodeState::Teardown)`, after `AnimateOut` completes and cells have been cleaned up. Fires on all entities. See [node-end-occurred](../ron-syntax/triggers/node-end-occurred.md)
- NodeTimerThresholdOccurred: Global. Node timer ratio crossed the given threshold (0.0-1.0). Fires on all entities. See [node-timer-threshold-occurred](../ron-syntax/triggers/node-timer-threshold-occurred.md)
- TimeExpires: Self. Countdown of the given seconds reached zero. Fires on owner only. Internal -- used by Until desugaring. See [time-expires](../ron-syntax/triggers/time-expires.md)
