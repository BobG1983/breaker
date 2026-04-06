use crate::types::*;

// -------------------------------------------------------------------------
// StressConfig — serde deserialization
// -------------------------------------------------------------------------

#[test]
fn stress_config_parses_full_ron() {
    let ron = "(runs: 64, parallelism: 8)";
    let result: StressConfig = ron::de::from_str(ron).expect("StressConfig full should parse");
    assert_eq!(
        result,
        StressConfig {
            runs: 64,
            parallelism: 8,
        }
    );
}

#[test]
fn stress_config_defaults_both_fields_from_empty_struct() {
    let ron = "()";
    let result: StressConfig =
        ron::de::from_str(ron).expect("StressConfig empty struct should parse");
    assert_eq!(
        result,
        StressConfig {
            runs: 32,
            parallelism: 32,
        }
    );
}

#[test]
fn stress_config_partial_override_only_runs() {
    let ron = "(runs: 64)";
    let result: StressConfig = ron::de::from_str(ron).expect("StressConfig runs-only should parse");
    assert_eq!(
        result,
        StressConfig {
            runs: 64,
            parallelism: 32,
        }
    );
}

#[test]
fn stress_config_partial_override_only_parallelism() {
    let ron = "(parallelism: 4)";
    let result: StressConfig =
        ron::de::from_str(ron).expect("StressConfig parallelism-only should parse");
    assert_eq!(
        result,
        StressConfig {
            runs: 32,
            parallelism: 4,
        }
    );
}
