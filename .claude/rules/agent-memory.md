# Agent Memory

All sub-agents have persistent memory. Read and follow this rule.

## Directory

Your memory directory is `.claude/agent-memory/<your-agent-name>/`. Its contents persist across conversations.

## Stable vs Ephemeral

**Stable** (root of your memory dir — committed): patterns, verified API facts, system maps, message inventories, known-conflicts lists — anything that applies to future sessions beyond the current one.

Examples: `pattern_*.md`, `system-map.md`, `message-flow.md`, `known-conflicts.md`

**Ephemeral** (`ephemeral/` subdirectory — gitignored): session review outputs, validation state, one-off analyses, date-stamped run notes — anything that decays after the session ends. Always save session-specific outputs here, not in the memory root.

Examples: `review-2026-03-17.md`, `VALIDATION_SESSION.md`, phase review snapshots

**Rule of thumb**: If you'd want it on a fresh clone: stable. If it describes what happened today: ephemeral. When unsure: ephemeral first, promote later.

## What NOT to Save

- Generic advice (Rust patterns, Bevy optimization, etc.) — you can look these up
- Anything that duplicates `CLAUDE.md`, `docs/architecture/`, or `docs/design/`
- Current file sizes, line counts, or other values that go stale immediately — recompute each run

## MEMORY.md

MEMORY.md is the index file at the root of your memory directory. It is loaded into your system prompt on each run.

- Only links to memory files with brief descriptions — no inline content
- Keep under 200 lines (truncated after that)
- Link stable files individually; for ephemeral, one line suffices:

```markdown
## Session History
See [ephemeral/](ephemeral/) — not committed.
```

## Using Memory

Consult your memory files at the start of each run to build on previous experience. When you discover patterns, confirmed behaviors, or recurring issues worth remembering, record them in stable memory. Update or remove memories that turn out to be wrong or outdated.
