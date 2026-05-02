# Failure Routing

> **Team mode**: when an agent team is active (see `.claude/rules/team-mode.md`), runner-cargo's RED/GREEN gate FAIL replies route directly to the appropriate fixer (writer-tests for RED, writer-code for GREEN) AND team-lead, bypassing the orchestrator-driven routing below. The categorization in this file (which finding goes to which fixer) is still authoritative — runner-cargo's reply rule encodes it.

Read this when a verification agent reports a failure. Each failure type routes to a different fix path.

See `.claude/rules/hint-formats.md` for the standardized hint block formats that agents emit.
See `.claude/rules/routing-repeated-failures.md` for when to stop retrying and escalate (3 attempts → `/investigate`).

## Hint Passthrough Rule

When routing failures to writer-code or writer-tests, pass the runner/reviewer agent's hint blocks **verbatim** — do NOT rewrite them. The hint formats are standardized so downstream agents can consume them directly. The main agent triages (decides which hints to act on and which to dismiss) but does not rephrase the hints themselves.

## runner-cargo lint failures

| Failure type | Route |
|---|---|
| Clippy errors | Fix spec hint → **writer-code** (no writer-tests needed) |
| Format failures | runner-cargo auto-formats — no further routing needed |

## runner-cargo test failures

| Failure type | Route |
|---|---|
| Existing test broke | Fix spec hint → **writer-code** (test exists, skip writer-tests) |
| Build failure (compiler error) | hint → **researcher-rust** → **writer-code** |
| No test exists for broken behavior | hint → **writer-tests** (regression spec) → **writer-code** |

## runner-cargo scenario failures

| Confidence | Route |
|---|---|
| High | Regression spec hint → **writer-tests** → **writer-code** |
| Low | `/investigate` (spawns `debugger`) → regression spec hint → **writer-tests** → **writer-code** |

## reviewer-correctness bugs

| Confidence | Route |
|---|---|
| High | Regression spec hint → **writer-tests** → **writer-code** |
| Low | `/investigate` (spawns `debugger`) → regression spec hint → **writer-tests** → **writer-code** |

## reviewer-completeness findings

| Category | Route |
|---|---|
| MISSING | Spec gap → **planning-writer-specs-tests** + **planning-writer-specs-code** (add missing item) → full RED/GREEN cycle |
| PARTIAL | Fix spec hint → **writer-code** (complete the stub/wiring) |
| SCOPE_NARROWED | Orchestrator reviews — if legitimately in scope, treat as MISSING. If descoped by user, add decision revision to session-state |
| DIVERGED | Orchestrator reviews — if undocumented, either revert to plan or add decision revision to session-state with user approval |

## reviewer-quality and reviewer-bevy-api findings

| Finding type | Route |
|---|---|
| Style/idiom issue (reviewer-quality) | Main agent fixes inline — low risk, no test needed |
| Deprecated API (reviewer-bevy-api) | Main agent fixes inline — follow stated replacement pattern |
| Logic-adjacent issue (wrong query filter, etc.) | Treat as correctness issue — write regression spec if testable |

## guard-security findings

| Severity | Route |
|---|---|
| Critical (unsound unsafe, known CVE) | Main agent writes regression spec → **writer-tests** → **writer-code** |
| Warning (unwrap on file data, unused deps) | Main agent fixes inline |
| Info (observation for future phases) | Note and move on |

## guard-dependencies findings

| Category | Route |
|---|---|
| Unused dependency | Main agent removes from Cargo.toml |
| Outdated (security patch) | Main agent bumps version |
| Outdated (feature update) | Main agent evaluates and bumps if appropriate |
| License issue | Main agent evaluates — may need to replace the crate |
| Duplicate transitive | Main agent evaluates — may need to pin or unify |

## reviewer-file-length findings

reviewer-file-length returns the split plan inline (summary table + refactor spec hint per file). It does NOT write to `docs/todos/`. The orchestrator launches background sub-agents (forks — `Agent` without `subagent_type`) to perform the splits in the current branch following `.claude/rules/file-splitting.md`. Multiple splits can run in parallel. After all sub-agents complete, run Basic Verification Tier. Splits MUST land before the branch merges to develop. Do NOT defer to a todo. Do NOT route via `/implement` or `/quickfix`. If LOW-priority files are flagged, surface them to the user to decide whether to include them in the current split batch.

## reviewer-tests findings

| Finding type | Route |
|---|---|
| BLOCKING (missing behavior, production logic in stub) | Test revision spec → **writer-tests** |
| IMPORTANT (wrong values, partial coverage) | Main agent triages → **writer-tests** if warranted |
| MINOR (naming, style) | Note and proceed to RED gate |

## writer-scenarios output

| Result | Route |
|---|---|
| Scenarios created, all pass | Done — committed with the feature |
| Scenarios created, some fail | Investigate — may indicate a real bug (route to writer-tests → writer-code) |
| Compilation failure | Fix scenario code — may need **researcher-rust** |
