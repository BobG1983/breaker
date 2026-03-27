# TimeExpires(f32)

**Scope**: Special (timer system)

Timer-based removal for Until nodes. Not used in When nodes directly — it's the until-trigger for timed buffs.

The timer system ticks Until entries in StagedEffects that are waiting for TimeExpires, decrementing the remaining time each frame. When it reaches zero, the Until's Reverse fires.

Example: `Until(trigger: TimeExpires(2.0), then: [Do(SpeedBoost(multiplier: 1.3))])` — the speed boost lasts 2 seconds.
