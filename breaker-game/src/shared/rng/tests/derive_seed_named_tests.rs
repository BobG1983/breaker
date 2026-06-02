use super::super::rng_types::*;

// ── Group B — derive_seed_named determinism & spread ───────────────────

mod derive_seed_named {
    use super::*;

    // Hard-coded reference constants for B8 — computed from the
    // canonical formula WITHOUT calling derive_seed/_named.
    //
    // Canonical formula (wrapping_add variant):
    //   name_hash = name.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
    //   h         = parent.wrapping_add(name_hash).wrapping_add(0x9E37_79B9_7F4A_7C15)
    //   h         = h.wrapping_mul(0x517c_c1b7_2722_0a95)
    //   final     = h ^ (h >> 32)
    //
    // Both constants are evaluated as const expressions using the same
    // arithmetic the production implementation must produce. If derive_seed
    // or derive_seed_named changes formula, update these constants to match.
    //
    // derive_seed_named(0, "bolt"):
    //   name_hash("bolt"): b=98,o=111,l=108,t=116
    //     ((((0*31+98)*31+111)*31+108)*31+116) = 3029653
    //   h = 0u64.wrapping_add(3029653).wrapping_add(0x9E3779B97F4A7C15)
    //   h = h.wrapping_mul(0x517cc1b727220a95)
    //   final = h ^ (h >> 32)
    const DERIVE_SEED_NAMED_0_BOLT: u64 = {
        let name_hash: u64 = {
            let bytes = b"bolt";
            let mut h: u64 = 0;
            let mut i = 0usize;
            while i < bytes.len() {
                h = h.wrapping_mul(31).wrapping_add(bytes[i] as u64);
                i += 1;
            }
            h
        };
        let mut h = (0u64)
            .wrapping_add(name_hash)
            .wrapping_add(0x9E37_79B9_7F4A_7C15);
        h = h.wrapping_mul(0x517c_c1b7_2722_0a95);
        h ^ (h >> 32)
    };

    // derive_seed_named(42, "effect"):
    //   name_hash("effect"): e=101,f=102,f=102,e=101,c=99,t=116
    //     = 2988882321
    //   h = 42u64.wrapping_add(2988882321).wrapping_add(0x9E3779B97F4A7C15)
    //   h = h.wrapping_mul(0x517cc1b727220a95)
    //   final = h ^ (h >> 32)
    const DERIVE_SEED_NAMED_42_EFFECT: u64 = {
        let name_hash: u64 = {
            let bytes = b"effect";
            let mut h: u64 = 0;
            let mut i = 0usize;
            while i < bytes.len() {
                h = h.wrapping_mul(31).wrapping_add(bytes[i] as u64);
                i += 1;
            }
            h
        };
        let mut h = (42u64)
            .wrapping_add(name_hash)
            .wrapping_add(0x9E37_79B9_7F4A_7C15);
        h = h.wrapping_mul(0x517c_c1b7_2722_0a95);
        h ^ (h >> 32)
    };

    #[test]
    fn same_inputs_return_same_value() {
        let a = derive_seed_named(100, "bolt");
        let b = derive_seed_named(100, "bolt");
        assert_eq!(a, b, "derive_seed_named must be deterministic");
    }

    #[test]
    fn empty_name_is_stable_and_does_not_panic() {
        let a = derive_seed_named(100, "");
        let b = derive_seed_named(100, "");
        assert_eq!(a, b, "derive_seed_named with empty name must be stable");
    }

    #[test]
    fn eight_channel_names_are_pairwise_distinct() {
        let names = [
            "bolt",
            "chip",
            "node_sequence",
            "effect",
            "hazard",
            "protocol",
            "node_gen",
            "chain_tick",
        ];
        let seeds: Vec<u64> = names.iter().map(|n| derive_seed_named(100, n)).collect();
        for i in 0..seeds.len() {
            for j in (i + 1)..seeds.len() {
                assert_ne!(
                    seeds[i], seeds[j],
                    "derive_seed_named(100, {:?}) must differ from derive_seed_named(100, {:?})",
                    names[i], names[j]
                );
            }
        }
    }

    #[test]
    fn one_byte_difference_in_name_produces_different_seed() {
        let bolt_seed = derive_seed_named(100, "bolt");
        let bolu_derived = derive_seed_named(100, "bolu");
        assert_ne!(
            bolt_seed, bolu_derived,
            "\"bolt\" and \"bolu\" must produce distinct seeds"
        );
    }

    #[test]
    fn distinct_parents_produce_distinct_outputs() {
        let v0 = derive_seed_named(0, "bolt");
        let v42 = derive_seed_named(42, "bolt");
        let vb = derive_seed_named(0xDEAD_BEEF, "bolt");
        assert_ne!(v0, v42);
        assert_ne!(v42, vb);
        assert_ne!(v0, vb);
    }

    #[test]
    fn adjacent_parents_produce_distinct_seeds() {
        let v0 = derive_seed_named(0, "bolt");
        let v1 = derive_seed_named(1, "bolt");
        assert_ne!(
            v0, v1,
            "parents 0 and 1 with 'bolt' must produce distinct seeds"
        );
    }

    /// B8 main: pins the formula to an externally-precomputed constant.
    /// A stub returning 0 fails because 0 != `DERIVE_SEED_NAMED_0_BOLT`.
    #[test]
    fn formula_matches_precomputed_constant_for_zero_parent_bolt() {
        let result = derive_seed_named(0, "bolt");
        assert_eq!(
            result, DERIVE_SEED_NAMED_0_BOLT,
            "derive_seed_named(0, \"bolt\") must equal the precomputed constant \
             0x{DERIVE_SEED_NAMED_0_BOLT:016X} — stub returning 0 fails this"
        );
    }

    /// B8 edge: pins the formula for (42, "effect").
    #[test]
    fn formula_matches_precomputed_constant_for_42_parent_effect() {
        let result = derive_seed_named(42, "effect");
        assert_eq!(
            result, DERIVE_SEED_NAMED_42_EFFECT,
            "derive_seed_named(42, \"effect\") must equal the precomputed constant \
             0x{DERIVE_SEED_NAMED_42_EFFECT:016X}"
        );
    }
}
