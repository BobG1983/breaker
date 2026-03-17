# Agent Memory Organization

Agent memory directories live at `.claude/agent-memory/<agent-name>/`.

## Stable (root of agent memory dir — committed)

Put here: patterns, verified API facts, system maps, message inventories, known-conflicts lists — anything that applies to future sessions beyond the current one.

Examples: `pattern_*.md`, `system-map.md`, `message-flow.md`, `known-conflicts.md`, `keyboard_input.md`

## Ephemeral (ephemeral/ subdirectory — gitignored)

Put here: session review outputs, validation state, one-off analyses, date-stamped run notes — anything that decays after the session ends.

Examples: `review-2026-03-17.md`, `VALIDATION_SESSION.md`, `validation-history.md`, phase review snapshots

## MEMORY.md

The index file. Stays at root (committed). Link stable files individually. For ephemeral, one line suffices:

```markdown
## Session History
See [ephemeral/](ephemeral/) — not committed.
```

## Rule of thumb

If you'd want it on a fresh clone: stable. If it describes what happened today: ephemeral. When unsure: ephemeral first, promote later.
