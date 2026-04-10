# Trigger Context

TriggerContext carries the entities involved in a trigger event so that On() nodes can resolve ParticipantTargets during tree walking. See [rust-types/trigger-context.md](../../rust-types/trigger-context.md) for the type definition.

## Building the right variant

Each trigger category has its own TriggerContext variant. The bridge system constructs it from the game event message fields.

| Trigger category | TriggerContext variant | Fields |
|-----------------|----------------------|--------|
| Bump triggers | `Bump { bolt, breaker }` | Both entities from the bump event |
| Impact triggers | `Impact { impactor, impactee }` | Both entities from the collision |
| Death triggers | `Death { victim, killer }` | Victim always present. Killer is Option — None for environmental deaths. |
| BoltLost | `BoltLost { bolt, breaker }` | The lost bolt and the breaker that lost it |
| Node lifecycle, Time | `None` | No participants |

## Rules

- DO use the variant that matches the trigger category. A bump event always produces `Bump { ... }`, never `Impact { ... }`.
- DO populate all fields. If a field is optional (killer in Death), set it to None explicitly rather than using the wrong variant.
- DO pass the same TriggerContext to every entity walked in a single dispatch. All entities see the same participants.
- DO NOT construct None when participants exist. Even if you think no On() nodes will be encountered, provide the participants.

## New trigger categories

If your trigger doesn't fit any existing category, add a new variant to TriggerContext. See [participant-targets.md](participant-targets.md) for when this is needed.
