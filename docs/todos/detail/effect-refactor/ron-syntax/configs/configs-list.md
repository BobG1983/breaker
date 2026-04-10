# Configs List

| Name | RON Syntax | Description |
|------|-----------|-------------|
| ShockwaveConfig | `Shockwave(base_range: 64.0, range_per_level: 8.0, stacks: 1, speed: 500.0)` | [Expanding ring that damages cells it passes through](shockwave-config.md) |
| ExplodeConfig | `Explode(range: 48.0, damage: 10.0)` | [Instant area damage at a position](explode-config.md) |
| ChainLightningConfig | `ChainLightning(arcs: 3, range: 80.0, damage_mult: 0.5, arc_speed: 200.0)` | [Sequential lightning arcs jumping between cells](chain-lightning-config.md) |
| PiercingBeamConfig | `PiercingBeam(damage_mult: 0.5, width: 16.0)` | [Instant beam along bolt velocity damaging all cells in its path](piercing-beam-config.md) |
| PulseConfig | `Pulse(base_range: 32.0, range_per_level: 4.0, stacks: 1, speed: 300.0, interval: 0.5)` | [Periodic shockwave emitter attached to a bolt](pulse-config.md) |
| ShieldConfig | `Shield(duration: 3.0, reflection_cost: 0.5)` | [Temporary floor wall that reflects bolts](shield-config.md) |
| SpawnBoltsConfig | `SpawnBolts(count: 2, lifespan: Some(5.0), inherit: false)` | [Spawn extra bolts from a bolt's position](spawn-bolts-config.md) |
| SpawnPhantomConfig | `SpawnPhantom(duration: 3.0, max_active: 2)` | [Spawn infinite-piercing phantom bolt with timed lifespan](spawn-phantom-config.md) |
| ChainBoltConfig | `ChainBolt(tether_distance: 120.0)` | [Spawn a tethered bolt pair connected by a distance constraint](chain-bolt-config.md) |
| MirrorConfig | `MirrorProtocol(inherit: false)` | [Spawn a bolt traveling in the reflected direction](mirror-config.md) |
| TetherBeamConfig | `TetherBeam(damage_mult: 0.5, chain: false)` | [Damaging beam between two connected bolts](tether-beam-config.md) |
| GravityWellConfig | `GravityWell(strength: 100.0, duration: 5.0, radius: 80.0, max: 3)` | [Stationary well that bends bolt trajectories toward it](gravity-well-config.md) |
| AttractionConfig | `Attraction(attraction_type: Cell, force: 100.0, max_force: Some(50.0))` | [Steer a bolt toward the nearest entity of a given type](attraction-config.md) |
| AnchorConfig | `Anchor(bump_force_multiplier: 2.0, perfect_window_multiplier: 1.5, plant_delay: 0.5)` | [Plant the breaker for boosted bump force and timing window](anchor-config.md) |
| CircuitBreakerConfig | `CircuitBreaker(bumps_required: 5, spawn_count: 2, inherit: false, shockwave_range: 64.0, shockwave_speed: 500.0)` | [Accumulate bumps then burst-fire bolts and a shockwave](circuit-breaker-config.md) |
| EntropyConfig | `EntropyEngine(max_effects: 3, pool: [(0.5, Shockwave(...)), (0.3, Explode(...)), (0.2, SpawnBolts(...))])` | [Escalating random effect selection from a weighted pool](entropy-config.md) |
