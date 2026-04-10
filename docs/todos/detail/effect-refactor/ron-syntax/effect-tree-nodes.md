# Effect Tree Nodes

**`Stamp(StampTarget, Tree)`** — Top-level wrapper. Required root of every `effects: []` entry. Declares which entity type to stamp onto. Not nestable inside other nodes — always the outermost layer. Contains one **Tree**.

**`Fire(Effect)`** — **Terminal**. Execute an Effect immediately on the **Owner**. Leaf node — nothing nests inside it.

**`When(Trigger, Tree)`** — Repeating gate. Every time the **Trigger** matches, evaluate the **Inner**. Entry stays in BoundEffects and re-arms. **Inner** can be any **Tree**.

**`Once(Trigger, Tree)`** — One-shot gate. Same as When but self-removes from BoundEffects after first match. **Inner** can be any **Tree**. Once doesn't care about reversibility — it's just a gate.

**`During(Condition, Scoped Tree)`** — State-scoped. Fires **Inner** on **Condition** start, reverses on **Condition** end. Stays in BoundEffects (Conditions can cycle). **Inner** is a **Scoped Tree** — immediate child Fire/Sequence must be reversible. A When child relaxes this (reversal removes the listener, not individual firings).

**`Until(Trigger, Scoped Tree)`** — Event-scoped. Fires **Inner** immediately, reverses when **Trigger** fires, then self-removes. Same rules as During — **Inner** is a **Scoped Tree**.

**`Sequence([Terminal, ...])`** — Ordered multi-execute. Runs children left to right. Each child is a **Terminal**. A Sequence is reversible if all its children are reversible. Reversal runs in reverse order.

**`On(Participant, Terminal)`** — **Participant** redirect. Routes the **Terminal** to a non-**Owner** entity from the trigger context. Never `On(Owner, ...)` — Fire already targets the **Owner** implicitly.

**`Spawned(EntityKind, Tree)`** — Entity-add listener. Fires **Inner** on `Added<T>` via bridge systems.
