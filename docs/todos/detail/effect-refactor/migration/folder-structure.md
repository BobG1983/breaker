# Folder Structure

Complete directory tree for `src/effect_v3/` post-refactor.

## Principles

- **Co-location**: everything for one effect lives in one folder — config, components, systems.
- **Co-location**: everything for one trigger category lives in one folder — bridges, game systems, resources, messages.
- Every effect gets its own folder under `effects/`, even simple ones.
- Every trigger category gets its own folder under `triggers/`.
- Cross-cutting infrastructure stays shared.

## Tree

```
src/effect_v3/
  mod.rs
  plugin.rs

  # ── Shared infrastructure ──────────────────────────────────────────────────

  types/
    mod.rs
    root_node.rs                    # RootNode enum
    tree.rs                         # Tree enum
    scoped_tree.rs                  # ScopedTree enum
    terminal.rs                     # Terminal enum
    scoped_terminal.rs              # ScopedTerminal enum
    trigger.rs                      # Trigger enum
    trigger_context.rs              # TriggerContext enum
    condition.rs                    # Condition enum
    stamp_target.rs                 # StampTarget enum
    effect_type.rs                  # EffectType enum (30 variants)
    reversible_effect_type.rs       # ReversibleEffectType enum (16 variants)
    entity_kind.rs                  # EntityKind enum
    bump_status.rs                  # BumpStatus enum
    attraction_type.rs              # AttractionType enum
    route_type.rs                   # RouteType enum
    participants.rs                 # ParticipantTarget + sub-enums

  traits/
    mod.rs
    fireable.rs                     # Fireable trait
    reversible.rs                   # Reversible trait
    passive_effect.rs               # PassiveEffect trait

  stacking/
    mod.rs
    effect_stack.rs                 # EffectStack<T> generic container

  storage/
    mod.rs
    bound_effects.rs                # BoundEffects component
    staged_effects.rs               # StagedEffects component
    spawn_stamp_registry.rs         # SpawnStampRegistry resource

  commands/
    mod.rs
    ext.rs                          # EffectCommandsExt trait
    fire.rs                         # fire_effect
    reverse.rs                      # reverse_effect
    route.rs                        # route_effect
    stamp.rs                        # stamp_effect (sugar for route_effect Bound)
    stage.rs                        # stage_effect (sugar for route_effect Staged)
    remove.rs                       # remove_effect (removes from Bound or Staged)

  walking/
    mod.rs
    walk_effects.rs                 # walk_effects outer loop
    fire.rs                         # Fire node evaluation
    when.rs                         # When node evaluation
    once.rs                         # Once node evaluation
    during.rs                       # During node evaluation
    until.rs                        # Until node evaluation
    sequence.rs                     # Sequence node evaluation
    on.rs                           # On node evaluation
    route.rs                        # Route node evaluation

  dispatch/
    mod.rs
    fire_dispatch.rs                # match EffectType → config.fire()
    reverse_dispatch.rs             # match ReversibleEffectType → config.reverse()

  conditions/
    mod.rs
    evaluate_conditions.rs          # per-frame condition polling system
    node_active.rs                  # is_node_active evaluator
    shield_active.rs                # is_shield_active evaluator
    combo_active.rs                 # is_combo_active evaluator

  components/
    mod.rs
    effect_source_chip.rs           # EffectSourceChip — tracks which chip sourced an effect

  # ── Effects ────────────────────────────────────────────────────────────────
  #
  # One folder per effect. Every folder has config.rs.
  # Add components.rs if the effect has runtime components.
  # Add systems.rs if the effect has tick/update systems.

  effects/
    mod.rs

    speed_boost/
      mod.rs
      config.rs                     # SpeedBoostConfig — PassiveEffect

    size_boost/
      mod.rs
      config.rs                     # SizeBoostConfig — PassiveEffect

    damage_boost/
      mod.rs
      config.rs                     # DamageBoostConfig — PassiveEffect

    bump_force/
      mod.rs
      config.rs                     # BumpForceConfig — PassiveEffect

    quick_stop/
      mod.rs
      config.rs                     # QuickStopConfig — PassiveEffect

    vulnerable/
      mod.rs
      config.rs                     # VulnerableConfig — PassiveEffect

    lose_life/
      mod.rs
      config.rs                     # LoseLifeConfig — fire-and-forget

    time_penalty/
      mod.rs
      config.rs                     # TimePenaltyConfig — fire-and-forget

    die/
      mod.rs
      config.rs                     # DieConfig — fire-and-forget

    spawn_bolts/
      mod.rs
      config.rs                     # SpawnBoltsConfig — fire-and-forget

    chain_bolt/
      mod.rs
      config.rs                     # ChainBoltConfig — fire-and-forget

    mirror_protocol/
      mod.rs
      config.rs                     # MirrorConfig — fire-and-forget

    random_effect/
      mod.rs
      config.rs                     # RandomEffectConfig — fire-and-forget

    explode/
      mod.rs
      config.rs                     # ExplodeConfig — fire-and-forget

    piercing_beam/
      mod.rs
      config.rs                     # PiercingBeamConfig — fire-and-forget

    shockwave/
      mod.rs
      config.rs                     # ShockwaveConfig + Fireable
      components.rs                 # ShockwaveSource, ShockwaveRadius, ShockwaveMaxRadius
      systems.rs                    # tick, sync_visual, apply_damage, despawn_finished

    chain_lightning/
      mod.rs
      config.rs                     # ChainLightningConfig + Fireable
      components.rs                 # ChainLightning runtime components
      systems.rs                    # tick arc propagation

    anchor/
      mod.rs
      config.rs                     # AnchorConfig + Fireable + Reversible
      components.rs                 # AnchorActive
      systems.rs                    # tick lock/unlock

    attraction/
      mod.rs
      config.rs                     # AttractionConfig + Fireable + Reversible
      components.rs                 # ActiveAttractions
      systems.rs                    # apply attraction forces

    pulse/
      mod.rs
      config.rs                     # PulseConfig + Fireable + Reversible
      components.rs                 # PulseEmitter
      systems.rs                    # tick cooldown and fire

    shield/
      mod.rs
      config.rs                     # ShieldConfig + Fireable + Reversible
      components.rs                 # Shield runtime components
      systems.rs                    # tick duration countdown

    second_wind/
      mod.rs
      config.rs                     # SecondWindConfig + Fireable + Reversible
      components.rs                 # SecondWind runtime components

    flash_step/
      mod.rs
      config.rs                     # FlashStepConfig + Fireable + Reversible
      components.rs                 # FlashStepActive

    circuit_breaker/
      mod.rs
      config.rs                     # CircuitBreakerConfig + Fireable + Reversible
      components.rs                 # CircuitBreakerCounter

    entropy_engine/
      mod.rs
      config.rs                     # EntropyConfig + Fireable + Reversible
      components.rs                 # EntropyCounter
      systems.rs                    # reset counter on node start

    gravity_well/
      mod.rs
      config.rs                     # GravityWellConfig + Fireable
      components.rs                 # GravityWell runtime components
      systems.rs                    # tick force application, despawn expired

    phantom_bolt/
      mod.rs
      config.rs                     # SpawnPhantomConfig + Fireable
      components.rs                 # Phantom runtime components
      systems.rs                    # tick lifetime countdown

    tether_beam/
      mod.rs
      config.rs                     # TetherBeamConfig + Fireable
      components.rs                 # TetherBeam runtime components
      systems.rs                    # tick damage, cleanup dead targets

    piercing/
      mod.rs
      config.rs                     # PiercingConfig + Fireable + Reversible + PassiveEffect
      components.rs                 # PiercingRemaining

    ramping_damage/
      mod.rs
      config.rs                     # RampingDamageConfig + Fireable + Reversible + PassiveEffect
      components.rs                 # RampingDamageAccumulator
      systems.rs                    # reset accumulator on node start

  # ── Triggers ───────────────────────────────────────────────────────────────
  #
  # One folder per trigger category. Bridges and game systems co-located.

  triggers/
    mod.rs

    bump/
      mod.rs
      register.rs                   # registers all bump bridges
      bridges.rs                    # on_bumped, on_perfect_bumped, on_early_bumped,
                                    # on_late_bumped, on_bump_occurred,
                                    # on_perfect_bump_occurred, on_early_bump_occurred,
                                    # on_late_bump_occurred, on_bump_whiff_occurred,
                                    # on_no_bump_occurred

    impact/
      mod.rs
      register.rs                   # registers impact bridges
      bridges.rs                    # on_impacted (6 collision types), on_impact_occurred

    death/
      mod.rs
      register.rs                   # registers death bridges
      bridges.rs                    # on_destroyed::<Cell>, on_destroyed::<Bolt>,
                                    # on_destroyed::<Wall>, on_destroyed::<Breaker>

    bolt_lost/
      mod.rs
      register.rs                   # registers bolt_lost bridge
      bridges.rs                    # on_bolt_lost_occurred

    node/
      mod.rs
      register.rs                   # registers node bridges, check_thresholds, resources
      bridges.rs                    # on_node_start_occurred, on_node_end_occurred,
                                    # on_node_timer_threshold_occurred
      check_thresholds.rs           # check_node_timer_thresholds game system
      resources.rs                  # NodeTimerThresholdRegistry
      messages.rs                   # NodeTimerThresholdCrossed

    time/
      mod.rs
      register.rs                   # registers time bridge, tick_timers, components
      bridges.rs                    # on_time_expires
      tick_timers.rs                # tick_effect_timers game system
      components.rs                 # EffectTimers
      messages.rs                   # EffectTimerExpired
```
