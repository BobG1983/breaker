# Orchestrator Memory

## Routing Patterns
- [named-agents-rule.md](named-agents-rule.md) — Always use named subagents, never run own versions
- [writer-tests-boundary.md](writer-tests-boundary.md) — writer-tests must only write failing tests, never implement (promoted to `.claude/rules/tdd.md`)

## Spec Patterns
- [query-placement-rule.md](query-placement-rule.md) — Query type aliases live in domain/queries.rs
- [test-content-rule.md](test-content-rule.md) — Tests create own data; only one RON parse test per folder

## TDD
- RED gate rule defined in `.claude/rules/tdd.md` — orchestrator MUST verify tests compile and fail before launching writer-code

## Domain Quirks

## Cross-References
When the main agent needs Bevy API knowledge or architecture patterns during spec review or wiring, read the relevant researcher's memory (`.claude/agent-memory/researcher-bevy-api/`, `.claude/agent-memory/researcher-system-dependencies/`) rather than maintaining a duplicate.

## Session History
See [ephemeral/](ephemeral/) — not committed.
