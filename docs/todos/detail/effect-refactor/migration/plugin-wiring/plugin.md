# EffectPlugin

## What it is

A Bevy plugin that registers the entire effect system. Each effect and trigger category owns its own registration — the plugin orchestrates the calls.

## What it does

1. Configures the `EffectSystems` system sets with ordering constraints.
2. Calls `Fireable::register(app)` on every effect config struct.
3. Calls `register(app)` on every trigger category.
4. Registers `evaluate_conditions` in `EffectSystems::Conditions`.
5. Initializes shared resources: `SpawnStampRegistry`, `ComboStreak`.

## How it registers

```rust
impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        // System sets
        app.configure_sets(FixedUpdate, (
            EffectSystems::Bridge,
            EffectSystems::Tick,
            EffectSystems::Conditions,
        ));

        // Effects — each config registers its own systems via Fireable::register
        SpeedBoostConfig::register(app);
        SizeBoostConfig::register(app);
        DamageBoostConfig::register(app);
        BumpForceConfig::register(app);
        QuickStopConfig::register(app);
        VulnerableConfig::register(app);
        LoseLifeConfig::register(app);
        TimePenaltyConfig::register(app);
        DieConfig::register(app);
        SpawnBoltsConfig::register(app);
        ChainBoltConfig::register(app);
        MirrorConfig::register(app);
        RandomEffectConfig::register(app);
        ExplodeConfig::register(app);
        PiercingBeamConfig::register(app);
        ShockwaveConfig::register(app);
        ChainLightningConfig::register(app);
        AnchorConfig::register(app);
        AttractionConfig::register(app);
        PulseConfig::register(app);
        ShieldConfig::register(app);
        SecondWindConfig::register(app);
        FlashStepConfig::register(app);
        CircuitBreakerConfig::register(app);
        EntropyConfig::register(app);
        GravityWellConfig::register(app);
        SpawnPhantomConfig::register(app);
        TetherBeamConfig::register(app);
        PiercingConfig::register(app);
        RampingDamageConfig::register(app);

        // Triggers — each category registers its own bridges and game systems
        triggers::bump::register(app);
        triggers::impact::register(app);
        triggers::death::register(app);
        triggers::bolt_lost::register(app);
        triggers::node::register(app);
        triggers::time::register(app);

        // Condition evaluation
        app.add_systems(FixedUpdate,
            evaluate_conditions.in_set(EffectSystems::Conditions)
        );

        // Shared resources
        app.init_resource::<SpawnStampRegistry>();
        app.init_resource::<ComboStreak>();
    }
}
```

Most effect configs use the default no-op `register` (passive effects, fire-and-forget). Only configs with runtime systems override it. The plugin calls all 30 regardless — the no-ops compile away.

Each trigger `register` function handles its own bridges, game systems, resources, and messages. Adding a new trigger category means adding one line here.

## What it does NOT do

- Does NOT register death pipeline types or systems (Hp, KilledBy, apply_damage, detect_deaths, process_despawn_requests). Those are shared/domain systems.
- Does NOT register game systems (collision, bump grading, bolt lost). Those belong to their domains.
- Does NOT register message types. Messages are registered by whoever defines them.
- Does NOT list individual bridge or tick systems. That is each effect's and trigger's responsibility.
