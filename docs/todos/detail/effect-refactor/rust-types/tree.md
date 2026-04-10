# Name
Tree

# Derives
`Debug, Clone, PartialEq, Eq, Serialize, Deserialize`

# Syntax
```rust
enum Tree {
    Fire(EffectType),
    When(Trigger, Box<Tree>),
    Once(Trigger, Box<Tree>),
    During(Condition, Box<ScopedTree>),
    Until(Trigger, Box<ScopedTree>),
    Sequence(Vec<Terminal>),
    On(ParticipantTarget, Terminal),
}
```

# Description
- Fire: Execute an effect immediately on the Owner. See [effect-type.md](enums/effect-type.md)
- When: Repeating gate. Every time the trigger matches, evaluate the inner tree. See [trigger.md](enums/trigger.md)
- Once: One-shot gate. Evaluates inner tree on first trigger match, then removes itself. See [trigger.md](enums/trigger.md)
- During: State-scoped. Applies inner effects while a condition is true, reverses them when false. See [condition.md](enums/condition.md), [scoped-tree.md](scoped-tree.md)
- Until: Event-scoped. Applies inner effects immediately, reverses them when the trigger fires. See [trigger.md](enums/trigger.md), [scoped-tree.md](scoped-tree.md)
- Sequence: Ordered multi-execute. Runs children left to right. See [terminal.md](terminal.md)
- On: Redirects a terminal to a different entity involved in the trigger event. See [participant-target.md](enums/participants/participant-target.md), [terminal.md](terminal.md)
