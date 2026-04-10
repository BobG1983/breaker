# DeathPipelinePlugin

## What it is

A Bevy plugin that registers the unified damage → death → despawn pipeline. Replaces the per-domain ad-hoc death handling with a generic, consistent chain.

## What it does

1. Configures the `DeathPipelineSystems` system sets with ordering constraints.
2. Registers `apply_damage::<T>` for each entity type in `DeathPipelineSystems::ApplyDamage`.
3. Registers `detect_*_deaths` for each entity type in `DeathPipelineSystems::DetectDeaths`.
4. Registers `process_despawn_requests` in PostFixedUpdate.

## How it registers

```rust
impl Plugin for DeathPipelinePlugin {
    fn build(&self, app: &mut App) {
        // System sets — ApplyDamage before DetectDeaths
        app.configure_sets(FixedUpdate, (
            DeathPipelineSystems::ApplyDamage
                .after(EffectSystems::Tick),
            DeathPipelineSystems::DetectDeaths
                .after(DeathPipelineSystems::ApplyDamage),
        ));

        // Apply damage — generic system, one per entity type
        app.add_systems(FixedUpdate, (
            apply_damage::<Cell>,
            apply_damage::<Bolt>,
            apply_damage::<Wall>,
            apply_damage::<Breaker>,
        ).in_set(DeathPipelineSystems::ApplyDamage));

        // Detect deaths — per-domain systems
        app.add_systems(FixedUpdate, (
            detect_cell_deaths,
            detect_bolt_deaths,
            detect_wall_deaths,
            detect_breaker_deaths,
        ).in_set(DeathPipelineSystems::DetectDeaths));

        // Deferred despawn — runs after all FixedUpdate processing
        app.add_systems(PostFixedUpdate, process_despawn_requests);
    }
}
```

## What it does NOT do

- Does NOT register domain kill handlers. Each domain plugin registers its own handler that reads `KillYourself<T>` and sends `Destroyed<T>` + `DespawnEntity`.
- Does NOT register death bridge systems (`on_destroyed::<T>`). Those live in the effect plugin — they call `walk_effects`, which is an effect concern.
- Does NOT register the `GameEntity` trait or Hp/KilledBy components. Those are shared types that exist independently.
- Does NOT handle visual feedback (damage flash, death animation). That is the fx domain's responsibility.
- Does NOT handle node completion tracking (counting cleared cells). That is the run domain's responsibility, reading `Destroyed<Cell>` or similar.
