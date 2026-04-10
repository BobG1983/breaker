# Triggers

| Trigger | RON Syntax | Scope | Fires On | Condition |
|---------|-----------|-------|----------|-----------|
| [`PerfectBumped`](perfect-bumped.md) | `PerfectBumped` | Local | Bolt + Breaker | Bolt bumped with perfect timing |
| [`EarlyBumped`](early-bumped.md) | `EarlyBumped` | Local | Bolt + Breaker | Bolt bumped with early timing |
| [`LateBumped`](late-bumped.md) | `LateBumped` | Local | Bolt + Breaker | Bolt bumped with late timing |
| [`Bumped`](bumped.md) | `Bumped` | Local | Bolt + Breaker | Bolt bumped with any successful timing (perfect, early, or late) |
| [`PerfectBumpOccurred`](perfect-bump-occurred.md) | `PerfectBumpOccurred` | Global | All entities | A perfect bump happened somewhere |
| [`EarlyBumpOccurred`](early-bump-occurred.md) | `EarlyBumpOccurred` | Global | All entities | An early bump happened somewhere |
| [`LateBumpOccurred`](late-bump-occurred.md) | `LateBumpOccurred` | Global | All entities | A late bump happened somewhere |
| [`BumpOccurred`](bump-occurred.md) | `BumpOccurred` | Global | All entities | Any successful bump happened somewhere |
| [`BumpWhiffOccurred`](bump-whiff-occurred.md) | `BumpWhiffOccurred` | Global | All entities | Bump timing window expired without contact |
| [`NoBumpOccurred`](no-bump-occurred.md) | `NoBumpOccurred` | Global | All entities | Bolt hit breaker with no bump input |
| [`Impacted`](impacted.md) | `Impacted(EntityKind)` | Local | Both collision participants | This entity was in a collision with an entity of the given kind |
| [`ImpactOccurred`](impact-occurred.md) | `ImpactOccurred(EntityKind)` | Global | All entities | A collision involving the given entity kind happened somewhere |
| [`Died`](died.md) | `Died` | Local | Victim only | This entity died |
| [`Killed`](killed.md) | `Killed(EntityKind)` | Local | Killer only | This entity killed an entity of the given kind |
| [`DeathOccurred`](death-occurred.md) | `DeathOccurred(EntityKind)` | Global | All entities | An entity of the given kind died somewhere |
| [`BoltLostOccurred`](bolt-lost-occurred.md) | `BoltLostOccurred` | Global | All entities | A bolt fell off the bottom |
| [`NodeStartOccurred`](node-start-occurred.md) | `NodeStartOccurred` | Global | All entities | A new node started |
| [`NodeEndOccurred`](node-end-occurred.md) | `NodeEndOccurred` | Global | All entities | The current node ended |
| [`NodeTimerThresholdOccurred`](node-timer-threshold-occurred.md) | `NodeTimerThresholdOccurred(f32)` | Global | All entities | Node timer ratio crossed the given threshold (0.0-1.0) |
| [`TimeExpires`](time-expires.md) | `TimeExpires(f32)` | Self | Owner only | Countdown of the given seconds reached zero. Internal — used by Until desugaring. |
