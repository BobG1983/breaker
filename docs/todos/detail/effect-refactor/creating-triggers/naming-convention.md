# Naming Convention

## Bridge Systems

All bridge systems use the `on_` prefix:

```
on_perfect_bumped
on_bumped
on_impacted
on_impact_occurred
on_bolt_lost_occurred
on_node_start_occurred
on_time_expires
```

The name matches the Trigger variant they dispatch, lowercased with underscores.

## Game Systems (non-bridge)

Game systems that produce messages consumed by bridges do NOT use the `on_` prefix. They use descriptive verb-noun names:

```
tick_effect_timers
check_node_timer_thresholds
```

## The Distinction

- `on_*` = bridge system. Reads a message, dispatches a trigger, calls the walker. Pure translation.
- Everything else = game system. Detects a condition, sends a message. Game logic.
