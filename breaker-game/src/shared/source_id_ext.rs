//! Game-side `SourceId` extension trait — typestate builder + reader helpers.
//!
//! The builder is the single legal path to constructing namespace-correct
//! `SourceId` values for protocols, hazards, chips, and armed-source plumbing.
//! Readers (`is_armed`, `extract_hazard_instance`) live alongside
//! construction so format strings remain in exactly one file.
//!
//! See `docs/architecture/source_id.md` (planned) and the W5 design doc for
//! the rationale.

use crate::{
    chips::definition::Rarity, hazard::definition::HazardKind, prelude::SourceId,
    protocol::definition::ProtocolKind,
};

// ── Trait surface ───────────────────────────────────────────────────────────

/// Game-side extension trait on [`SourceId`].
///
/// Provides typestate-builder entry points and reader helpers. Lives
/// game-side because it references `ProtocolKind`, `HazardKind`, and `Rarity`
/// — all game vocabulary that must NOT leak into `rantzsoft_dmg`.
pub(crate) trait SourceIdExt {
    /// Begin a chip-namespace source with the given template name.
    fn chip(template: impl Into<String>) -> ChipBuilder;

    /// Begin a protocol-namespace source for the given protocol kind.
    fn protocol(kind: ProtocolKind) -> ProtocolBuilder;

    /// Begin a hazard-namespace source for the given hazard kind.
    fn hazard(kind: HazardKind) -> HazardBuilder;

    /// Wrap an existing `SourceId` as an armed-firing source.
    fn armed(inner: SourceId) -> ArmedBuilder;

    /// Returns `true` if this source represents an armed-firing context.
    /// The canonical form is a trailing `":armed"` suffix.
    fn is_armed(&self) -> bool;

    /// If this source matches the form `"hazard:<kind_slug>:<u64>"`, returns
    /// the parsed instance id. Returns `None` for any mismatch (wrong kind,
    /// no instance segment, unparsable suffix, armed-wrapped form).
    fn extract_hazard_instance(&self, kind: HazardKind) -> Option<u64>;
}

// ── Builder typestate structs ───────────────────────────────────────────────

/// Chip-namespace typestate.
#[derive(Clone, Debug)]
pub(crate) struct ChipBuilder {
    template: String,
    rarity:   Option<Rarity>,
}

/// Protocol-namespace typestate.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ProtocolBuilder {
    kind:   ProtocolKind,
    action: Option<&'static str>,
}

/// Hazard-namespace typestate.
#[derive(Clone, Copy, Debug)]
pub(crate) struct HazardBuilder {
    kind:     HazardKind,
    instance: Option<u64>,
}

/// Armed-wrapper typestate.
#[derive(Clone, Debug)]
pub(crate) struct ArmedBuilder {
    inner: SourceId,
}

// ── Typed builder modifiers + build() ───────────────────────────────────────

impl ChipBuilder {
    /// Attach a rarity segment to the source.
    #[must_use]
    pub(crate) const fn rarity(mut self, rarity: Rarity) -> Self {
        self.rarity = Some(rarity);
        self
    }

    /// Build the final `SourceId`.
    ///
    /// Format:
    /// - No rarity: `"chip:<template>"`
    /// - With rarity: `"chip:<template>:<rarity>"` (uses `Rarity::Display`)
    #[must_use]
    pub(crate) fn build(self) -> SourceId {
        let s = match self.rarity {
            Some(rarity) => format!("chip:{}:{rarity}", self.template),
            None => format!("chip:{}", self.template),
        };
        SourceId::from(s)
    }
}

impl ProtocolBuilder {
    /// Attach an action segment to the source.
    #[must_use]
    pub(crate) const fn action(mut self, action: &'static str) -> Self {
        self.action = Some(action);
        self
    }

    /// Build the final `SourceId`.
    ///
    /// Format:
    /// - No action: `"protocol:<kind_slug>"`
    /// - With action: `"protocol:<kind_slug>:<action>"`
    #[must_use]
    pub(crate) fn build(self) -> SourceId {
        let slug = self.kind.kind_slug();
        let s = match self.action {
            Some(action) => format!("protocol:{slug}:{action}"),
            None => format!("protocol:{slug}"),
        };
        SourceId::from(s)
    }
}

impl HazardBuilder {
    /// Attach an instance id segment to the source.
    #[must_use]
    pub(crate) const fn instance(mut self, instance: u64) -> Self {
        self.instance = Some(instance);
        self
    }

    /// Build the final `SourceId`.
    ///
    /// Format:
    /// - No instance: `"hazard:<kind_slug>"`
    /// - With instance: `"hazard:<kind_slug>:<n>"`
    #[must_use]
    pub(crate) fn build(self) -> SourceId {
        let slug = self.kind.kind_slug();
        let s = match self.instance {
            Some(n) => format!("hazard:{slug}:{n}"),
            None => format!("hazard:{slug}"),
        };
        SourceId::from(s)
    }
}

impl ArmedBuilder {
    /// Build the final `SourceId`.
    ///
    /// Format: `"<inner>:armed"` — uses `SourceId::Display`.
    #[must_use]
    pub(crate) fn build(self) -> SourceId {
        SourceId::from(format!("{}:armed", self.inner))
    }
}

// ── Trait impl on `SourceId` ────────────────────────────────────────────────

impl SourceIdExt for SourceId {
    fn chip(template: impl Into<String>) -> ChipBuilder {
        ChipBuilder {
            template: template.into(),
            rarity:   None,
        }
    }

    fn protocol(kind: ProtocolKind) -> ProtocolBuilder {
        ProtocolBuilder { kind, action: None }
    }

    fn hazard(kind: HazardKind) -> HazardBuilder {
        HazardBuilder {
            kind,
            instance: None,
        }
    }

    fn armed(inner: SourceId) -> ArmedBuilder {
        ArmedBuilder { inner }
    }

    fn is_armed(&self) -> bool {
        self.0.ends_with(":armed")
    }

    fn extract_hazard_instance(&self, kind: HazardKind) -> Option<u64> {
        let prefix = format!("hazard:{}:", kind.kind_slug());
        self.0
            .as_ref()
            .strip_prefix(prefix.as_str())
            .and_then(|s| s.parse::<u64>().ok())
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Behavior 2: .chip("Piercing").build() → "chip:Piercing" ──

    #[test]
    fn chip_with_template_only_produces_namespaced_source_no_rarity() {
        let id = SourceId::chip("Piercing").build();
        assert_eq!(id.0.as_ref(), "chip:Piercing");
    }

    #[test]
    fn chip_with_empty_template_produces_chip_colon() {
        // Edge case: empty template name is allowed — produces "chip:".
        let id = SourceId::chip("").build();
        assert_eq!(id.0.as_ref(), "chip:");
    }

    // ── Behavior 3: .chip(name).rarity(r).build() → "chip:<name>:<rarity>" ──

    #[test]
    fn chip_with_template_and_common_rarity_produces_three_segment_source() {
        let id = SourceId::chip("Pulse").rarity(Rarity::Common).build();
        assert_eq!(id.0.as_ref(), "chip:Pulse:Common");
    }

    #[test]
    fn chip_with_uncommon_rarity_uses_uncommon_display() {
        let id = SourceId::chip("Pulse").rarity(Rarity::Uncommon).build();
        assert_eq!(id.0.as_ref(), "chip:Pulse:Uncommon");
    }

    #[test]
    fn chip_with_rare_rarity_uses_rare_display() {
        let id = SourceId::chip("Pulse").rarity(Rarity::Rare).build();
        assert_eq!(id.0.as_ref(), "chip:Pulse:Rare");
    }

    #[test]
    fn chip_with_evolution_rarity_uses_evolution_display() {
        let id = SourceId::chip("Supernova")
            .rarity(Rarity::Evolution)
            .build();
        assert_eq!(id.0.as_ref(), "chip:Supernova:Evolution");
    }

    // ── Behavior 4: multi-word template name preserves spaces and case ──

    #[test]
    fn chip_with_multiword_template_preserves_spaces_and_case() {
        let id = SourceId::chip("Shrapnel Core")
            .rarity(Rarity::Evolution)
            .build();
        assert_eq!(id.0.as_ref(), "chip:Shrapnel Core:Evolution");
    }

    #[test]
    fn chip_with_trailing_whitespace_in_template_is_verbatim() {
        // Edge case: caller passes unclean name; builder does NOT trim.
        let id = SourceId::chip("Piercing ").rarity(Rarity::Common).build();
        assert_eq!(id.0.as_ref(), "chip:Piercing :Common");
    }

    // ── Behavior 5: protocol() format pin per variant ──

    #[test]
    fn protocol_burnout_produces_protocol_burnout() {
        let id = SourceId::protocol(ProtocolKind::Burnout).build();
        assert_eq!(id.0.as_ref(), "protocol:burnout");
    }

    #[test]
    fn protocol_all_15_variants_produce_expected_slug() {
        let cases: [(ProtocolKind, &str); 15] = [
            (ProtocolKind::Deadline, "protocol:deadline"),
            (ProtocolKind::Ricochet, "protocol:ricochet"),
            (ProtocolKind::Anchor, "protocol:anchor"),
            (ProtocolKind::Kickstart, "protocol:kickstart"),
            (ProtocolKind::DebtCollector, "protocol:debt_collector"),
            (ProtocolKind::IronCurtain, "protocol:iron_curtain"),
            (ProtocolKind::EchoStrike, "protocol:echo_strike"),
            (ProtocolKind::Siphon, "protocol:siphon"),
            (ProtocolKind::Greed, "protocol:greed"),
            (ProtocolKind::RecklessDash, "protocol:reckless_dash"),
            (ProtocolKind::Burnout, "protocol:burnout"),
            (ProtocolKind::Conductor, "protocol:conductor"),
            (ProtocolKind::Afterimage, "protocol:afterimage"),
            (ProtocolKind::Fission, "protocol:fission"),
            (ProtocolKind::TierRegression, "protocol:tier_regression"),
        ];
        for (kind, expected) in cases {
            let id = SourceId::protocol(kind).build();
            assert_eq!(id.0.as_ref(), expected, "kind = {kind:?}");
        }
    }

    // ── Behavior 6: protocol().action(...) format pin ──

    #[test]
    fn protocol_with_action_produces_three_segment_source() {
        let id = SourceId::protocol(ProtocolKind::Burnout)
            .action("shockwave")
            .build();
        assert_eq!(id.0.as_ref(), "protocol:burnout:shockwave");
    }

    #[test]
    fn protocol_with_empty_action_produces_trailing_colon() {
        // Edge case: empty action produces "protocol:burnout:" verbatim.
        let id = SourceId::protocol(ProtocolKind::Burnout).action("").build();
        assert_eq!(id.0.as_ref(), "protocol:burnout:");
    }

    #[test]
    fn protocol_with_spaced_action_is_verbatim() {
        // Edge case: action with spaces — opaque, no normalization.
        let id = SourceId::protocol(ProtocolKind::Burnout)
            .action("two words")
            .build();
        assert_eq!(id.0.as_ref(), "protocol:burnout:two words");
    }

    // ── Behavior 7: hazard() format pin per variant ──

    #[test]
    fn hazard_tether_produces_hazard_tether() {
        let id = SourceId::hazard(HazardKind::Tether).build();
        assert_eq!(id.0.as_ref(), "hazard:tether");
    }

    #[test]
    fn hazard_all_16_variants_produce_expected_slug() {
        let cases: [(HazardKind, &str); 16] = [
            (HazardKind::Decay, "hazard:decay"),
            (HazardKind::Drift, "hazard:drift"),
            (HazardKind::Haste, "hazard:haste"),
            (HazardKind::EchoCells, "hazard:echo_cells"),
            (HazardKind::Erosion, "hazard:erosion"),
            (HazardKind::Cascade, "hazard:cascade"),
            (HazardKind::Fracture, "hazard:fracture"),
            (HazardKind::Renewal, "hazard:renewal"),
            (HazardKind::Volatility, "hazard:volatility"),
            (HazardKind::GravitySurge, "hazard:gravity_surge"),
            (HazardKind::Overcharge, "hazard:overcharge"),
            (HazardKind::Resonance, "hazard:resonance"),
            (HazardKind::Diffusion, "hazard:diffusion"),
            (HazardKind::Tether, "hazard:tether"),
            (HazardKind::Momentum, "hazard:momentum"),
            (HazardKind::Sympathy, "hazard:sympathy"),
        ];
        for (kind, expected) in cases {
            let id = SourceId::hazard(kind).build();
            assert_eq!(id.0.as_ref(), expected, "kind = {kind:?}");
        }
    }

    // ── Behavior 8: hazard().instance(n) format pin ──

    #[test]
    fn hazard_diffusion_with_instance_42_produces_three_segment_source() {
        let id = SourceId::hazard(HazardKind::Diffusion).instance(42).build();
        assert_eq!(id.0.as_ref(), "hazard:diffusion:42");
    }

    #[test]
    fn hazard_diffusion_with_instance_zero_includes_zero_segment() {
        // Edge case: instance 0 is NOT the same as no instance.
        let id = SourceId::hazard(HazardKind::Diffusion).instance(0).build();
        assert_eq!(id.0.as_ref(), "hazard:diffusion:0");
    }

    #[test]
    fn hazard_diffusion_with_instance_u64_max_produces_full_decimal() {
        let id = SourceId::hazard(HazardKind::Diffusion)
            .instance(u64::MAX)
            .build();
        assert_eq!(id.0.as_ref(), "hazard:diffusion:18446744073709551615");
    }

    #[test]
    fn hazard_tether_with_instance_is_structurally_allowed() {
        // Edge case: typestate doesn't restrict instance to Diffusion only —
        // the semantic restriction is at runtime.
        let id = SourceId::hazard(HazardKind::Tether).instance(7).build();
        assert_eq!(id.0.as_ref(), "hazard:tether:7");
    }

    // ── Behavior 9: armed(inner).build() → "<inner>:armed" ──

    #[test]
    fn armed_wraps_chip_source_with_armed_suffix() {
        let inner = SourceId::from(String::from("chip:Piercing:Common"));
        let armed = SourceId::armed(inner).build();
        assert_eq!(armed.0.as_ref(), "chip:Piercing:Common:armed");
    }

    #[test]
    fn armed_wraps_protocol_source() {
        let inner = SourceId::from(String::from("protocol:burnout"));
        let armed = SourceId::armed(inner).build();
        assert_eq!(armed.0.as_ref(), "protocol:burnout:armed");
    }

    #[test]
    fn armed_wraps_hazard_with_instance_source() {
        let inner = SourceId::from(String::from("hazard:diffusion:42"));
        let armed = SourceId::armed(inner).build();
        assert_eq!(armed.0.as_ref(), "hazard:diffusion:42:armed");
    }

    #[test]
    fn armed_wraps_empty_source_to_colon_armed() {
        // Edge case: degenerate empty inner — no panic.
        let inner = SourceId::from(String::new());
        let armed = SourceId::armed(inner).build();
        assert_eq!(armed.0.as_ref(), ":armed");
    }

    #[test]
    fn armed_double_wrap_is_not_idempotent() {
        // Edge case: armed-wrapping an already-armed source produces ":armed:armed".
        let inner = SourceId::from(String::from("chip:X:armed"));
        let armed = SourceId::armed(inner).build();
        assert_eq!(armed.0.as_ref(), "chip:X:armed:armed");
    }

    // ── Behavior 16: is_armed() on non-armed sources → false ──

    #[test]
    fn is_armed_returns_false_on_non_armed_source() {
        let id = SourceId::from("chip:Piercing");
        assert!(!id.is_armed());
    }

    #[test]
    fn is_armed_returns_false_on_empty_source() {
        let id = SourceId::from("");
        assert!(!id.is_armed());
    }

    #[test]
    fn is_armed_returns_false_on_bare_armed_no_colon() {
        // Edge case: `"armed"` (no leading `:`) — not armed.
        let id = SourceId::from("armed");
        assert!(!id.is_armed());
    }

    #[test]
    fn is_armed_returns_false_on_armed_substring_in_middle() {
        // Edge case: "armed" appearing mid-string is not the suffix.
        let id = SourceId::from("chip:armed:foo");
        assert!(!id.is_armed());
    }

    // ── Behavior 17: is_armed() on armed sources → true ──

    #[test]
    fn is_armed_returns_true_on_armed_chip_source() {
        let id = SourceId::from(String::from("chip:Piercing:Common:armed"));
        assert!(id.is_armed());
    }

    #[test]
    fn is_armed_returns_true_on_degenerate_colon_armed() {
        // Edge case: bare ":armed" is still armed.
        let id = SourceId::from(String::from(":armed"));
        assert!(id.is_armed());
    }

    #[test]
    fn is_armed_round_trips_through_builder() {
        // Edge case: any builder-produced armed source must report is_armed.
        let inner = SourceId::from(String::from("chip:X"));
        let armed = SourceId::armed(inner).build();
        assert!(armed.is_armed());
    }

    // ── Behavior 20: extract_hazard_instance parses the trailing u64 ──

    #[test]
    fn extract_hazard_instance_returns_some_for_diffusion_with_instance() {
        let id = SourceId::hazard(HazardKind::Diffusion).instance(42).build();
        assert_eq!(id.extract_hazard_instance(HazardKind::Diffusion), Some(42));
    }

    #[test]
    fn extract_hazard_instance_for_zero() {
        let id = SourceId::hazard(HazardKind::Diffusion).instance(0).build();
        assert_eq!(id.extract_hazard_instance(HazardKind::Diffusion), Some(0));
    }

    #[test]
    fn extract_hazard_instance_for_u64_max() {
        let id = SourceId::hazard(HazardKind::Diffusion)
            .instance(u64::MAX)
            .build();
        assert_eq!(
            id.extract_hazard_instance(HazardKind::Diffusion),
            Some(u64::MAX)
        );
    }

    #[test]
    fn extract_hazard_instance_round_trips_for_multiple_values() {
        for n in [0_u64, 1, 42, u64::MAX] {
            let id = SourceId::hazard(HazardKind::Diffusion).instance(n).build();
            assert_eq!(
                id.extract_hazard_instance(HazardKind::Diffusion),
                Some(n),
                "n = {n}"
            );
        }
    }

    // ── Behavior 21: extract_hazard_instance returns None on no-instance form ──

    #[test]
    fn extract_hazard_instance_returns_none_on_no_instance() {
        let id = SourceId::hazard(HazardKind::Diffusion).build();
        assert_eq!(id.extract_hazard_instance(HazardKind::Diffusion), None);
    }

    #[test]
    fn extract_hazard_instance_returns_none_on_trailing_colon_no_number() {
        let id = SourceId::from("hazard:diffusion:");
        assert_eq!(id.extract_hazard_instance(HazardKind::Diffusion), None);
    }

    #[test]
    fn extract_hazard_instance_returns_none_on_non_numeric_suffix() {
        let id = SourceId::from("hazard:diffusion:not_a_number");
        assert_eq!(id.extract_hazard_instance(HazardKind::Diffusion), None);
    }

    #[test]
    fn extract_hazard_instance_returns_none_on_negative_suffix() {
        let id = SourceId::from("hazard:diffusion:-1");
        assert_eq!(id.extract_hazard_instance(HazardKind::Diffusion), None);
    }

    // ── Behavior 22: extract_hazard_instance returns None for mismatched kind ──

    #[test]
    fn extract_hazard_instance_returns_none_for_kind_mismatch() {
        let id = SourceId::hazard(HazardKind::Tether).instance(7).build();
        assert_eq!(id.extract_hazard_instance(HazardKind::Diffusion), None);
    }

    #[test]
    fn extract_hazard_instance_returns_some_for_same_kind() {
        let id = SourceId::hazard(HazardKind::Tether).instance(7).build();
        assert_eq!(id.extract_hazard_instance(HazardKind::Tether), Some(7));
    }

    #[test]
    fn extract_hazard_instance_returns_none_for_non_hazard_source() {
        let id = SourceId::from("chip:Piercing");
        assert_eq!(id.extract_hazard_instance(HazardKind::Diffusion), None);
    }

    #[test]
    fn extract_hazard_instance_returns_none_for_armed_wrapped_hazard() {
        // Edge case: armed-wrapped hazard has trailing ":armed", not a u64.
        let id = SourceId::from(String::from("hazard:diffusion:42:armed"));
        assert_eq!(id.extract_hazard_instance(HazardKind::Diffusion), None);
    }

    // ── Behavior 23: hazard instance round-trip across all 16 variants ──

    #[test]
    fn hazard_instance_round_trips_for_every_kind_and_value() {
        for kind in HazardKind::ALL {
            for n in [0_u64, 1, 42, u64::MAX] {
                let id = SourceId::hazard(*kind).instance(n).build();
                assert_eq!(
                    id.extract_hazard_instance(*kind),
                    Some(n),
                    "kind={kind:?}, n={n}"
                );
            }
        }
    }

    // ── Behavior 24: armed round-trip across builder kinds (via is_armed) ──

    #[test]
    fn armed_round_trips_chip_inner() {
        let inner = SourceId::chip("X").rarity(Rarity::Common).build();
        let armed = SourceId::armed(inner).build();
        assert!(armed.is_armed());
        assert_eq!(armed.0.as_ref(), "chip:X:Common:armed");
    }

    #[test]
    fn armed_round_trips_protocol_inner() {
        let inner = SourceId::protocol(ProtocolKind::Burnout).build();
        let armed = SourceId::armed(inner).build();
        assert!(armed.is_armed());
        assert_eq!(armed.0.as_ref(), "protocol:burnout:armed");
    }

    #[test]
    fn armed_round_trips_hazard_inner() {
        let inner = SourceId::hazard(HazardKind::Tether).build();
        let armed = SourceId::armed(inner).build();
        assert!(armed.is_armed());
        assert_eq!(armed.0.as_ref(), "hazard:tether:armed");
    }

    #[test]
    fn armed_round_trips_hazard_with_instance_inner() {
        let inner = SourceId::hazard(HazardKind::Diffusion).instance(42).build();
        let armed = SourceId::armed(inner).build();
        assert!(armed.is_armed());
        assert_eq!(armed.0.as_ref(), "hazard:diffusion:42:armed");
    }
}
