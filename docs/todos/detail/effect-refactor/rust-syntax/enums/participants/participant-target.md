# Name
ParticipantTarget

# Syntax
```rust
enum ParticipantTarget {
    Bump(BumpTarget),
    Impact(ImpactTarget),
    Death(DeathTarget),
    BoltLost(BoltLostTarget),
}
```

# Description
- Bump: A role in a bump event. See [BumpTarget](bump-target.md)
- Impact: A role in a collision event. See [ImpactTarget](impact-target.md)
- Death: A role in a death event. See [DeathTarget](death-target.md)
- BoltLost: A role in a bolt lost event. See [BoltLostTarget](bolt-lost-target.md)
