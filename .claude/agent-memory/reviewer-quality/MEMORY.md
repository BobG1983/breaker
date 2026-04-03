# Memory

- [LivesCount is established vocabulary](stable/livescount_vocabulary.md) — LivesCount is correct project term
- [BumpFeedback rename pattern](stable/bump_feedback_rename.md) — BumpVisualParams → BumpFeedback rename; config fields still use bump_visual_ prefix
- [default_bump_visual_params helper](stable/stale_helper_name.md) — bump_visual/tests.rs has stale helper name
- [bump_visual_ prefix in BreakerDefinition](stable/bump_visual_prefix_definition.md) — Intentional; matches RON keys in BreakerConfig
- [MovementSettings missing derives](stable/movement_settings_derives.md) — MovementSettings has no derives; BumpSettings only Clone — inconsistent with DashSettings (Clone, Copy)

- [Default impl must call private serde-default fns](stable/default_impl_vs_serde_default_fns.md) — BreakerDefinition pattern (call default_* fns); WallDefinition inline is wrong

- [pub(super) vs pub(crate) in systems/mod.rs](stable/pub_super_vs_pub_crate_systems.md) — plain `pub use` for system fns is too wide; use pub(super) or pub(crate) only
- [color_from_rgb canonical location](stable/color_from_rgb_canonical.md) — lives in shared/color.rs; chip_select/mod.rs has a stale duplicate

## Session History
See [ephemeral/](ephemeral/) — not committed.
