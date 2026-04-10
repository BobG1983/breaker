# EffectPlugin

## What it is

A Bevy plugin that registers the entire effect system: tree walking, trigger bridges, effect tick systems, condition evaluation, and all supporting resources.

## What it does

1. Registers the `EffectSystems` system sets with ordering constraints.
2. Registers all bridge systems in `EffectSystems::Bridge`.
3. Registers all tick systems in `EffectSystems::Tick`.
4. Registers `evaluate_conditions` in `EffectSystems::Conditions`.
5. Registers reset systems on `OnEnter(NodeState)`.
6. Initializes resources: `SpawnStampRegistry`, `NodeTimerThresholdRegistry`, `ComboStreak`.
7. Registers death bridge systems (`bridge_destroyed<Cell>`, `bridge_destroyed<Bolt>`, `bridge_destroyed<Wall>`, `bridge_destroyed<Breaker>`) — these live in the effect domain because they call `walk_effects`.

## How it registers systems

Each effect config struct that has runtime systems provides a registration function:

```rust
impl ShockwaveConfig {
    pub fn register_systems(app: &mut App) {
        app.add_systems(FixedUpdate, (
            tick_shockwave,
            sync_shockwave_visual,
            apply_shockwave_damage,
            despawn_finished_shockwave,
        ).chain().in_set(EffectSystems::Tick));
    }
}
```

The plugin calls each config's `register_systems` during build:

```rust
impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        // System sets
        app.configure_sets(FixedUpdate, (
            EffectSystems::Bridge,
            EffectSystems::Tick,
            EffectSystems::Conditions,
        ));

        // Bridges
        // ... register all on_* bridge systems ...

        // Per-effect systems
        ShockwaveConfig::register_systems(app);
        ChainLightningConfig::register_systems(app);
        AnchorConfig::register_systems(app);
        // ... etc for each config with systems ...

        // Condition evaluation
        app.add_systems(FixedUpdate,
            evaluate_conditions.in_set(EffectSystems::Conditions)
        );

        // Reset systems
        app.add_systems(OnEnter(NodeState::Running), (
            reset_ramping_damage,
            reset_entropy_counter,
        ).in_set(EffectSystems::Reset));

        // Resources
        app.init_resource::<SpawnStampRegistry>();
        app.init_resource::<NodeTimerThresholdRegistry>();
        app.init_resource::<ComboStreak>();
    }
}
```

## What it does NOT do

- Does NOT register death pipeline types or systems (Hp, KilledBy, apply_damage, detect_deaths, process_despawn_requests). Those are shared/domain systems.
- Does NOT register game systems (collision, bump grading, bolt lost). Those belong to their domains.
- Does NOT register message types. Messages are registered by whoever defines them.
