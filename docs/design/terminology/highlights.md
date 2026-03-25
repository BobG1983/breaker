# Run Stats & Highlights

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **RunStats** | Resource accumulating gameplay statistics for the current run. Displayed on the run-end screen. | `RunStats`, `run/resources.rs` |
| **HighlightTracker** | Tracking state resource for highlight detection. Per-node fields reset each node; cross-node fields persist across the run. | `HighlightTracker`, `reset_highlight_tracker` |
| **HighlightKind** | Enum of 15 memorable run moment categories. All thresholds configurable via `defaults.highlights.ron`. | `HighlightKind`, `RunHighlight` |
| **RunHighlight** | A single recorded highlight moment with `kind`, `node_index`, and `value`. | `RunHighlight`, `RunStats::highlights` |
| **HighlightDefaults** | RON asset type holding all highlight detection thresholds and the `highlight_cap`. | `HighlightDefaults`, `HighlightConfig` |
| **HighlightTriggered** | Message emitted by highlight detection systems each time a memorable moment fires. | `HighlightTriggered`, `run/messages.rs` |
