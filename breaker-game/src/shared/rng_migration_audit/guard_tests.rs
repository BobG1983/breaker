//! Guard tests for the RNG-domain-partitioning migration (Wave 3).
//!
//! Each test asserts a post-migration structural invariant about `GameRng`
//! usage in production code. The forbidden identifier and the snapshot list
//! of production files live in `super::file_list`.

use super::file_list::{GAME_RNG, PRODUCTION_FILES};

// Behavior 1 (intentionally GREEN at RED phase): no production file outside the
// allow-list holds `ResMut<GameRng>` or `Res<GameRng>` as a system parameter.
//
// Allowed: capture_run_seed.rs and reset_run_state.rs (the only authorized consumers).
// All other files must have zero occurrences.
#[test]
fn only_capture_run_seed_and_reset_run_state_take_game_rng_system_param() {
    let res_game_rng = concat!("Res", "Mut<", "Game", "Rng>");
    let legacy_rng_ref = concat!("Res<", "Game", "Rng>");
    for (path, content) in PRODUCTION_FILES {
        let mut_count = content.matches(res_game_rng).count();
        let ref_count = content.matches(legacy_rng_ref).count();
        let total = mut_count + ref_count;
        let allowed = matches!(
            *path,
            "state/run/loading/systems/capture_run_seed.rs"
                | "state/run/loading/systems/reset_run_state/system.rs"
        );
        if !allowed {
            assert_eq!(
                total, 0,
                "unexpected ResMut<GameRng>/Res<GameRng> in {path}; \
                 only capture_run_seed.rs and reset_run_state/system.rs are allowed"
            );
        }
    }
}

// Behavior 2 (intentionally GREEN at RED phase): `init_resource::<GameRng>()` appears
// exactly once — in `state/run/plugin.rs` — and nowhere else in production code.
#[test]
fn init_resource_game_rng_appears_exactly_once_in_state_run_plugin() {
    let needle = concat!("init_resource::<", "Game", "Rng>()");
    let mut matches: Vec<&str> = Vec::new();
    for (path, content) in PRODUCTION_FILES {
        if content.contains(needle) {
            matches.push(path);
        }
    }
    assert_eq!(
        matches,
        &["state/run/plugin.rs"],
        "init_resource::<GameRng>() must appear in exactly one production file \
         (state/run/plugin.rs); found in: {matches:?}"
    );
}

// Behavior 3a (intentionally GREEN at RED phase): `shared/rng/rng_types.rs` still
// defines `pub struct GameRng`.
#[test]
fn shared_rng_types_rs_defines_pub_struct_game_rng() {
    let rng_source = include_str!("../rng/rng_types.rs");
    assert!(
        rng_source.contains(concat!("pub struct ", "Game", "Rng")),
        "shared/rng/rng_types.rs must still define `pub struct GameRng(...)` — \
         capture_run_seed and reset_run_state depend on it"
    );
}

// Behavior 3b (intentionally GREEN at RED phase): `shared/mod.rs` re-exports `GameRng`
// and `prelude/resources.rs` references it.
#[test]
fn shared_mod_rs_reexports_game_rng_and_prelude_resources_references_it() {
    let mod_source = include_str!("../mod.rs");
    assert!(
        mod_source.contains(concat!("pub use rng::", "Game", "Rng")),
        "shared/mod.rs must re-export `pub use rng::GameRng`"
    );
    let prelude_source = include_str!("../../prelude/resources.rs");
    assert!(
        prelude_source.contains(GAME_RNG),
        "prelude/resources.rs must reference GameRng (re-export for downstream consumers)"
    );
}

// Behavior 9 (FAILS at RED — docs/architecture/rng.md does not exist yet):
// The architecture doc exists and contains the required sections.
//
// Runtime file read so compilation succeeds; the test fails at runtime until
// writer-code creates the file at GREEN.
#[test]
fn docs_architecture_rng_md_exists_and_has_required_sections() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/architecture/rng.md");
    let content = std::fs::read_to_string(path)
        .expect("docs/architecture/rng.md must exist — writer-code creates it at GREEN");
    assert!(
        content.contains("Seed Derivation Tree"),
        "docs/architecture/rng.md must contain a 'Seed Derivation Tree' section"
    );
    assert!(
        content.contains("What the Run Seed Covers"),
        "docs/architecture/rng.md must contain a 'What the Run Seed Covers' section"
    );
    assert!(
        content.contains("Scenario Runner"),
        "docs/architecture/rng.md must contain a 'Scenario Runner' section"
    );
}
