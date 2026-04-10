# TimeExpires Trigger

## Trigger
- `TimeExpires(f32)` — countdown timer that fires when remaining reaches zero

## Locality: SELF-CONSUMING
Not local or global — lives in StagedEffects on a specific entity, consumed when it fires.

## Source
Internal timer system that ticks down `TimeExpires` entries in StagedEffects each FixedUpdate tick.

## Timer System Behavior
```
fn tick_time_expires(time: Res<Time>, mut query: Query<&mut StagedEffects>) {
    let dt = time.delta_secs();
    for mut staged in &mut query {
        // Find TimeExpires entries, tick them down
        for entry in &mut staged.entries {
            if let Trigger::TimeExpires(ref mut remaining) = entry.trigger {
                *remaining -= dt;
            }
        }
        
        // Collect expired entries (remaining <= 0)
        let expired: Vec<_> = staged.entries
            .drain_filter(|e| matches!(e.trigger, Trigger::TimeExpires(r) if r <= 0.0))
            .collect();
        
        // Execute each expired entry's tree
        for entry in expired {
            execute_tree(&entry.tree, entity, ...);
        }
    }
}
```

## How TimeExpires Gets Into StagedEffects
`TimeExpires` is always created by `Until` desugaring:
```
Until(TimeExpires(3.0), Fire(SpeedBoost(1.5)))
```
Desugars to:
1. Fire SpeedBoost immediately
2. Insert `StagedEntry { trigger: TimeExpires(3.0), tree: Reverse(SpeedBoost(1.5)) }` into StagedEffects

When 3 seconds pass, the timer system fires and the Reverse executes.

## Notes
- TimeExpires is the only trigger that mutates StagedEffects entries (ticking down the f32)
- Only appears in StagedEffects, never in BoundEffects (it's always one-shot)
- The f32 value is the remaining seconds, decremented each tick
- Consumed (removed from StagedEffects) when it fires
- This is why Trigger needs to be mutable — the timer tick system modifies the f32 in-place
