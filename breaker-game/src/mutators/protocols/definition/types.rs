//! Protocol definition types — `ProtocolKind`, `ProtocolTuning`, `ProtocolDefinition`.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::effect_v3::types::RootNode;

/// Identifies a protocol by its variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtocolKind {
    /// Deadline — node-timer protocol.
    Deadline,
    /// Ricochet — bolt-reflection protocol.
    Ricochet,
    /// Anchor — breaker-positioning protocol.
    Anchor,
    /// Kickstart — run-launch protocol.
    Kickstart,
    /// `DebtCollector` — bump-stack protocol.
    DebtCollector,
    /// `IronCurtain` — damage-fraction protocol.
    IronCurtain,
    /// `EchoStrike` — echo-damage protocol.
    EchoStrike,
    /// Siphon — kill-streak-time protocol.
    Siphon,
    /// Greed — rarity-weighted chip-offer protocol.
    Greed,
    /// `RecklessDash` — risky-zone damage protocol.
    RecklessDash,
    /// Burnout — heat accumulation protocol.
    Burnout,
    /// Conductor — primary-swap protocol.
    Conductor,
    /// Afterimage — phantom-bolt protocol.
    Afterimage,
    /// Fission — kill-split protocol.
    Fission,
    /// `TierRegression` — tier-back protocol.
    TierRegression,
}

impl ProtocolKind {
    /// Canonical slice of every variant.
    pub const ALL: &[Self] = &[
        Self::Deadline,
        Self::Ricochet,
        Self::Anchor,
        Self::Kickstart,
        Self::DebtCollector,
        Self::IronCurtain,
        Self::EchoStrike,
        Self::Siphon,
        Self::Greed,
        Self::RecklessDash,
        Self::Burnout,
        Self::Conductor,
        Self::Afterimage,
        Self::Fission,
        Self::TierRegression,
    ];

    /// Returns the lowercase, underscore-separated slug used in `SourceId`
    /// construction (`"protocol:<slug>"`).
    #[must_use]
    pub const fn kind_slug(&self) -> &'static str {
        match self {
            Self::Deadline => "deadline",
            Self::Ricochet => "ricochet",
            Self::Anchor => "anchor",
            Self::Kickstart => "kickstart",
            Self::DebtCollector => "debt_collector",
            Self::IronCurtain => "iron_curtain",
            Self::EchoStrike => "echo_strike",
            Self::Siphon => "siphon",
            Self::Greed => "greed",
            Self::RecklessDash => "reckless_dash",
            Self::Burnout => "burnout",
            Self::Conductor => "conductor",
            Self::Afterimage => "afterimage",
            Self::Fission => "fission",
            Self::TierRegression => "tier_regression",
        }
    }
}

/// Per-protocol tuning data carried by [`ProtocolDefinition`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProtocolTuning {
    // ── Effect-tree protocols (effects() returns Some) ──
    /// Deadline — node-timer protocol driven by an effect tree.
    Deadline {
        /// Effect-tree roots stamped onto each breaker at activation.
        effects: Vec<RootNode>,
    },
    /// Ricochet — bolt-reflection protocol driven by an effect tree.
    Ricochet {
        /// Effect-tree roots stamped onto each breaker at activation.
        effects: Vec<RootNode>,
    },
    /// Anchor — breaker-positioning protocol driven by an effect tree.
    Anchor {
        /// Effect-tree roots stamped onto each breaker at activation.
        effects: Vec<RootNode>,
    },
    /// Kickstart — run-launch protocol driven by an effect tree.
    Kickstart {
        /// Effect-tree roots stamped onto each breaker at activation.
        effects: Vec<RootNode>,
    },

    // ── Custom-system protocols (effects() returns None) ──
    /// `DebtCollector` — bump-stack protocol; damage accrues per bump and
    /// cashes out on a Perfect.
    DebtCollector {
        /// Fraction of a damage stack added per bump.
        stack_per_bump: f32,
    },
    /// `IronCurtain` — damage-fraction protocol; emits a damage wave when a
    /// bolt is lost.
    IronCurtain {
        /// Fraction of node damage dealt by the wave.
        damage_fraction: f32,
        /// Distance from breaker at which falloff begins (0.0–1.0 of
        /// playfield height).
        falloff_start:   f32,
    },
    /// `EchoStrike` — echo-damage protocol; Perfect Bumps build an echo
    /// network that fires on cell hits.
    EchoStrike {
        /// Maximum echoes the network can hold.
        max_echoes:      u32,
        /// Damage fraction for the newest echo.
        newest_fraction: f32,
        /// Damage fraction for middle echoes.
        middle_fraction: f32,
        /// Damage fraction for the oldest echo.
        oldest_fraction: f32,
    },
    /// Siphon — kill-streak-time protocol; consecutive cell kills within a
    /// window award bonus node time.
    Siphon {
        /// Seconds after the last kill before the streak expires.
        streak_window: f32,
        /// Seconds added to the node timer per streak kill.
        time_per_kill: f32,
    },
    /// Greed — rarity-weighted chip-offer protocol; skipping offers boosts
    /// the rare-rarity weight next visit.
    Greed {
        /// Weight shift per skip, expressed as a fraction (e.g. 0.05 =
        /// +5% per skip).
        rarity_boost_per_skip: f32,
    },
    /// `RecklessDash` — risky-zone damage protocol; bumping inside the danger
    /// zone multiplies damage, but bolt loss doubles.
    RecklessDash {
        /// Y threshold (0.0–1.0 of playfield height) below which bumps
        /// are in the risky zone.
        risky_zone_start:  f32,
        /// Damage multiplier applied to bumps in the risky zone.
        damage_multiplier: f32,
        /// When `true`, losing a bolt while in the risky zone doubles the
        /// bolt-loss penalty.
        double_penalty:    bool,
    },
    /// Burnout — heat accumulation protocol; sustained dashing builds heat
    /// that boosts damage at full heat, then vents with a speed burst.
    Burnout {
        /// Seconds of continuous dashing required to reach full heat.
        fill_duration:               f32,
        /// Seconds at rest required to drain all heat after venting.
        drain_duration:              f32,
        /// Seconds the breaker must be still before heat drains.
        still_threshold:             f32,
        /// Damage multiplier applied at full heat.
        full_heat_damage_multiplier: f32,
        /// Duration of the speed burst emitted when heat vents.
        speed_boost_duration:        f32,
    },
    /// Conductor — primary-swap protocol; no per-protocol tuning fields.
    Conductor,
    /// Afterimage — phantom-bolt protocol; destroyed bolts leave a phantom
    /// that continues for a short time.
    Afterimage {
        /// Seconds the phantom bolt persists after the real bolt is lost.
        phantom_duration:      f32,
        /// Seconds the phantom bolt itself remains active.
        phantom_bolt_duration: f32,
    },
    /// Fission — kill-split protocol; every Nth cell kill splits a bolt
    /// into two.
    Fission {
        /// Number of cell kills required to trigger a split.
        kills_per_split: u32,
    },
    /// `TierRegression` — tier-back protocol; chip offers shift down by a
    /// number of rarity tiers.
    TierRegression {
        /// Number of rarity tiers to shift down on each offer.
        tiers_back: u32,
    },
}

impl ProtocolTuning {
    /// Returns the [`ProtocolKind`] associated with this tuning.
    #[must_use]
    pub const fn kind(&self) -> ProtocolKind {
        match self {
            Self::Deadline { .. } => ProtocolKind::Deadline,
            Self::Ricochet { .. } => ProtocolKind::Ricochet,
            Self::Anchor { .. } => ProtocolKind::Anchor,
            Self::Kickstart { .. } => ProtocolKind::Kickstart,
            Self::DebtCollector { .. } => ProtocolKind::DebtCollector,
            Self::IronCurtain { .. } => ProtocolKind::IronCurtain,
            Self::EchoStrike { .. } => ProtocolKind::EchoStrike,
            Self::Siphon { .. } => ProtocolKind::Siphon,
            Self::Greed { .. } => ProtocolKind::Greed,
            Self::RecklessDash { .. } => ProtocolKind::RecklessDash,
            Self::Burnout { .. } => ProtocolKind::Burnout,
            Self::Conductor => ProtocolKind::Conductor,
            Self::Afterimage { .. } => ProtocolKind::Afterimage,
            Self::Fission { .. } => ProtocolKind::Fission,
            Self::TierRegression { .. } => ProtocolKind::TierRegression,
        }
    }

    /// Returns the effect tree for effect-tree protocols; `None` for
    /// custom-system protocols.
    #[must_use]
    pub const fn effects(&self) -> Option<&[RootNode]> {
        match self {
            Self::Deadline { effects }
            | Self::Ricochet { effects }
            | Self::Anchor { effects }
            | Self::Kickstart { effects } => Some(effects.as_slice()),
            _ => None,
        }
    }
}

/// A complete protocol definition — the payload loaded from `.protocol.ron`
/// files and stored in [`crate::mutators::protocols::resources::ProtocolRegistry`].
#[derive(Asset, TypePath, Clone, Debug, Serialize, Deserialize)]
pub struct ProtocolDefinition {
    /// Display name shown on the protocol card.
    pub name:        String,
    /// Flavor text shown below the name.
    pub description: String,
    /// Meta-progression unlock tier. Defaults to 0 (always unlocked).
    #[serde(default)]
    pub unlock_tier: u32,
    /// Per-protocol tuning data.
    pub tuning:      ProtocolTuning,
}

impl ProtocolDefinition {
    /// Returns the [`ProtocolKind`] of the inner tuning.
    #[must_use]
    pub const fn kind(&self) -> ProtocolKind {
        self.tuning.kind()
    }
}
