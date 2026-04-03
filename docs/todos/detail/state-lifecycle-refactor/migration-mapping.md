# Migration Mapping — Index

This file was the original migration analysis. The detailed content has been split into dedicated files. Use those instead.

## Detail Files

| File | Contains |
|------|----------|
| [implementation-waves.md](implementation-waves.md) | Ordered waves (branches), sub-waves, parallelism |
| [system-moves.md](system-moves.md) | Every system, whether it stays or moves, where it goes |
| [system-changes.md](system-changes.md) | Systems that need merging, splitting, or rewriting |
| [post-restructure-tree.md](post-restructure-tree.md) | Expected `src/` folder tree after Wave 2 |
| [state-assignments.md](state-assignments.md) | Every system's current state vs target state (Wave 4) |
| [routing-tables.md](routing-tables.md) | Each state's routing implementation |
| [crate-design.md](crate-design.md) | rantzsoft_lifecycle crate design (written by agent — validate) |
| [crate-migration.md](crate-migration.md) | Systems that need updating for lifecycle crate (Wave 7) |
| [scenario-runner-impact.md](scenario-runner-impact.md) | Every change needed in breaker-scenario-runner (18 files) |

## Key Principles

1. **Domain systems stay in their domains** — bolt, breaker, cells, chips, effect, input, fx, audio, debug only get gate/import changes in plugin.rs
2. **Setup systems move to state/** — OnEnter systems that set up a state (reset_bolt, spawn_walls, etc.) belong to the state, not the domain
3. **run/ is absorbed into state/run/** — run is a state, not a content domain
4. **screen/ and ui/ are dissolved into state/** — each screen/UI becomes a subfolder of its owning state
5. **wall/ renamed to walls/** — plural consistency

## Scenario Runner Impact

See [scenario-runner-impact.md](scenario-runner-impact.md) for the full analysis — 18 files, ~8 path-only changes, ~6 rewrites, ~4 verifications. Key rewrites: `bypass_menu_to_playing` (navigate new state hierarchy), pause mutations (`Time<Virtual>` replaces `PlayingState`), `valid_state_transitions` invariant (new hierarchy), `entered_playing` gate (state check replaces message flag).
