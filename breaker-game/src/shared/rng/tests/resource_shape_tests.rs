use rand::Rng;

use super::super::rng_types::*;

// ── Group C — New resource types: shape, derives, defaults ─────────────

mod new_resource_shapes {
    use super::*;

    // B9 — FxRng
    #[test]
    fn fx_rng_from_seed_is_deterministic() {
        let mut a = FxRng::from_seed(7);
        let mut b = FxRng::from_seed(7);
        let va: u64 = a.0.random();
        let vb: u64 = b.0.random();
        assert_eq!(va, vb, "FxRng::from_seed must be deterministic");
    }

    #[test]
    fn fx_rng_default_does_not_panic() {
        let mut rng = FxRng::default();
        let _ = rng.0.random::<u64>();
    }

    // B10 — NodeSequenceRng
    #[test]
    fn node_sequence_rng_from_seed_is_deterministic() {
        let mut a = NodeSequenceRng::from_seed(99);
        let mut b = NodeSequenceRng::from_seed(99);
        let va: u32 = a.0.random();
        let vb: u32 = b.0.random();
        assert_eq!(va, vb, "NodeSequenceRng::from_seed must be deterministic");
    }

    #[test]
    fn node_sequence_rng_default_is_from_seed_zero() {
        let mut a = NodeSequenceRng::default();
        let mut b = NodeSequenceRng::from_seed(0);
        let va: u32 = a.0.random();
        let vb: u32 = b.0.random();
        assert_eq!(va, vb, "NodeSequenceRng::default() must equal from_seed(0)");
    }

    // B11 — NodeGenRng
    #[test]
    fn node_gen_rng_from_seed_is_deterministic() {
        let mut a = NodeGenRng::from_seed(123);
        let mut b = NodeGenRng::from_seed(123);
        let va: u64 = a.0.random();
        let vb: u64 = b.0.random();
        assert_eq!(va, vb, "NodeGenRng::from_seed must be deterministic");
    }

    #[test]
    fn node_gen_rng_default_is_from_seed_zero() {
        let mut a = NodeGenRng::default();
        let mut b = NodeGenRng::from_seed(0);
        let va: u64 = a.0.random();
        let vb: u64 = b.0.random();
        assert_eq!(va, vb, "NodeGenRng::default() must equal from_seed(0)");
    }

    // B12 — BoltRng
    #[test]
    fn bolt_rng_from_seed_is_deterministic() {
        let mut a = BoltRng::from_seed(0xCAFE);
        let mut b = BoltRng::from_seed(0xCAFE);
        let va: u64 = a.0.random();
        let vb: u64 = b.0.random();
        assert_eq!(va, vb, "BoltRng::from_seed must be deterministic");
    }

    #[test]
    fn bolt_rng_default_is_from_seed_zero() {
        let mut a = BoltRng::default();
        let mut b = BoltRng::from_seed(0);
        let va: u64 = a.0.random();
        let vb: u64 = b.0.random();
        assert_eq!(va, vb, "BoltRng::default() must equal from_seed(0)");
    }

    // B13 — ChipRng
    #[test]
    fn chip_rng_from_seed_is_deterministic() {
        let mut a = ChipRng::from_seed(7);
        let mut b = ChipRng::from_seed(7);
        let va: u64 = a.0.random();
        let vb: u64 = b.0.random();
        assert_eq!(va, vb, "ChipRng::from_seed must be deterministic");
    }

    #[test]
    fn chip_rng_default_is_from_seed_zero() {
        let mut a = ChipRng::default();
        let mut b = ChipRng::from_seed(0);
        let va: u64 = a.0.random();
        let vb: u64 = b.0.random();
        assert_eq!(va, vb, "ChipRng::default() must equal from_seed(0)");
    }

    // B14 — ProtocolRng
    #[test]
    fn protocol_rng_from_seed_is_deterministic() {
        let mut a = ProtocolRng::from_seed(11);
        let mut b = ProtocolRng::from_seed(11);
        let va: u64 = a.0.random();
        let vb: u64 = b.0.random();
        assert_eq!(va, vb, "ProtocolRng::from_seed must be deterministic");
    }

    #[test]
    fn protocol_rng_default_is_from_seed_zero() {
        let mut a = ProtocolRng::default();
        let mut b = ProtocolRng::from_seed(0);
        let va: u64 = a.0.random();
        let vb: u64 = b.0.random();
        assert_eq!(va, vb, "ProtocolRng::default() must equal from_seed(0)");
    }

    // B15 — HazardRng
    #[test]
    fn hazard_rng_from_seed_is_deterministic() {
        let mut a = HazardRng::from_seed(13);
        let mut b = HazardRng::from_seed(13);
        let va: u64 = a.0.random();
        let vb: u64 = b.0.random();
        assert_eq!(va, vb, "HazardRng::from_seed must be deterministic");
    }

    #[test]
    fn hazard_rng_default_is_from_seed_zero() {
        let mut a = HazardRng::default();
        let mut b = HazardRng::from_seed(0);
        let va: u64 = a.0.random();
        let vb: u64 = b.0.random();
        assert_eq!(va, vb, "HazardRng::default() must equal from_seed(0)");
    }

    // B16 — EffectEventCounter
    #[test]
    fn effect_event_counter_default_is_zero() {
        let a = EffectEventCounter::default();
        let b = EffectEventCounter::default();
        assert_eq!(a.0, 0, "EffectEventCounter default must be 0");
        assert_eq!(b.0, 0, "both EffectEventCounter defaults must be 0");
    }

    // B17 — EffectBaseSeed
    #[test]
    fn effect_base_seed_default_is_zero() {
        let a = EffectBaseSeed::default();
        assert_eq!(a.0, 0, "EffectBaseSeed default must be 0");
    }

    #[test]
    fn effect_base_seed_tuple_ctor_is_public() {
        let s = EffectBaseSeed(0xABCD_1234);
        assert_eq!(s.0, 0xABCD_1234);
    }

    // B18 — ChipSelectCount
    #[test]
    fn chip_select_count_default_is_zero() {
        let a = ChipSelectCount::default();
        assert_eq!(a.0, 0, "ChipSelectCount default must be 0");
    }

    #[test]
    fn chip_select_count_tuple_ctor_is_public() {
        let c = ChipSelectCount(5);
        assert_eq!(c.0, 5);
    }
}
