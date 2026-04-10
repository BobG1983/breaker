# Folder Structure

Complete directory tree for `src/effect/` post-refactor. Every file references the doc that specifies its contents. Test modules are omitted.

```
src/effect/
  mod.rs                                # plugin registration, pub(crate) mod declarations
  plugin.rs                             # EffectPlugin, system sets, system registration
                                        #   see migration/plugin-wiring/

  # ── Tree types (RON-deserializable) ────────────────────────────────────────
  types/
    mod.rs                              # re-exports for all tree type enums
    root_node.rs                        # RootNode enum
                                        #   see rust-types/root-node.md
    tree.rs                             # Tree enum
                                        #   see rust-types/tree.md
    scoped_tree.rs                      # ScopedTree enum
                                        #   see rust-types/scoped-tree.md
    terminal.rs                         # Terminal enum
                                        #   see rust-types/terminal.md
    scoped_terminal.rs                  # ScopedTerminal enum
                                        #   see rust-types/scoped-terminal.md
    trigger.rs                          # Trigger enum
                                        #   see rust-types/enums/trigger.md
    trigger_context.rs                  # TriggerContext enum (runtime only, not Deserialize)
                                        #   see rust-types/trigger-context.md
    condition.rs                        # Condition enum
                                        #   see rust-types/enums/condition.md
    stamp_target.rs                     # StampTarget enum
                                        #   see rust-types/enums/stamp-target.md
    effect_type.rs                      # EffectType enum
                                        #   see rust-types/enums/effect-type.md
    reversible_effect_type.rs           # ReversibleEffectType enum
                                        #   see rust-types/enums/reversible-effect-type.md
    entity_kind.rs                      # EntityKind enum
                                        #   see rust-types/enums/entity-kind.md
    bump_status.rs                      # BumpStatus enum
                                        #   see rust-types/enums/bump-status.md
    attraction_type.rs                  # AttractionType enum
                                        #   see rust-types/enums/attraction-type.md
    route_type.rs                       # RouteType enum
                                        #   see rust-types/enums/route-type.md
    participants.rs                     # ParticipantTarget, BumpTarget, ImpactTarget,
                                        #   DeathTarget, BoltLostTarget
                                        #   see rust-types/enums/participants/

  # ── Config structs (RON-deserializable, implement Fireable/Reversible) ─────
  configs/
    mod.rs                              # re-exports for all config structs
                                        #   see rust-types/configs/index.md
    speed_boost.rs                      # SpeedBoostConfig + Fireable + Reversible + PassiveEffect
                                        #   see migration/new-effect-implementations/speed-boost.md
                                        #   see rust-types/configs/speed-boost-config.md
    size_boost.rs                       # SizeBoostConfig + Fireable + Reversible + PassiveEffect
                                        #   see migration/new-effect-implementations/size-boost.md
                                        #   see rust-types/configs/size-boost-config.md
    damage_boost.rs                     # DamageBoostConfig + Fireable + Reversible + PassiveEffect
                                        #   see migration/new-effect-implementations/damage-boost.md
                                        #   see rust-types/configs/damage-boost-config.md
    bump_force.rs                       # BumpForceConfig + Fireable + Reversible + PassiveEffect
                                        #   see migration/new-effect-implementations/bump-force.md
                                        #   see rust-types/configs/bump-force-config.md
    quick_stop.rs                       # QuickStopConfig + Fireable + Reversible + PassiveEffect
                                        #   see migration/new-effect-implementations/quick-stop.md
                                        #   see rust-types/configs/quick-stop-config.md
    flash_step.rs                       # FlashStepConfig + Fireable + Reversible
                                        #   see migration/new-effect-implementations/flash-step.md
                                        #   see rust-types/configs/flash-step-config.md
    piercing.rs                         # PiercingConfig + Fireable + Reversible + PassiveEffect
                                        #   see migration/new-effect-implementations/piercing.md
                                        #   see rust-types/configs/piercing-config.md
    vulnerable.rs                       # VulnerableConfig + Fireable + Reversible + PassiveEffect
                                        #   see migration/new-effect-implementations/vulnerable.md
                                        #   see rust-types/configs/vulnerable-config.md
    ramping_damage.rs                   # RampingDamageConfig + Fireable + Reversible + PassiveEffect
                                        #   see migration/new-effect-implementations/ramping-damage.md
                                        #   see rust-types/configs/ramping-damage-config.md
    attraction.rs                       # AttractionConfig + Fireable + Reversible
                                        #   see migration/new-effect-implementations/attraction.md
                                        #   see rust-types/configs/attraction-config.md
    anchor.rs                           # AnchorConfig + Fireable + Reversible
                                        #   see migration/new-effect-implementations/anchor.md
                                        #   see rust-types/configs/anchor-config.md
    pulse.rs                            # PulseConfig + Fireable + Reversible
                                        #   see migration/new-effect-implementations/pulse.md
                                        #   see rust-types/configs/pulse-config.md
    shield.rs                           # ShieldConfig + Fireable + Reversible
                                        #   see migration/new-effect-implementations/shield.md
                                        #   see rust-types/configs/shield-config.md
    second_wind.rs                      # SecondWindConfig + Fireable + Reversible
                                        #   see migration/new-effect-implementations/second-wind.md
                                        #   see rust-types/configs/second-wind-config.md
    shockwave.rs                        # ShockwaveConfig + Fireable
                                        #   see migration/new-effect-implementations/shockwave.md
                                        #   see rust-types/configs/shockwave-config.md
    explode.rs                          # ExplodeConfig + Fireable
                                        #   see migration/new-effect-implementations/explode.md
                                        #   see rust-types/configs/explode-config.md
    chain_lightning.rs                  # ChainLightningConfig + Fireable
                                        #   see migration/new-effect-implementations/chain-lightning.md
                                        #   see rust-types/configs/chain-lightning-config.md
    piercing_beam.rs                    # PiercingBeamConfig + Fireable
                                        #   see migration/new-effect-implementations/piercing-beam.md
                                        #   see rust-types/configs/piercing-beam-config.md
    spawn_bolts.rs                      # SpawnBoltsConfig + Fireable
                                        #   see migration/new-effect-implementations/spawn-bolts.md
                                        #   see rust-types/configs/spawn-bolts-config.md
    spawn_phantom.rs                    # SpawnPhantomConfig + Fireable
                                        #   see migration/new-effect-implementations/spawn-phantom.md
                                        #   see rust-types/configs/spawn-phantom-config.md
    chain_bolt.rs                       # ChainBoltConfig + Fireable
                                        #   see migration/new-effect-implementations/chain-bolt.md
                                        #   see rust-types/configs/chain-bolt-config.md
    mirror_protocol.rs                  # MirrorConfig + Fireable
                                        #   see migration/new-effect-implementations/mirror-protocol.md
                                        #   see rust-types/configs/mirror-config.md
    tether_beam.rs                      # TetherBeamConfig + Fireable
                                        #   see migration/new-effect-implementations/tether-beam.md
                                        #   see rust-types/configs/tether-beam-config.md
    gravity_well.rs                     # GravityWellConfig + Fireable
                                        #   see migration/new-effect-implementations/gravity-well.md
                                        #   see rust-types/configs/gravity-well-config.md
    lose_life.rs                        # LoseLifeConfig + Fireable
                                        #   see migration/new-effect-implementations/lose-life.md
                                        #   see rust-types/configs/lose-life-config.md
    time_penalty.rs                     # TimePenaltyConfig + Fireable
                                        #   see migration/new-effect-implementations/time-penalty.md
                                        #   see rust-types/configs/time-penalty-config.md
    die.rs                              # DieConfig + Fireable
                                        #   see migration/new-effect-implementations/die.md
                                        #   see rust-types/configs/die-config.md
    circuit_breaker.rs                  # CircuitBreakerConfig + Fireable + Reversible
                                        #   see migration/new-effect-implementations/circuit-breaker.md
                                        #   see rust-types/configs/circuit-breaker-config.md
    entropy_engine.rs                   # EntropyConfig + Fireable + Reversible
                                        #   see migration/new-effect-implementations/entropy-engine.md
                                        #   see rust-types/configs/entropy-config.md
    random_effect.rs                    # RandomEffectConfig + Fireable
                                        #   see migration/new-effect-implementations/random-effect.md
                                        #   see rust-types/configs/random-effect-config.md

  # ── Traits ─────────────────────────────────────────────────────────────────
  traits/
    mod.rs                              # re-exports for all trait definitions
    fireable.rs                         # Fireable trait
                                        #   see rust-types/fireable.md
                                        #   see creating-effects/effect-api/fireable.md
    reversible.rs                       # Reversible trait
                                        #   see rust-types/reversible.md
                                        #   see creating-effects/effect-api/reversible.md
    passive_effect.rs                   # PassiveEffect trait
                                        #   see rust-types/effect-stacking/passive-effect.md
                                        #   see creating-effects/effect-api/passive-effect.md

  # ── Effect stacking (generic + concrete instances) ─────────────────────────
  stacking/
    mod.rs                              # re-exports EffectStack<T>
    effect_stack.rs                     # EffectStack<T> generic container
                                        #   see rust-types/effect-stacking/effect-stack.md
                                        #   see rust-types/effect-stacking/index.md

  # ── Storage components ─────────────────────────────────────────────────────
  storage/
    mod.rs                              # re-exports for storage types
    bound_effects.rs                    # BoundEffects component
                                        #   see storing-effects/bound-effects.md
    staged_effects.rs                   # StagedEffects component
                                        #   see storing-effects/staged-effects.md

  # ── Command extensions ─────────────────────────────────────────────────────
  commands/
    mod.rs                              # re-exports for command extension types
    ext.rs                              # EffectCommandsExt trait + impls
                                        #   see command-extensions/index.md
                                        #   see command-extensions/why-an-extension.md
    fire.rs                             # FireEffectCommand
                                        #   see command-extensions/fire-effect.md
    reverse.rs                          # ReverseEffectCommand
                                        #   see command-extensions/reverse-effect.md
    route.rs                            # RouteEffectCommand
                                        #   see command-extensions/route-effect.md
    stamp.rs                            # StampEffectCommand
                                        #   see command-extensions/stamp-effect.md

  # ── Tree walking ───────────────────────────────────────────────────────────
  walking/
    mod.rs                              # re-exports walk_effects entry point
    walk_effects.rs                     # walk_effects function (outer loop)
                                        #   see walking-effects/walking-algorithm.md
    fire.rs                             # Fire node evaluation
                                        #   see walking-effects/fire.md
    when.rs                             # When node evaluation
                                        #   see walking-effects/when.md
    once.rs                             # Once node evaluation
                                        #   see walking-effects/once.md
    during.rs                           # During node evaluation
                                        #   see walking-effects/during.md
    until.rs                            # Until node evaluation
                                        #   see walking-effects/until.md
    sequence.rs                         # Sequence node evaluation
                                        #   see walking-effects/sequence.md
    on.rs                               # On node evaluation
                                        #   see walking-effects/on.md
    route.rs                            # Route node evaluation
                                        #   see walking-effects/route.md

  # ── Dispatch (match EffectType -> config.fire/reverse) ─────────────────────
  dispatch/
    mod.rs                              # re-exports fire_dispatch + reverse_dispatch
    fire_dispatch.rs                    # match EffectType -> config.fire()
                                        #   see creating-effects/wiring-an-effect.md
    reverse_dispatch.rs                 # match ReversibleEffectType -> config.reverse()
                                        #   see creating-effects/wiring-an-effect.md

  # ── Bridge systems (game event -> trigger -> walk) ─────────────────────────
  bridges/
    mod.rs                              # re-exports for all bridge systems
                                        #   see creating-triggers/trigger-api/bridge-systems.md
    bump.rs                             # 10 bump bridges (4 local + 6 global)
                                        #   see migration/new-trigger-implementations/bump/
                                        #   see dispatching-triggers/bump/
    impact.rs                           # 2 impact bridges (1 local + 1 global)
                                        #   see migration/new-trigger-implementations/impact/
                                        #   see dispatching-triggers/impact/
    bolt_lost.rs                        # bolt lost bridge (1 global)
                                        #   see migration/new-trigger-implementations/bolt-lost/
                                        #   see dispatching-triggers/bolt-lost/
    node.rs                             # 3 node bridges (node start, node end, threshold)
                                        #   see migration/new-trigger-implementations/node/
                                        #   see dispatching-triggers/node/
    time.rs                             # time expires bridge
                                        #   see migration/new-trigger-implementations/time/
                                        #   see dispatching-triggers/time/

  # ── Effect-specific runtime components ─────────────────────────────────────
  components/
    mod.rs                              # re-exports for all effect runtime components
                                        #   see rust-types/components/index.md
    effect_source_chip.rs               # EffectSourceChip component
                                        #   see rust-types/components/effect-source-chip.md
    effect_timers.rs                    # EffectTimers component
                                        #   see rust-types/components/effect-timers.md
    shockwave.rs                        # Shockwave* components (ShockwaveSource, etc.)
                                        #   see rust-types/components/shockwave.md
    chain_lightning.rs                  # ChainLightning* components
                                        #   see rust-types/components/chain-lightning.md
    anchor.rs                           # Anchor* components (AnchorActive, etc.)
                                        #   see rust-types/components/anchor.md
    attraction.rs                       # ActiveAttractions component
                                        #   see rust-types/components/attraction.md
    circuit_breaker.rs                  # CircuitBreakerCounter component
                                        #   see rust-types/components/circuit-breaker.md
    entropy_engine.rs                   # EntropyCounter component
                                        #   see rust-types/components/entropy-engine.md
    flash_step.rs                       # FlashStepActive component
                                        #   see rust-types/components/flash-step.md
    gravity_well.rs                     # GravityWell* components
                                        #   see rust-types/components/gravity-well.md
    piercing.rs                         # PiercingRemaining component
                                        #   see rust-types/components/piercing-remaining.md
    pulse.rs                            # PulseEmitter component
                                        #   see rust-types/components/pulse-emitter.md
    ramping_damage.rs                   # RampingDamageAccumulator component
                                        #   see rust-types/components/ramping-damage-accumulator.md
    shield.rs                           # Shield* components
                                        #   see rust-types/components/shield.md
    second_wind.rs                      # SecondWind* components
                                        #   see rust-types/components/second-wind.md
    phantom_bolt.rs                     # Phantom* components
                                        #   see rust-types/components/phantom-bolt.md
    tether_beam.rs                      # TetherBeam* components
                                        #   see rust-types/components/tether-beam.md

  # ── Effect-specific runtime systems (tick, damage, cleanup) ────────────────
  systems/
    mod.rs                              # re-exports + system registration helpers
    tick_shockwave.rs                   # tick shockwave expansion
                                        #   see migration/new-effect-implementations/shockwave.md
    sync_shockwave_visual.rs            # sync shockwave visual to radius
                                        #   see migration/new-effect-implementations/shockwave.md
    apply_shockwave_damage.rs           # apply shockwave damage to overlapping entities
                                        #   see migration/new-effect-implementations/shockwave.md
    despawn_finished_shockwave.rs       # despawn shockwaves that have reached max radius
                                        #   see migration/new-effect-implementations/shockwave.md
    tick_chain_lightning.rs             # tick chain lightning arc propagation
                                        #   see migration/new-effect-implementations/chain-lightning.md
    tick_anchor.rs                      # tick anchor lock/unlock logic
                                        #   see migration/new-effect-implementations/anchor.md
    apply_attraction.rs                 # apply attraction forces to bolts
                                        #   see migration/new-effect-implementations/attraction.md
    tick_pulse.rs                       # tick pulse emitter cooldown and fire
                                        #   see migration/new-effect-implementations/pulse.md
    tick_shield_duration.rs             # tick shield lifetime countdown
                                        #   see migration/new-effect-implementations/shield.md
    tick_phantom_lifetime.rs            # tick phantom bolt lifetime countdown
                                        #   see migration/new-effect-implementations/spawn-phantom.md
    tick_tether_beam_damage.rs          # tick tether beam damage application
                                        #   see migration/new-effect-implementations/tether-beam.md
    cleanup_tether_beams.rs             # cleanup tether beams whose target is gone
                                        #   see migration/new-effect-implementations/tether-beam.md
    tick_gravity_wells.rs               # tick gravity well force application
                                        #   see migration/new-effect-implementations/gravity-well.md
    despawn_expired_wells.rs            # despawn gravity wells past their duration
                                        #   see migration/new-effect-implementations/gravity-well.md
    tick_effect_timers.rs               # tick EffectTimers, emit EffectTimerExpired messages
                                        #   see migration/new-trigger-implementations/time/tick-effect-timers.md
    check_node_timer_thresholds.rs      # check node timer against threshold registry
                                        #   see migration/new-trigger-implementations/node/check-node-timer-thresholds.md
    reset_ramping_damage.rs             # reset RampingDamageAccumulator on node start
                                        #   see migration/new-effect-implementations/ramping-damage.md
    reset_entropy_counter.rs            # reset EntropyCounter on node start
                                        #   see migration/new-effect-implementations/entropy-engine.md

  # ── Messages ───────────────────────────────────────────────────────────────
  messages/
    mod.rs                              # re-exports for all effect messages
                                        #   see rust-types/messages/index.md
    effect_timer_expired.rs             # EffectTimerExpired message
                                        #   see rust-types/messages/effect-timer-expired.md
    node_timer_threshold_crossed.rs     # NodeTimerThresholdCrossed message
                                        #   see rust-types/messages/node-timer-threshold-crossed.md

  # ── Resources ──────────────────────────────────────────────────────────────
  resources/
    mod.rs                              # re-exports for all effect resources
                                        #   see rust-types/resources/index.md
    spawn_stamp_registry.rs             # SpawnStampRegistry resource
                                        #   see storing-effects/spawn-stamp-registry.md
    node_timer_threshold_registry.rs    # NodeTimerThresholdRegistry resource
                                        #   see rust-types/resources/node-timer-threshold-registry.md
```

All `see` paths are relative to `docs/todos/detail/effect-refactor/`.

Death bridge systems (`bridge_destroyed<T>`) are NOT in this plugin. They are registered by the unified death pipeline plugin -- see `docs/architecture/effects/death_pipeline.md` and `docs/todos/detail/effect-refactor/dispatching-triggers/death/`.
