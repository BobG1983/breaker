# Memory

- [LivesCount is established vocabulary](stable/livescount_vocabulary.md) — LivesCount is correct project term
- [BumpFeedback rename pattern](stable/bump_feedback_rename.md) — BumpVisualParams → BumpFeedback rename; config fields still use bump_visual_ prefix
- [default_bump_visual_params helper](stable/stale_helper_name.md) — bump_visual/tests.rs has stale helper name
- [bump_visual_ prefix in BreakerDefinition](stable/bump_visual_prefix_definition.md) — Intentional; matches RON keys in BreakerConfig
- [MovementSettings missing derives](stable/movement_settings_derives.md) — MovementSettings has no derives; BumpSettings only Clone — inconsistent with DashSettings (Clone, Copy)

## Session History
See [ephemeral/](ephemeral/) — not committed.
