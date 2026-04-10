# Trigger API

A trigger is a value — a variant of the Trigger enum. It is not a system, not a component, not a message. It is data that the tree walker matches against When/Once/Until nodes.

Triggers don't do anything on their own. Bridge systems translate game events into trigger dispatches. The bridge determines scope, builds a TriggerContext, and calls the walking algorithm on each affected entity. The trigger is just the key the walker matches on.

- [scope.md](scope.md) — Local vs Global vs Self: which entities get notified
- [trigger-context.md](trigger-context.md) — How to populate TriggerContext for On() resolution
- [bridge-systems.md](bridge-systems.md) — The bridge pattern: game event → trigger dispatch
- [participant-targets.md](participant-targets.md) — When and how to create a new participant enum
