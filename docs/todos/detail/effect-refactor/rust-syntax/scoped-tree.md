# Name
ScopedTree

# Syntax
```rust
enum ScopedTree {
    Fire(ReversibleEffectType),
    Sequence(Vec<ReversibleEffectType>),
    When(Trigger, Box<Tree>),
    On(ParticipantTarget, ScopedTerminal),
}
```

# Description
- Fire: Execute a reversible effect immediately on the Owner. See [reversible-effect-type.md](enums/reversible-effect-type.md)
- Sequence: Ordered multi-execute of reversible effects. All children must be reversible. See [reversible-effect-type.md](enums/reversible-effect-type.md)
- When: Repeating gate inside a scoped context. Reversal removes the listener, not individual firings. Inner tree is unrestricted. See [trigger.md](enums/trigger.md), [tree.md](tree.md)
- On: Redirects a scoped terminal to a different entity involved in the trigger event. See [participant-target.md](enums/participants/participant-target.md), [scoped-terminal.md](scoped-terminal.md)
