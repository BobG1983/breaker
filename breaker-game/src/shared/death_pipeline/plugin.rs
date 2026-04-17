//! `DeathPipelinePlugin` — registers the unified damage -> death -> heal -> despawn pipeline.

use bevy::prelude::*;

use super::{
    damage_dealt::DamageDealt, despawn_entity::DespawnEntity, destroyed::Destroyed,
    heal_dealt::HealDealt, kill_yourself::KillYourself, sets::DeathPipelineSystems, systems,
};
use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    cells::{behaviors::survival::salvo::components::Salvo, components::Cell},
    effect_v3::sets::EffectV3Systems,
    walls::components::Wall,
};

/// Plugin for the unified death pipeline.
///
/// Registers `DamageDealt<T>`, `HealDealt<T>`, `KillYourself<T>`, and
/// `Destroyed<T>` messages for all entity types, wires up `apply_damage<T>`,
/// `detect_deaths<T>`, `handle_kill<T>`, and `apply_heal<T>` systems in the
/// four-stage `FixedUpdate` chain `ApplyDamage -> DetectDeaths -> HandleKill
/// -> ApplyHeal`, and schedules `process_despawn_requests` in
/// `FixedPostUpdate`. `ApplyHeal` runs after `HandleKill` so the
/// `Without<Dead>` filter on `apply_heal<T>`'s query prevents same-tick
/// revival of entities that were damage-killed earlier in the tick.
pub(crate) struct DeathPipelinePlugin;

impl Plugin for DeathPipelinePlugin {
    fn build(&self, app: &mut App) {
        // Message registration — one queue per entity type per message kind
        app.add_message::<DamageDealt<Cell>>();
        app.add_message::<DamageDealt<Bolt>>();
        app.add_message::<DamageDealt<Wall>>();
        app.add_message::<DamageDealt<Breaker>>();
        app.add_message::<DamageDealt<Salvo>>();

        app.add_message::<HealDealt<Cell>>();
        app.add_message::<HealDealt<Bolt>>();
        app.add_message::<HealDealt<Wall>>();
        app.add_message::<HealDealt<Breaker>>();
        app.add_message::<HealDealt<Salvo>>();

        app.add_message::<KillYourself<Cell>>();
        app.add_message::<KillYourself<Bolt>>();
        app.add_message::<KillYourself<Wall>>();
        app.add_message::<KillYourself<Breaker>>();
        app.add_message::<KillYourself<Salvo>>();

        app.add_message::<Destroyed<Cell>>();
        app.add_message::<Destroyed<Bolt>>();
        app.add_message::<Destroyed<Wall>>();
        app.add_message::<Destroyed<Breaker>>();
        app.add_message::<Destroyed<Salvo>>();

        app.add_message::<DespawnEntity>();

        // System set ordering: ApplyDamage after effect tick, DetectDeaths after
        // ApplyDamage, HandleKill after DetectDeaths.
        app.configure_sets(
            FixedUpdate,
            (
                DeathPipelineSystems::ApplyDamage.after(EffectV3Systems::Tick),
                DeathPipelineSystems::DetectDeaths.after(DeathPipelineSystems::ApplyDamage),
                DeathPipelineSystems::HandleKill.after(DeathPipelineSystems::DetectDeaths),
                DeathPipelineSystems::ApplyHeal.after(DeathPipelineSystems::HandleKill),
            ),
        );

        // Damage application — monomorphized per entity type
        app.add_systems(
            FixedUpdate,
            (
                systems::apply_damage::<Cell>,
                systems::apply_damage::<Bolt>,
                systems::apply_damage::<Wall>,
                systems::apply_damage::<Breaker>,
                systems::apply_damage::<Salvo>,
            )
                .in_set(DeathPipelineSystems::ApplyDamage),
        );

        // Death detection — monomorphized per entity type
        app.add_systems(
            FixedUpdate,
            (
                systems::detect_deaths::<Cell>,
                systems::detect_deaths::<Bolt>,
                systems::detect_deaths::<Wall>,
                systems::detect_deaths::<Breaker>,
                systems::detect_deaths::<Salvo>,
            )
                .in_set(DeathPipelineSystems::DetectDeaths),
        );

        // Kill handling — monomorphized per entity type. Consumes
        // `KillYourself<T>`, marks `Dead`, emits `Destroyed<T>`, and
        // enqueues `DespawnEntity`.
        //
        // `Cell` and `Bolt` are the active producers today. `Wall` is wired
        // as a future-proofing measure: walls have no death producer in the
        // current game, but the generic handler is harmless — if no
        // `KillYourself<Wall>` messages are written, the system is a no-op.
        // `Breaker` is handled separately by
        // [`handle_breaker_death`](crate::state::run::node::lifecycle::systems::handle_breaker_death)
        // because the breaker must survive through the end-of-run flow and
        // therefore cannot use the generic `DespawnEntity`-emitting handler.
        app.add_systems(
            FixedUpdate,
            (
                systems::handle_kill::<Cell>,
                systems::handle_kill::<Bolt>,
                systems::handle_kill::<Wall>,
                systems::handle_kill::<Salvo>,
            )
                .in_set(DeathPipelineSystems::HandleKill),
        );

        // Heal application — monomorphized per entity type
        app.add_systems(
            FixedUpdate,
            (
                systems::apply_heal::<Cell>,
                systems::apply_heal::<Bolt>,
                systems::apply_heal::<Wall>,
                systems::apply_heal::<Breaker>,
                systems::apply_heal::<Salvo>,
            )
                .in_set(DeathPipelineSystems::ApplyHeal),
        );

        // Deferred despawn — runs after all FixedUpdate processing
        app.add_systems(FixedPostUpdate, systems::process_despawn_requests);
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use bevy::prelude::*;
    use rantzsoft_spatial2d::components::Position2D;

    use super::*;
    use crate::shared::{
        death_pipeline::{
            dead::Dead, game_entity::GameEntity, heal_dealt::HealCap, hp::Hp, killed_by::KilledBy,
        },
        test_utils::{MessageCollector, attach_message_capture, tick},
    };

    // Local test entity — `TestEntity` in `systems::tests::helpers` is
    // `pub(super)` and unreachable from this module (different subtree).
    #[derive(Component)]
    struct PluginTestEntity;
    impl GameEntity for PluginTestEntity {}

    /// Builds a test app with `DeathPipelinePlugin` plus `PluginTestEntity`
    /// monomorphizations of all four death-pipeline systems (`apply_damage`,
    /// `detect_deaths`, `handle_kill`, `apply_heal`) wired with the same
    /// `DeathPipelineSystems::*` annotations the plugin uses for production
    /// types.
    fn build_pipeline_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(DeathPipelinePlugin);

        app.add_message::<DamageDealt<PluginTestEntity>>();
        app.add_message::<HealDealt<PluginTestEntity>>();
        app.add_message::<KillYourself<PluginTestEntity>>();
        app.add_message::<Destroyed<PluginTestEntity>>();

        app.add_systems(
            FixedUpdate,
            systems::apply_damage::<PluginTestEntity>.in_set(DeathPipelineSystems::ApplyDamage),
        );
        app.add_systems(
            FixedUpdate,
            systems::detect_deaths::<PluginTestEntity>.in_set(DeathPipelineSystems::DetectDeaths),
        );
        app.add_systems(
            FixedUpdate,
            systems::handle_kill::<PluginTestEntity>.in_set(DeathPipelineSystems::HandleKill),
        );
        app.add_systems(
            FixedUpdate,
            systems::apply_heal::<PluginTestEntity>.in_set(DeathPipelineSystems::ApplyHeal),
        );

        app
    }

    // ── Group K / Behavior 34: DeathPipelineSystems exhaustive match ─────

    const _: fn(DeathPipelineSystems) = |s| match s {
        DeathPipelineSystems::ApplyDamage
        | DeathPipelineSystems::DetectDeaths
        | DeathPipelineSystems::HandleKill
        | DeathPipelineSystems::ApplyHeal => (),
    };

    // ── Group K / Behavior 32: HealDealt<T> registered for all 5 types ───

    #[test]
    fn plugin_registers_heal_dealt_for_cell() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(DeathPipelinePlugin);
        assert!(
            app.world()
                .get_resource::<Messages<HealDealt<Cell>>>()
                .is_some(),
            "DeathPipelinePlugin should register Messages<HealDealt<Cell>>"
        );
    }

    #[test]
    fn plugin_registers_heal_dealt_for_bolt() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(DeathPipelinePlugin);
        assert!(
            app.world()
                .get_resource::<Messages<HealDealt<Bolt>>>()
                .is_some(),
            "DeathPipelinePlugin should register Messages<HealDealt<Bolt>>"
        );
    }

    #[test]
    fn plugin_registers_heal_dealt_for_wall() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(DeathPipelinePlugin);
        assert!(
            app.world()
                .get_resource::<Messages<HealDealt<Wall>>>()
                .is_some(),
            "DeathPipelinePlugin should register Messages<HealDealt<Wall>>"
        );
    }

    #[test]
    fn plugin_registers_heal_dealt_for_breaker() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(DeathPipelinePlugin);
        assert!(
            app.world()
                .get_resource::<Messages<HealDealt<Breaker>>>()
                .is_some(),
            "DeathPipelinePlugin should register Messages<HealDealt<Breaker>>"
        );
    }

    #[test]
    fn plugin_registers_heal_dealt_for_salvo() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(DeathPipelinePlugin);
        assert!(
            app.world()
                .get_resource::<Messages<HealDealt<Salvo>>>()
                .is_some(),
            "DeathPipelinePlugin should register Messages<HealDealt<Salvo>>"
        );
    }

    // ── Group K / Behavior 33: apply_heal::<T> wired for all 5 types ──

    #[derive(Resource, Default)]
    struct PendingCellHeal(Vec<HealDealt<Cell>>);
    fn enqueue_cell_heal(
        pending: Res<PendingCellHeal>,
        mut writer: MessageWriter<HealDealt<Cell>>,
    ) {
        for msg in &pending.0 {
            writer.write(msg.clone());
        }
    }

    #[test]
    fn plugin_registers_apply_heal_cell() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(DeathPipelinePlugin);
        app.init_resource::<PendingCellHeal>();
        app.add_systems(
            FixedUpdate,
            enqueue_cell_heal.before(DeathPipelineSystems::ApplyHeal),
        );

        let cell = app
            .world_mut()
            .spawn((
                Cell,
                Hp {
                    current:  5.0,
                    starting: 10.0,
                    max:      None,
                },
                KilledBy::default(),
                Position2D(Vec2::ZERO),
            ))
            .id();

        app.insert_resource(PendingCellHeal(vec![HealDealt::<Cell> {
            healer:  None,
            target:  cell,
            amount:  2.0,
            source:  None,
            cap:     HealCap::Max,
            _marker: PhantomData,
        }]));

        tick(&mut app);

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert!(
            (hp.current - 7.0).abs() < f32::EPSILON,
            "Cell Hp should be 7.0 after 2.0 heal, got {}",
            hp.current
        );
    }

    #[derive(Resource, Default)]
    struct PendingBoltHeal(Vec<HealDealt<Bolt>>);
    fn enqueue_bolt_heal(
        pending: Res<PendingBoltHeal>,
        mut writer: MessageWriter<HealDealt<Bolt>>,
    ) {
        for msg in &pending.0 {
            writer.write(msg.clone());
        }
    }

    #[test]
    fn plugin_registers_apply_heal_bolt() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(DeathPipelinePlugin);
        app.init_resource::<PendingBoltHeal>();
        app.add_systems(
            FixedUpdate,
            enqueue_bolt_heal.before(DeathPipelineSystems::ApplyHeal),
        );

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Hp {
                    current:  5.0,
                    starting: 10.0,
                    max:      None,
                },
                KilledBy::default(),
                Position2D(Vec2::ZERO),
            ))
            .id();

        app.insert_resource(PendingBoltHeal(vec![HealDealt::<Bolt> {
            healer:  None,
            target:  bolt,
            amount:  2.0,
            source:  None,
            cap:     HealCap::Max,
            _marker: PhantomData,
        }]));

        tick(&mut app);

        let hp = app.world().get::<Hp>(bolt).unwrap();
        assert!(
            (hp.current - 7.0).abs() < f32::EPSILON,
            "Bolt Hp should be 7.0 after 2.0 heal, got {}",
            hp.current
        );
    }

    #[derive(Resource, Default)]
    struct PendingWallHeal(Vec<HealDealt<Wall>>);
    fn enqueue_wall_heal(
        pending: Res<PendingWallHeal>,
        mut writer: MessageWriter<HealDealt<Wall>>,
    ) {
        for msg in &pending.0 {
            writer.write(msg.clone());
        }
    }

    #[test]
    fn plugin_registers_apply_heal_wall() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(DeathPipelinePlugin);
        app.init_resource::<PendingWallHeal>();
        app.add_systems(
            FixedUpdate,
            enqueue_wall_heal.before(DeathPipelineSystems::ApplyHeal),
        );

        let wall = app
            .world_mut()
            .spawn((
                Wall,
                Hp {
                    current:  5.0,
                    starting: 10.0,
                    max:      None,
                },
                KilledBy::default(),
                Position2D(Vec2::ZERO),
            ))
            .id();

        app.insert_resource(PendingWallHeal(vec![HealDealt::<Wall> {
            healer:  None,
            target:  wall,
            amount:  2.0,
            source:  None,
            cap:     HealCap::Max,
            _marker: PhantomData,
        }]));

        tick(&mut app);

        let hp = app.world().get::<Hp>(wall).unwrap();
        assert!(
            (hp.current - 7.0).abs() < f32::EPSILON,
            "Wall Hp should be 7.0 after 2.0 heal, got {}",
            hp.current
        );
    }

    #[derive(Resource, Default)]
    struct PendingBreakerHeal(Vec<HealDealt<Breaker>>);
    fn enqueue_breaker_heal(
        pending: Res<PendingBreakerHeal>,
        mut writer: MessageWriter<HealDealt<Breaker>>,
    ) {
        for msg in &pending.0 {
            writer.write(msg.clone());
        }
    }

    #[test]
    fn plugin_registers_apply_heal_breaker() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(DeathPipelinePlugin);
        app.init_resource::<PendingBreakerHeal>();
        app.add_systems(
            FixedUpdate,
            enqueue_breaker_heal.before(DeathPipelineSystems::ApplyHeal),
        );

        let breaker = app
            .world_mut()
            .spawn((
                Breaker,
                Hp {
                    current:  5.0,
                    starting: 10.0,
                    max:      None,
                },
                KilledBy::default(),
                Position2D(Vec2::ZERO),
            ))
            .id();

        app.insert_resource(PendingBreakerHeal(vec![HealDealt::<Breaker> {
            healer:  None,
            target:  breaker,
            amount:  2.0,
            source:  None,
            cap:     HealCap::Max,
            _marker: PhantomData,
        }]));

        tick(&mut app);

        let hp = app.world().get::<Hp>(breaker).unwrap();
        assert!(
            (hp.current - 7.0).abs() < f32::EPSILON,
            "Breaker Hp should be 7.0 after 2.0 heal, got {}",
            hp.current
        );
    }

    #[derive(Resource, Default)]
    struct PendingSalvoHealPlugin(Vec<HealDealt<Salvo>>);
    fn enqueue_salvo_heal_plugin(
        pending: Res<PendingSalvoHealPlugin>,
        mut writer: MessageWriter<HealDealt<Salvo>>,
    ) {
        for msg in &pending.0 {
            writer.write(msg.clone());
        }
    }

    #[test]
    fn plugin_registers_apply_heal_salvo() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(DeathPipelinePlugin);
        app.init_resource::<PendingSalvoHealPlugin>();
        app.add_systems(
            FixedUpdate,
            enqueue_salvo_heal_plugin.before(DeathPipelineSystems::ApplyHeal),
        );

        let salvo = app
            .world_mut()
            .spawn((
                Salvo,
                Hp {
                    current:  5.0,
                    starting: 10.0,
                    max:      None,
                },
                KilledBy::default(),
                Position2D(Vec2::ZERO),
            ))
            .id();

        app.insert_resource(PendingSalvoHealPlugin(vec![HealDealt::<Salvo> {
            healer:  None,
            target:  salvo,
            amount:  2.0,
            source:  None,
            cap:     HealCap::Max,
            _marker: PhantomData,
        }]));

        tick(&mut app);

        let hp = app.world().get::<Hp>(salvo).unwrap();
        assert!(
            (hp.current - 7.0).abs() < f32::EPSILON,
            "Salvo Hp should be 7.0 after 2.0 heal, got {}",
            hp.current
        );
    }

    // ── Group I / Behaviors 26–29: Full pipeline ordering ──

    #[derive(Resource, Default)]
    struct PendingPtDamage(Vec<DamageDealt<PluginTestEntity>>);
    fn enqueue_pt_damage(
        pending: Res<PendingPtDamage>,
        mut writer: MessageWriter<DamageDealt<PluginTestEntity>>,
    ) {
        for msg in &pending.0 {
            writer.write(msg.clone());
        }
    }

    #[derive(Resource, Default)]
    struct PendingPtHeal(Vec<HealDealt<PluginTestEntity>>);
    fn enqueue_pt_heal(
        pending: Res<PendingPtHeal>,
        mut writer: MessageWriter<HealDealt<PluginTestEntity>>,
    ) {
        for msg in &pending.0 {
            writer.write(msg.clone());
        }
    }

    fn pt_damage_msg(target: Entity, amount: f32) -> DamageDealt<PluginTestEntity> {
        DamageDealt {
            dealer: None,
            target,
            amount,
            source_chip: None,
            _marker: PhantomData,
        }
    }

    fn pt_heal_msg(target: Entity, amount: f32, cap: HealCap) -> HealDealt<PluginTestEntity> {
        HealDealt {
            healer: None,
            target,
            amount,
            source: None,
            cap,
            _marker: PhantomData,
        }
    }

    /// Behavior 26: damage kill + heal in same tick → entity dies, heal blocked.
    #[test]
    fn damage_kills_before_heal_runs_max_cap() {
        let mut app = build_pipeline_app();
        attach_message_capture::<Destroyed<PluginTestEntity>>(&mut app);

        app.init_resource::<PendingPtDamage>();
        app.init_resource::<PendingPtHeal>();
        app.add_systems(
            FixedUpdate,
            enqueue_pt_damage.before(DeathPipelineSystems::ApplyDamage),
        );
        app.add_systems(
            FixedUpdate,
            enqueue_pt_heal.before(DeathPipelineSystems::ApplyHeal),
        );

        let victim = app
            .world_mut()
            .spawn((
                PluginTestEntity,
                Hp::new(10.0),
                KilledBy::default(),
                Position2D(Vec2::ZERO),
            ))
            .id();

        app.insert_resource(PendingPtDamage(vec![pt_damage_msg(victim, 10.0)]));
        app.insert_resource(PendingPtHeal(vec![pt_heal_msg(victim, 5.0, HealCap::Max)]));

        tick(&mut app);

        let destroyed = app
            .world()
            .resource::<MessageCollector<Destroyed<PluginTestEntity>>>();
        assert_eq!(
            destroyed.0.len(),
            1,
            "exactly one Destroyed<PluginTestEntity> should be emitted; heal must not prevent it"
        );

        // If the entity still exists, it must carry the Dead marker with KilledBy set.
        if app.world().get_entity(victim).is_ok() {
            assert!(
                app.world().get::<Dead>(victim).is_some(),
                "victim entity (if still alive) must carry the Dead marker"
            );
            let hp = app.world().get::<Hp>(victim).unwrap();
            assert!(
                hp.current <= 0.0,
                "victim Hp should be <= 0.0, heal must not revive, got {}",
                hp.current
            );
        }
    }

    /// Behavior 26 edge: massive heal still cannot revive.
    #[test]
    fn damage_kills_before_heal_runs_massive_heal_blocked() {
        let mut app = build_pipeline_app();
        attach_message_capture::<Destroyed<PluginTestEntity>>(&mut app);

        app.init_resource::<PendingPtDamage>();
        app.init_resource::<PendingPtHeal>();
        app.add_systems(
            FixedUpdate,
            enqueue_pt_damage.before(DeathPipelineSystems::ApplyDamage),
        );
        app.add_systems(
            FixedUpdate,
            enqueue_pt_heal.before(DeathPipelineSystems::ApplyHeal),
        );

        let victim = app
            .world_mut()
            .spawn((
                PluginTestEntity,
                Hp::new(10.0),
                KilledBy::default(),
                Position2D(Vec2::ZERO),
            ))
            .id();

        app.insert_resource(PendingPtDamage(vec![pt_damage_msg(victim, 10.0)]));
        app.insert_resource(PendingPtHeal(vec![pt_heal_msg(
            victim,
            1_000_000.0,
            HealCap::Max,
        )]));

        tick(&mut app);

        let destroyed = app
            .world()
            .resource::<MessageCollector<Destroyed<PluginTestEntity>>>();
        assert_eq!(
            destroyed.0.len(),
            1,
            "massive heal should not prevent exactly 1 Destroyed; got {}",
            destroyed.0.len()
        );
    }

    /// Behavior 26 edge: 11.0 heal that would more-than-cover the overkill.
    #[test]
    fn damage_kills_before_heal_runs_cover_overkill_blocked() {
        let mut app = build_pipeline_app();
        attach_message_capture::<Destroyed<PluginTestEntity>>(&mut app);

        app.init_resource::<PendingPtDamage>();
        app.init_resource::<PendingPtHeal>();
        app.add_systems(
            FixedUpdate,
            enqueue_pt_damage.before(DeathPipelineSystems::ApplyDamage),
        );
        app.add_systems(
            FixedUpdate,
            enqueue_pt_heal.before(DeathPipelineSystems::ApplyHeal),
        );

        let victim = app
            .world_mut()
            .spawn((
                PluginTestEntity,
                Hp::new(10.0),
                KilledBy::default(),
                Position2D(Vec2::ZERO),
            ))
            .id();

        app.insert_resource(PendingPtDamage(vec![pt_damage_msg(victim, 10.0)]));
        app.insert_resource(PendingPtHeal(vec![pt_heal_msg(victim, 11.0, HealCap::Max)]));

        tick(&mut app);

        let destroyed = app
            .world()
            .resource::<MessageCollector<Destroyed<PluginTestEntity>>>();
        assert_eq!(
            destroyed.0.len(),
            1,
            "11.0 heal must not prevent death; exactly 1 Destroyed expected"
        );
    }

    /// Behavior 26 edge: cap-agnostic — Starting cap also cannot revive.
    #[test]
    fn damage_kills_before_heal_runs_starting_cap_blocked() {
        let mut app = build_pipeline_app();
        attach_message_capture::<Destroyed<PluginTestEntity>>(&mut app);

        app.init_resource::<PendingPtDamage>();
        app.init_resource::<PendingPtHeal>();
        app.add_systems(
            FixedUpdate,
            enqueue_pt_damage.before(DeathPipelineSystems::ApplyDamage),
        );
        app.add_systems(
            FixedUpdate,
            enqueue_pt_heal.before(DeathPipelineSystems::ApplyHeal),
        );

        let victim = app
            .world_mut()
            .spawn((
                PluginTestEntity,
                Hp::new(10.0),
                KilledBy::default(),
                Position2D(Vec2::ZERO),
            ))
            .id();

        app.insert_resource(PendingPtDamage(vec![pt_damage_msg(victim, 10.0)]));
        app.insert_resource(PendingPtHeal(vec![pt_heal_msg(
            victim,
            1_000_000.0,
            HealCap::Starting,
        )]));

        tick(&mut app);

        let destroyed = app
            .world()
            .resource::<MessageCollector<Destroyed<PluginTestEntity>>>();
        assert_eq!(
            destroyed.0.len(),
            1,
            "Starting cap heal must not prevent death"
        );
    }

    /// Behavior 27: non-lethal damage + heal in same tick — additive.
    #[test]
    fn heal_after_non_lethal_damage_adds_in_same_tick() {
        let mut app = build_pipeline_app();
        attach_message_capture::<Destroyed<PluginTestEntity>>(&mut app);

        app.init_resource::<PendingPtDamage>();
        app.init_resource::<PendingPtHeal>();
        app.add_systems(
            FixedUpdate,
            enqueue_pt_damage.before(DeathPipelineSystems::ApplyDamage),
        );
        app.add_systems(
            FixedUpdate,
            enqueue_pt_heal.before(DeathPipelineSystems::ApplyHeal),
        );

        let victim = app
            .world_mut()
            .spawn((
                PluginTestEntity,
                Hp {
                    current:  10.0,
                    starting: 20.0,
                    max:      None,
                },
                KilledBy::default(),
                Position2D(Vec2::ZERO),
            ))
            .id();

        app.insert_resource(PendingPtDamage(vec![pt_damage_msg(victim, 3.0)]));
        app.insert_resource(PendingPtHeal(vec![pt_heal_msg(victim, 5.0, HealCap::Max)]));

        tick(&mut app);

        let hp = app.world().get::<Hp>(victim).unwrap();
        assert!(
            (hp.current - 12.0).abs() < f32::EPSILON,
            "Non-lethal damage + heal should leave 12.0, got {}",
            hp.current
        );
        assert!(
            app.world().get::<Dead>(victim).is_none(),
            "victim must NOT be Dead — damage was non-lethal"
        );
        let killed_by = app.world().get::<KilledBy>(victim).unwrap();
        assert_eq!(killed_by.dealer, None, "KilledBy should remain unset");

        let destroyed = app
            .world()
            .resource::<MessageCollector<Destroyed<PluginTestEntity>>>();
        assert!(
            destroyed.0.is_empty(),
            "No Destroyed should be emitted for non-lethal damage"
        );
    }

    /// Behavior 27 edge: same result regardless of message enqueue order.
    /// (The test app enqueues via separate systems so message order is governed
    /// by set ordering, not enqueue ordering — this edge is an observational
    /// restatement that the two systems run in their own sets.)
    #[test]
    fn heal_after_non_lethal_damage_order_insensitive() {
        let mut app = build_pipeline_app();
        attach_message_capture::<Destroyed<PluginTestEntity>>(&mut app);

        app.init_resource::<PendingPtDamage>();
        app.init_resource::<PendingPtHeal>();
        app.add_systems(
            FixedUpdate,
            enqueue_pt_heal.before(DeathPipelineSystems::ApplyHeal),
        );
        app.add_systems(
            FixedUpdate,
            enqueue_pt_damage.before(DeathPipelineSystems::ApplyDamage),
        );

        let victim = app
            .world_mut()
            .spawn((
                PluginTestEntity,
                Hp {
                    current:  10.0,
                    starting: 20.0,
                    max:      None,
                },
                KilledBy::default(),
                Position2D(Vec2::ZERO),
            ))
            .id();

        app.insert_resource(PendingPtDamage(vec![pt_damage_msg(victim, 3.0)]));
        app.insert_resource(PendingPtHeal(vec![pt_heal_msg(victim, 5.0, HealCap::Max)]));

        tick(&mut app);

        let hp = app.world().get::<Hp>(victim).unwrap();
        assert!(
            (hp.current - 12.0).abs() < f32::EPSILON,
            "Order-independent result should still be 12.0, got {}",
            hp.current
        );
    }

    /// Behavior 28: tick 1 damage-kills, tick 2 heal is skipped.
    #[test]
    fn sequence_tick_kill_then_heal_is_skipped() {
        let mut app = build_pipeline_app();
        attach_message_capture::<Destroyed<PluginTestEntity>>(&mut app);

        app.init_resource::<PendingPtDamage>();
        app.init_resource::<PendingPtHeal>();
        app.add_systems(
            FixedUpdate,
            enqueue_pt_damage.before(DeathPipelineSystems::ApplyDamage),
        );
        app.add_systems(
            FixedUpdate,
            enqueue_pt_heal.before(DeathPipelineSystems::ApplyHeal),
        );

        let victim = app
            .world_mut()
            .spawn((
                PluginTestEntity,
                Hp {
                    current:  5.0,
                    starting: 10.0,
                    max:      None,
                },
                KilledBy::default(),
                Position2D(Vec2::ZERO),
            ))
            .id();

        // Tick 1: damage kills the victim.
        app.insert_resource(PendingPtDamage(vec![pt_damage_msg(victim, 5.0)]));
        app.insert_resource(PendingPtHeal(Vec::<HealDealt<PluginTestEntity>>::new()));
        tick(&mut app);

        // After tick 1: collector holds the Destroyed from this tick.
        {
            let destroyed = app
                .world()
                .resource::<MessageCollector<Destroyed<PluginTestEntity>>>();
            assert_eq!(
                destroyed.0.len(),
                1,
                "tick 1 damage-kill must emit exactly 1 Destroyed",
            );
        }

        // Tick 2: massive heal targeting the now-dead/despawned victim.
        app.insert_resource(PendingPtDamage(Vec::<DamageDealt<PluginTestEntity>>::new()));
        app.insert_resource(PendingPtHeal(vec![pt_heal_msg(
            victim,
            100.0,
            HealCap::Max,
        )]));
        tick(&mut app);

        // After tick 2: collector is cleared at First of tick 2 and re-populated
        // only from tick-2 emissions. It must be empty — no second kill, no revival.
        {
            let destroyed = app
                .world()
                .resource::<MessageCollector<Destroyed<PluginTestEntity>>>();
            assert_eq!(
                destroyed.0.len(),
                0,
                "tick 2 heal must not emit Destroyed (no revival, no second kill)",
            );
        }

        // Final check: the victim is not alive with a positive HP. Either the
        // entity was despawned (query returns None) or, if it survived despawn
        // somehow, it still holds the Dead marker and HP did not rebound.
        let world = app.world();
        if let Some(hp) = world.get::<Hp>(victim) {
            assert!(
                hp.current <= 0.0,
                "heal must not rebound HP above zero on a dead entity; got {}",
                hp.current,
            );
            assert!(
                world.entity(victim).contains::<Dead>(),
                "victim must still hold Dead marker after tick-2 heal is skipped",
            );
        }
    }

    /// Behavior 29: observational proof that `apply_heal` runs AFTER `handle_kill`.
    #[test]
    fn apply_heal_runs_after_handle_kill_observationally() {
        let mut app = build_pipeline_app();
        attach_message_capture::<Destroyed<PluginTestEntity>>(&mut app);

        app.init_resource::<PendingPtDamage>();
        app.init_resource::<PendingPtHeal>();
        app.add_systems(
            FixedUpdate,
            enqueue_pt_damage.before(DeathPipelineSystems::ApplyDamage),
        );
        app.add_systems(
            FixedUpdate,
            enqueue_pt_heal.before(DeathPipelineSystems::ApplyHeal),
        );

        let victim = app
            .world_mut()
            .spawn((
                PluginTestEntity,
                Hp::new(10.0),
                KilledBy::default(),
                Position2D(Vec2::ZERO),
            ))
            .id();

        app.insert_resource(PendingPtDamage(vec![pt_damage_msg(victim, 10.0)]));
        app.insert_resource(PendingPtHeal(vec![pt_heal_msg(
            victim,
            999.0,
            HealCap::Max,
        )]));

        tick(&mut app);

        let destroyed = app
            .world()
            .resource::<MessageCollector<Destroyed<PluginTestEntity>>>();
        assert_eq!(
            destroyed.0.len(),
            1,
            "pinned ApplyDamage->DetectDeaths->HandleKill->ApplyHeal ordering should produce exactly 1 Destroyed"
        );

        // If the entity is still alive (before FixedPostUpdate despawn), Hp must be <= 0.
        if app.world().get_entity(victim).is_ok() {
            let hp = app.world().get::<Hp>(victim).unwrap();
            assert!(
                hp.current <= 0.0,
                "Hp should remain <= 0.0 — heal must NOT run before DetectDeaths; got {}",
                hp.current
            );
        }
    }
}
