# Evaluating Conditions

How During nodes monitor conditions and fire/reverse their scoped effects.

Conditions are not triggers — they are continuous states polled each frame. The condition evaluation system does not call `walk_effects`. It directly manages During entries in BoundEffects.

- [overview.md](overview.md) — How condition evaluation works
- [evaluate-conditions.md](evaluate-conditions.md) — The main evaluation system
- [is-node-active.md](is-node-active.md) — NodeActive condition evaluator
- [is-shield-active.md](is-shield-active.md) — ShieldActive condition evaluator
- [is-combo-active.md](is-combo-active.md) — ComboActive condition evaluator
