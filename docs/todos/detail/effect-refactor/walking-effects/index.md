# Walking Effects

How to implement effect tree walking.

- [walking-algorithm.md](walking-algorithm.md) — The outer loop: StagedEffects then BoundEffects, when to call command extensions
- [fire.md](fire.md) — Evaluate Fire(EffectType)
- [when.md](when.md) — Evaluate When(Trigger, Tree)
- [once.md](once.md) — Evaluate Once(Trigger, Tree)
- [during.md](during.md) — Evaluate During(Condition, ScopedTree)
- [until.md](until.md) — Evaluate Until(Trigger, ScopedTree)
- [sequence.md](sequence.md) — Evaluate Sequence(Vec<Terminal>)
- [on.md](on.md) — Evaluate On(ParticipantTarget, Terminal)
- [route.md](route.md) — Evaluate Route(RouteType, Tree)
- [arming-effects.md](arming-effects.md) — What "arming" means
