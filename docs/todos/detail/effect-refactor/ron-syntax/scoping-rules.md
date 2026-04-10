# Scoping Rules

Rules for what can nest inside what in effect tree definitions.

1. Every `effects: []` entry must start with **Stamp**.
2. **During/Until** immediate children must be reversible — a **Fire** must be a reversible Effect, a **Sequence** must have all reversible children. A **When** child relaxes this rule (reversal removes the listener, not individual firings).
3. **On** children must be **Terminals** (Fire, Stamp, or Route).
4. **Sequence** children must be **Terminals**.
5. **When/Once/Spawned** inner can be any **Tree** — no restrictions.
