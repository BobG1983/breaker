//! Group D — Schedule wiring tests for Wave 2C `ChipRng` migration.
//!
//! Covers Behaviors 15–20: `reseed_chip_rng` and `tick_chip_select_count` fire on
//! `OnEnter(ChipSelectState::Selecting)`, set-membership probes, and the
//! visit-ordering contract.
//!
//! Harness rule: `init_resource::<RoutingTable<ChipSelectState>>()` MUST be called
//! BEFORE `add_plugins(ChipSelectPlugin)`. Without it the plugin panics during build.

use bevy::prelude::*;
use rand::Rng;
use rantzsoft_stateflow::{ChangeState, RoutingTable};

use super::systems::{generate_chip_offerings, reseed_chip_rng, tick_chip_select_count};
use crate::{
    chips::{ChipCatalog, ChipDefinition, definition::Rarity, inventory::ChipInventory},
    effect_v3::{
        effects::PiercingConfig,
        types::{EffectType, Tree},
    },
    input::resources::InputConfig,
    mutators::protocols::{
        messages::ProtocolSelected,
        resources::{ActiveProtocols, ProtocolOffer},
    },
    prelude::*,
    shared::{
        RunSeed,
        rng::{ChipRng, ChipSelectCount, derive_seed, derive_seed_named},
    },
    state::run::chip_select::{
        ChipSelectConfig, ChipSelectPlugin,
        messages::{ChipOfferSkipped, ChipSelected},
        resources::{ChipOffers, ChipSelectSelection},
        sets::ChipSelectSystems,
    },
};

const SENTINEL: u64 = 0xDEAD_BEEF_CAFE_1234;

// ── Shared harness helpers ───────────────────────────────────────────────────

/// Registry with 4 chips of mixed rarities — ensures RNG draws affect offering
/// names (so Behaviors 17 and 20 can observe which chip sequence was produced).
fn make_mixed_registry() -> ChipCatalog {
    let mut registry = ChipCatalog::default();
    for i in 0..3 {
        registry.insert(ChipDefinition {
            rarity: Rarity::Common,
            ..ChipDefinition::test(
                &format!("Common_{i}"),
                Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
                3,
            )
        });
    }
    registry.insert(ChipDefinition {
        rarity: Rarity::Rare,
        ..ChipDefinition::test(
            "Rare_0",
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
            3,
        )
    });
    registry
}

/// Base schedule app — adds all resources `ChipSelectPlugin::build()` requires
/// transitively so the plugin does not panic at system-param validation.
///
/// Callers MUST call `init_resource::<RoutingTable<ChipSelectState>>()` then
/// `add_plugins(ChipSelectPlugin)` AFTER building this app.
fn schedule_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<RunStats>()
        .with_resource::<RunSeed>()
        .with_resource::<ChipSelectCount>()
        .with_resource::<ChipRng>()
        .with_resource::<GameRng>()
        .with_resource::<ChipInventory>()
        .build();
    app.insert_resource(RunStats {
        seed: 42,
        ..default()
    });
    app.insert_resource(ChipRng::from_seed(SENTINEL));
    app.insert_resource(ChipSelectCount(0));
    app.insert_resource(GameRng::from_seed(42));
    app.insert_resource(make_mixed_registry());
    app.insert_resource(ChipSelectConfig::default());
    // Resources required by handle_chip_input, spawn_chip_select, and tick_chip_timer:
    app.insert_resource(InputConfig::default());
    app.insert_resource(ActiveProtocols::default());
    app.init_resource::<ProtocolOffer>();
    app.init_resource::<ChipSelectSelection>();
    // Messages required by ChipSelectPlugin systems:
    app.add_message::<ChipOfferSkipped>();
    app.add_message::<ChipSelected>();
    app.add_message::<ProtocolSelected>();
    app.add_message::<ChangeState<ChipSelectState>>();
    app
}

/// Drive the state hierarchy into `ChipSelectState::Selecting`.
fn drive_to_selecting(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Game);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Run);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<RunState>>()
        .set(RunState::ChipSelect);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<ChipSelectState>>()
        .set(ChipSelectState::Selecting);
    app.update();
}

// ── Behavior 15 — reseed_chip_rng fires on OnEnter(ChipSelectState::Selecting) ─

/// B15 primary: `ChipRng` is reseeded with canonical formula on first entry to Selecting.
/// At RED: stub does nothing → `ChipRng` keeps SENTINEL seed → draw does NOT match
/// canonical seed → assertion FAILS.
#[test]
fn reseed_chip_rng_fires_on_enter_chip_select_selecting() {
    let mut app = schedule_app();
    app.init_resource::<RoutingTable<ChipSelectState>>();
    app.add_plugins(ChipSelectPlugin);

    drive_to_selecting(&mut app);

    let expected_seed = derive_seed(derive_seed_named(42, "chip"), 0);
    let mut expected_rng = ChipRng::from_seed(expected_seed);
    let expected_draw: u64 = expected_rng.0.random();

    let actual_draw: u64 = app.world_mut().resource_mut::<ChipRng>().0.random();
    assert_eq!(
        actual_draw, expected_draw,
        "ChipRng after OnEnter(ChipSelectState::Selecting) must match \
         derive_seed(derive_seed_named(42, \"chip\"), 0) for count=0"
    );
}

/// B15 edge case: reseed fires on EVERY entry to `Selecting`, not just the first.
/// Transition out to `AnimateOut`, then back to `Selecting` with `ChipSelectCount` initialized to 1.
/// At RED: stub does nothing → `ChipRng` holds whatever state was left → assertion FAILS.
#[test]
fn reseed_chip_rng_reseeds_on_second_entry_to_selecting() {
    let mut app = schedule_app();
    app.init_resource::<RoutingTable<ChipSelectState>>();
    app.add_plugins(ChipSelectPlugin);

    drive_to_selecting(&mut app);

    // Transition out, then set count=1, then back to Selecting.
    app.world_mut()
        .resource_mut::<NextState<ChipSelectState>>()
        .set(ChipSelectState::AnimateOut);
    app.update();

    // At this point tick_chip_select_count has incremented to 1 in production;
    // at RED we set it manually so the assertion is about the reseed formula.
    app.insert_resource(ChipSelectCount(1));
    // Re-sentinel the ChipRng so stale state doesn't accidentally match.
    app.insert_resource(ChipRng::from_seed(SENTINEL));

    app.world_mut()
        .resource_mut::<NextState<ChipSelectState>>()
        .set(ChipSelectState::Selecting);
    app.update();

    let expected_seed = derive_seed(derive_seed_named(42, "chip"), 1);
    let mut expected_rng = ChipRng::from_seed(expected_seed);
    let expected_draw: u64 = expected_rng.0.random();

    let actual_draw: u64 = app.world_mut().resource_mut::<ChipRng>().0.random();
    assert_eq!(
        actual_draw, expected_draw,
        "ChipRng on second entry to Selecting must match formula for count=1"
    );
}

// ── Behavior 16 — reseed_chip_rng is in ChipSelectSystems::ReseedRng ────────

/// B16 primary: `reseed_chip_rng` is in `ChipSelectSystems::ReseedRng`.
/// At RED: this depends on the set registration in `ChipSelectPlugin::build()`.
/// If the plugin registers the system in the correct set, this passes at RED too
/// (set-membership tests validate plugin wiring, not production logic).
#[test]
fn reseed_chip_rng_is_in_reseed_rng_set() {
    let mut app = schedule_app();
    app.init_resource::<RoutingTable<ChipSelectState>>();
    app.add_plugins(ChipSelectPlugin);

    assert!(
        system_in_set(
            &mut app,
            OnEnter(ChipSelectState::Selecting),
            reseed_chip_rng,
            ChipSelectSystems::ReseedRng,
        ),
        "reseed_chip_rng must be in ChipSelectSystems::ReseedRng in \
         OnEnter(ChipSelectState::Selecting)"
    );
}

/// B16 edge case: `reseed_chip_rng` is NOT in `ChipSelectSystems::GenerateOfferings`.
#[test]
fn reseed_chip_rng_is_not_in_generate_offerings_set() {
    let mut app = schedule_app();
    app.init_resource::<RoutingTable<ChipSelectState>>();
    app.add_plugins(ChipSelectPlugin);

    assert!(
        !system_in_set(
            &mut app,
            OnEnter(ChipSelectState::Selecting),
            reseed_chip_rng,
            ChipSelectSystems::GenerateOfferings,
        ),
        "reseed_chip_rng must NOT be in ChipSelectSystems::GenerateOfferings"
    );
}

// ── Behavior 17 — reseed_chip_rng runs BEFORE generate_chip_offerings ────────

/// B17: offerings from the plugin-driven harness (which runs reseed before generate)
/// must match a standalone harness seeded directly with the canonical seed.
///
/// At RED: both harnesses read `GameRng::from_seed(42)` (unmigrated generate reads `GameRng`)
/// → names trivially match. This test passes at RED and is load-bearing at GREEN.
/// At GREEN: plugin harness uses `ChipRng` post-reseed (canonical seed); if reseed runs
/// AFTER generate, the `ChipRng` would still hold SENTINEL when generate runs, and the
/// canonical-seed standalone would differ.
#[test]
fn reseed_runs_before_generate_chip_offerings() {
    // Plugin-driven harness: reaches Selecting and fires both systems.
    let plugin_names: Vec<String> = {
        let mut app = schedule_app();
        app.init_resource::<RoutingTable<ChipSelectState>>();
        app.add_plugins(ChipSelectPlugin);

        drive_to_selecting(&mut app);

        app.world()
            .resource::<ChipOffers>()
            .0
            .iter()
            .map(|o| o.name().to_owned())
            .collect()
    };

    // Standalone harness: seeded directly with the canonical seed for visit 0.
    let canonical_seed = derive_seed(derive_seed_named(42, "chip"), 0);
    let standalone_names: Vec<String> = {
        let mut app = TestAppBuilder::new()
            .insert_resource(make_mixed_registry())
            .with_resource::<ChipInventory>()
            .insert_resource(ChipSelectConfig::default())
            .insert_resource(ChipRng::from_seed(canonical_seed))
            .insert_resource(GameRng::from_seed(42))
            .with_system(Update, generate_chip_offerings)
            .build();
        app.update();
        app.world()
            .resource::<ChipOffers>()
            .0
            .iter()
            .map(|o| o.name().to_owned())
            .collect()
    };

    assert_eq!(
        plugin_names, standalone_names,
        "offerings from the plugin-driven harness must match the standalone canonical-seed \
         harness; if they differ at GREEN, reseed ran after generate instead of before"
    );
}

// ── Behavior 18 — generate_chip_offerings is in ChipSelectSystems::GenerateOfferings

/// B18 primary: `generate_chip_offerings` is in `ChipSelectSystems::GenerateOfferings`.
#[test]
fn generate_chip_offerings_is_in_generate_offerings_set() {
    let mut app = schedule_app();
    app.init_resource::<RoutingTable<ChipSelectState>>();
    app.add_plugins(ChipSelectPlugin);

    assert!(
        system_in_set(
            &mut app,
            OnEnter(ChipSelectState::Selecting),
            generate_chip_offerings,
            ChipSelectSystems::GenerateOfferings,
        ),
        "generate_chip_offerings must be in ChipSelectSystems::GenerateOfferings in \
         OnEnter(ChipSelectState::Selecting)"
    );
}

/// B18 edge case: `generate_chip_offerings` is NOT in `ChipSelectSystems::ReseedRng`.
#[test]
fn generate_chip_offerings_is_not_in_reseed_rng_set() {
    let mut app = schedule_app();
    app.init_resource::<RoutingTable<ChipSelectState>>();
    app.add_plugins(ChipSelectPlugin);

    assert!(
        !system_in_set(
            &mut app,
            OnEnter(ChipSelectState::Selecting),
            generate_chip_offerings,
            ChipSelectSystems::ReseedRng,
        ),
        "generate_chip_offerings must NOT be in ChipSelectSystems::ReseedRng"
    );
}

// ── Behavior 19 — tick_chip_select_count fires on OnEnter(Selecting) ────────

/// B19 primary: entering Selecting increments `ChipSelectCount` from 0 to 1.
/// At RED: stub does nothing → count stays 0 → assertion FAILS.
#[test]
fn tick_chip_select_count_fires_on_enter_selecting() {
    let mut app = schedule_app();
    app.init_resource::<RoutingTable<ChipSelectState>>();
    app.add_plugins(ChipSelectPlugin);

    drive_to_selecting(&mut app);

    assert_eq!(
        app.world().resource::<ChipSelectCount>().0,
        1,
        "ChipSelectCount must be 1 after the first OnEnter(ChipSelectState::Selecting)"
    );
}

/// B19 edge case: second entry increments from 1 to 2.
/// At RED: stub does nothing → count stays 0 after first entry, still 0 after second.
#[test]
fn tick_chip_select_count_increments_on_each_selecting_entry() {
    let mut app = schedule_app();
    app.init_resource::<RoutingTable<ChipSelectState>>();
    app.add_plugins(ChipSelectPlugin);

    drive_to_selecting(&mut app);

    assert_eq!(
        app.world().resource::<ChipSelectCount>().0,
        1,
        "ChipSelectCount must be 1 after the first entry"
    );

    app.world_mut()
        .resource_mut::<NextState<ChipSelectState>>()
        .set(ChipSelectState::AnimateOut);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<ChipSelectState>>()
        .set(ChipSelectState::Selecting);
    app.update();

    assert_eq!(
        app.world().resource::<ChipSelectCount>().0,
        2,
        "ChipSelectCount must be 2 after the second entry to Selecting"
    );
}

// ── Behavior 20 — Visit-ordering contract ────────────────────────────────────

fn standalone_offerings_for_discriminator(run_seed: u64, discriminator: u64) -> Vec<String> {
    let seed = derive_seed(derive_seed_named(run_seed, "chip"), discriminator);
    let mut app = TestAppBuilder::new()
        .insert_resource(make_mixed_registry())
        .with_resource::<ChipInventory>()
        .insert_resource(ChipSelectConfig::default())
        .insert_resource(ChipRng::from_seed(seed))
        .insert_resource(GameRng::from_seed(run_seed))
        .with_system(Update, generate_chip_offerings)
        .build();
    app.update();
    app.world()
        .resource::<ChipOffers>()
        .0
        .iter()
        .map(|o| o.name().to_owned())
        .collect()
}

/// B20: the Nth chip-select entry (0-indexed) reseeds from discriminator N.
/// After reseed, `generate_chip_offerings` draws from the correctly-seeded `ChipRng`.
///
/// We verify this by comparing three consecutive entry sequences against standalone
/// harnesses seeded with the canonical seeds for discriminators 0, 1, and 2.
///
/// At RED: `generate_chip_offerings` reads `GameRng` (unmigrated), not `ChipRng`, so the
/// standalone comparison also uses `GameRng::from_seed(42)` — the names match trivially
/// because both harnesses draw from identical `GameRng` state. The discriminator-specific
/// diff assertion (edge case below) fails because both "visit 1" and "visit 2" produce
/// the same sequence (`GameRng` doesn't get reseeded between visits at RED).
#[test]
fn visit_ordering_contract_nth_entry_uses_discriminator_n() {
    let mut app = schedule_app();
    app.init_resource::<RoutingTable<ChipSelectState>>();
    app.add_plugins(ChipSelectPlugin);

    let collect_names = |app: &App| -> Vec<String> {
        app.world()
            .resource::<ChipOffers>()
            .0
            .iter()
            .map(|o| o.name().to_owned())
            .collect()
    };

    let advance_to_selecting = |app: &mut App| {
        app.world_mut()
            .resource_mut::<NextState<ChipSelectState>>()
            .set(ChipSelectState::AnimateOut);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<ChipSelectState>>()
            .set(ChipSelectState::Selecting);
        app.update();
    };

    // Visit 1 (discriminator 0)
    drive_to_selecting(&mut app);
    assert_eq!(
        collect_names(&app),
        standalone_offerings_for_discriminator(42, 0),
        "1st chip-select entry offerings must match standalone for discriminator 0"
    );

    // Visit 2 (discriminator 1)
    advance_to_selecting(&mut app);
    assert_eq!(
        collect_names(&app),
        standalone_offerings_for_discriminator(42, 1),
        "2nd chip-select entry offerings must match standalone for discriminator 1"
    );

    // Visit 3 (discriminator 2)
    advance_to_selecting(&mut app);
    assert_eq!(
        collect_names(&app),
        standalone_offerings_for_discriminator(42, 2),
        "3rd chip-select entry offerings must match standalone for discriminator 2"
    );

    assert_eq!(
        app.world().resource::<ChipSelectCount>().0,
        3,
        "ChipSelectCount must be 3 after three entries to Selecting"
    );
}

/// B20 edge case: 1st-entry and 2nd-entry offerings MUST differ (different discriminator).
/// At RED: `generate_chip_offerings` reads `GameRng` which does NOT get reseeded between
/// visits → both visits draw from the same post-first-visit `GameRng` state → names
/// may or may not differ depending on the RNG. This assertion is about the `ChipRng`
/// discriminator producing a different sequence; it becomes load-bearing at GREEN.
#[test]
fn consecutive_chip_select_entries_produce_different_offerings() {
    let mut app = schedule_app();
    app.init_resource::<RoutingTable<ChipSelectState>>();
    app.add_plugins(ChipSelectPlugin);

    drive_to_selecting(&mut app);

    let visit1_names: Vec<String> = app
        .world()
        .resource::<ChipOffers>()
        .0
        .iter()
        .map(|o| o.name().to_owned())
        .collect();

    app.world_mut()
        .resource_mut::<NextState<ChipSelectState>>()
        .set(ChipSelectState::AnimateOut);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<ChipSelectState>>()
        .set(ChipSelectState::Selecting);
    app.update();

    let visit2_names: Vec<String> = app
        .world()
        .resource::<ChipOffers>()
        .0
        .iter()
        .map(|o| o.name().to_owned())
        .collect();

    assert_ne!(
        visit1_names, visit2_names,
        "consecutive chip-select entries must produce different offerings \
         (different ChipRng discriminator → different seed → different sequence)"
    );
}
