# Conditions

State-based scoping for During. Unlike a Trigger (one-time event), a Condition has a start and end and can cycle.

| Condition | RON Syntax | Description |
|-----------|-----------|-------------|
| [NodeActive](node-active.md) | `NodeActive` | True while a node is playing or paused. Starts on node enter, ends on node teardown. |
| [ShieldActive](shield-active.md) | `ShieldActive` | True while at least one ShieldWall entity exists in the world. |
| [ComboActive](combo-active.md) | `ComboActive(u32)` | True while the consecutive perfect bump streak is at or above the given count. Ends when a non-perfect bump breaks the streak. |
