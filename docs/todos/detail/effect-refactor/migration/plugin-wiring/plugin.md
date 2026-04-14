# EffectV3Plugin

## What it is

A Bevy plugin that registers the entire effect v3 system. Each effect and trigger category owns its own registration — the plugin orchestrates the calls.

## What it does

1. Configures the `EffectV3Systems` system sets with ordering constraints (`Bridge → Tick → Conditions`).
2. Calls `Fireable::register(app)` on every effect config struct (all 30).
3. Calls `register(app)` on every trigger category.
4. Registers `evaluate_conditions` in `EffectV3Systems::Conditions`.
5. Initializes shared resource: `SpawnStampRegistry`.

## How it registers

```rust
impl Plugin for EffectV3Plugin {
    fn build(&self, app: &mut App) {
        // System set ordering: Bridge → Tick → Conditions
        app.configure_sets(FixedUpdate, (
            EffectV3Systems::Bridge,
            EffectV3Systems::Tick.after(EffectV3Systems::Bridge),
            EffectV3Systems::Conditions.after(EffectV3Systems::Tick),
        ));

        // Condition evaluation
        app.add_systems(FixedUpdate,
            evaluate_conditions.in_set(EffectV3Systems::Conditions)
        );

        // Shared resources
        app.init_resource::<SpawnStampRegistry>();

        // Triggers — each category registers its own bridges and game systems
        triggers::bump::register::register(app);
        triggers::impact::register::register(app);
        triggers::death::register::register(app);
        triggers::bolt_lost::register::register(app);
        triggers::node::register::register(app);
        triggers::time::register::register(app);

        // Effects — each config registers its own systems via Fireable::register.
        // All 30 are called even if their register() is a no-op, so that adding
        // systems later cannot be silently dropped.
        effects::AnchorConfig::register(app);
        effects::AttractionConfig::register(app);
        effects::BumpForceConfig::register(app);
        effects::ChainBoltConfig::register(app);
        effects::ChainLightningConfig::register(app);
        effects::CircuitBreakerConfig::register(app);
        effects::DamageBoostConfig::register(app);
        effects::DieConfig::register(app);
        effects::EntropyConfig::register(app);
        effects::ExplodeConfig::register(app);
        effects::FlashStepConfig::register(app);
        effects::GravityWellConfig::register(app);
        effects::LoseLifeConfig::register(app);
        effects::MirrorConfig::register(app);
        effects::PiercingConfig::register(app);
        effects::PiercingBeamConfig::register(app);
        effects::PulseConfig::register(app);
        effects::QuickStopConfig::register(app);
        effects::RampingDamageConfig::register(app);
        effects::RandomEffectConfig::register(app);
        effects::SecondWindConfig::register(app);
        effects::ShieldConfig::register(app);
        effects::ShockwaveConfig::register(app);
        effects::SizeBoostConfig::register(app);
        effects::SpawnBoltsConfig::register(app);
        effects::SpawnPhantomConfig::register(app);
        effects::SpeedBoostConfig::register(app);
        effects::TetherBeamConfig::register(app);
        effects::TimePenaltyConfig::register(app);
        effects::VulnerableConfig::register(app);
    }
}
```

Most effect configs use the default no-op `register` (passive effects, fire-and-forget). Only configs with runtime systems override it. The plugin calls all 30 regardless — the no-ops compile away.

Reset systems (e.g. `reset_entropy_counter`, `reset_ramping_damage`) are registered into `EffectV3Systems::Reset` and scheduled on `OnEnter(NodeState::Loading)` — they are NOT part of the `FixedUpdate` ordering chain.

Each trigger `register` function handles its own bridges, game systems, resources, and messages. Adding a new trigger category means adding one line here.

Spawned effect entities (shockwaves, chain lightning arcs, tether beams, gravity wells, etc.) that need automatic cleanup on node exit are tagged with `CleanupOnExit::<NodeState>` from `rantzsoft_stateflow`. This is done inside each effect's `fire()` method, not in the plugin.

## What it does NOT do

- Does NOT initialize a `ComboStreak` resource — combo tracking lives in `HighlightTracker.consecutive_perfect_bumps` (run domain).
- Does NOT register death pipeline types or systems (Hp, KilledBy, apply_damage, detect_deaths, process_despawn_requests). Those are shared/domain systems.
- Does NOT register game systems (collision, bump grading, bolt lost). Those belong to their domains.
- Does NOT register message types. Messages are registered by whoever defines them.
- Does NOT list individual bridge or tick systems. That is each effect's and trigger's responsibility.
