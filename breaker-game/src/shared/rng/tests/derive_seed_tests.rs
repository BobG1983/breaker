use super::super::rng_types::*;

// ── Group A — derive_seed determinism & spread ──────────────────────────

mod derive_seed_basic_determinism {
    use super::*;

    #[test]
    fn same_inputs_always_return_same_u64() {
        let a = derive_seed(0xDEAD_BEEF_u64, 0u64);
        let b = derive_seed(0xDEAD_BEEF_u64, 0u64);
        assert_eq!(a, b, "derive_seed must be pure and deterministic");
    }

    #[test]
    fn zero_inputs_are_stable_across_calls() {
        let a = derive_seed(0, 0);
        let b = derive_seed(0, 0);
        assert_eq!(
            a, b,
            "derive_seed(0,0) must be stable (value may be 0, equality is the invariant)"
        );
    }

    #[test]
    fn distinct_discriminators_produce_distinct_outputs() {
        let v0 = derive_seed(42, 0);
        let v1 = derive_seed(42, 1);
        let v2 = derive_seed(42, 2);
        assert_ne!(
            v0, v1,
            "derive_seed(42,0) must differ from derive_seed(42,1)"
        );
        assert_ne!(
            v1, v2,
            "derive_seed(42,1) must differ from derive_seed(42,2)"
        );
        assert_ne!(
            v0, v2,
            "derive_seed(42,0) must differ from derive_seed(42,2)"
        );
    }

    #[test]
    fn max_discriminator_does_not_panic_and_differs_from_small() {
        let v_max = derive_seed(42, u64::MAX);
        let v0 = derive_seed(42, 0);
        let v1 = derive_seed(42, 1);
        // No panic is the primary contract; additionally it must differ.
        assert_ne!(
            v_max, v0,
            "derive_seed(42, u64::MAX) must differ from derive_seed(42,0)"
        );
        assert_ne!(
            v_max, v1,
            "derive_seed(42, u64::MAX) must differ from derive_seed(42,1)"
        );
    }

    #[test]
    fn distinct_parents_produce_distinct_outputs() {
        let v0 = derive_seed(0, 7);
        let v1 = derive_seed(1, 7);
        let v2 = derive_seed(2, 7);
        assert_ne!(v0, v1, "derive_seed(0,7) must differ from derive_seed(1,7)");
        assert_ne!(v1, v2, "derive_seed(1,7) must differ from derive_seed(2,7)");
        assert_ne!(v0, v2, "derive_seed(0,7) must differ from derive_seed(2,7)");
    }

    #[test]
    fn max_parent_and_discriminator_do_not_panic_and_differ_from_zero() {
        let v_max = derive_seed(u64::MAX, u64::MAX);
        let v0 = derive_seed(0, 0);
        assert_ne!(
            v_max, v0,
            "derive_seed(u64::MAX, u64::MAX) must differ from derive_seed(0,0)"
        );
    }

    #[test]
    fn boundary_pairs_do_not_panic() {
        // B4: total function — none of these must panic
        let _ = derive_seed(0, 0);
        let _ = derive_seed(0, u64::MAX);
        let _ = derive_seed(u64::MAX, 0);
        let _ = derive_seed(u64::MAX, u64::MAX);
        let _ = derive_seed(1, u64::MAX - 1);
        let _ = derive_seed(u64::MAX - 1, 1);
    }
}
