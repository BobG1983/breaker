# Save/Load System

## Summary
Add save and load functionality so players can persist and resume runs.

## Context
The game will need save/load for roguelite progression — players need to be able to quit mid-run and resume later. Loading a save produces configs with modified values (upgraded stats, applied chips, run state), which is why the builder config refactor (todo #2) is a prerequisite.

Save/load was discussed in the context of why builders need `config()` instead of `definition()` — the save system will produce configs that include runtime modifications from a saved run.

## Scope
- In: Save format design, serialization/deserialization, save triggers (auto-save, manual), load flow (resume run from save), what state needs persisting (run seed, node progress, chip inventory, entity configs)
- Out: Cloud saves, multiple save slots (start with single slot)

## Dependencies
- Depends on: Builder config refactor (todo #2) — builders must accept configs for save-derived entity spawning
- Depends on: Phase 8 roguelite progression — meta-progression defines what persists across runs vs within runs
- Blocks: Nothing directly

## Notes
- Needs to decide: what serialization format? RON (consistent with asset pipeline) vs bincode (compact, fast)?
- Run seed + decisions made = deterministic replay possible? Or do we save full state?
- Save corruption handling — what happens if a save is invalid?

## Status
`[NEEDS DETAIL]` — Missing: save format design, what state to persist, serialization approach, save/load UI flow, interaction with roguelite meta-progression
