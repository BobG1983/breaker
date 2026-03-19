# Sub-Agent Development Flow

When to launch which agents, how to interpret their output, and how failures chain to fixes.

See @.claude/rules/delegated-implementation.md for the planner-spec → planner-review → writer-tests → writer-code pipeline.
See @.claude/rules/orchestration.md for session state, verification tiers, circuit breaking, and context pruning.
See @.claude/rules/hint-formats.md for the standardized hint block formats that Phase 2 agents produce.

## Phase 1 — Before Writing Code (sequential, blocks implementation)

| Trigger | Agent |
|---------|-------|
| Unfamiliar Bevy 0.18 API or pattern | **researcher-bevy-api** |
| Choosing between Rust idiom alternatives | **researcher-rust-idioms** |
| Feature ready for spec writing | **planner-spec** |
| Specs produced — novel mechanic, cross-domain, or uncertain scope | **planner-review** |
| planner-review found BLOCKING/IMPORTANT issues | **planner-spec** (revision — send feedback back to get corrected specs) |

### Spec Revision Loop

After planner-review produces findings, the main agent:
1. Triages findings (dismiss false positives, note valid issues)
2. Sends valid feedback back to **planner-spec** to produce corrected specs
3. Re-launches **planner-review** on the corrected specs if needed (skip if only MINOR findings remain)
4. Only proceeds to writer-tests once specs are confirmed clean

**Never launch writer-tests with unreviewed or uncorrected specs.** The cost of a bad spec propagating through writer-tests → writer-code is high (rework). The cost of one revision loop is low.

## Phase 2 + Phase 3 — REFACTOR (verification → fix routing → /simplify)

Phase 2 and Phase 3 together form the **REFACTOR** stage of the TDD cycle. Reviewers and runners identify what needs improving (Phase 2), failure routing executes the fixes (Phase 3), and `/simplify` catches anything left over. See delegated-implementation.md for the full mapping.

### Phase 2 — Post-Implementation (single parallel wave)

Launch per verification tier — see @.claude/rules/orchestration.md for Standard vs Full tier definitions.

All agents in a tier launch in a **single message** with multiple Agent tool calls. They are independent and must run in parallel — separate messages make them sequential.

### Conditional agents (add to the same single message)

| Condition | Agent |
|-----------|-------|
| 3+ systems added, or cross-plugin data flow | **researcher-system-dependencies** |
| New gameplay mechanic or upgrade designed | **guard-game-design** |
| Phase complete or significant structural change | **guard-docs** |
| New dependencies added or security-sensitive code | **guard-security** |
| New dependencies added or before release | **guard-dependencies** |
| New mechanic needs adversarial scenario coverage | **writer-scenarios** |
| Phase complete or multiple sessions since last audit | **guard-agent-memory** |

## Phase 3 — Failure Routing (sequential, reactive)

React to output from Phase 2. Each failure type routes differently. See orchestration.md for circuit breaking and context pruning rules. See hint-formats.md for the exact block formats agents emit.

**IMPORTANT**: When routing failures to writer-code or writer-tests, pass the runner/reviewer agent's hint blocks verbatim — do NOT rewrite them. The hint formats are standardized so downstream agents can consume them directly. The main agent triages (decides which hints to act on and which to dismiss) but does not rephrase the hints themselves.

### runner-linting failures

| Failure type | Route |
|---|---|
| Clippy errors | Fix spec hint → **writer-code** (no writer-tests needed) |
| Format failures | runner-linting auto-formats — no further routing needed |

### runner-tests failures

| Failure type | Route |
|---|---|
| Existing test broke | Fix spec hint → **writer-code** (test exists, skip writer-tests) |
| Build failure (compiler error) | hint → **researcher-rust-errors** → **writer-code** |
| No test exists for broken behavior | hint → **writer-tests** (regression spec) → **writer-code** |

### runner-scenarios failures

| Confidence | Route |
|---|---|
| High | Regression spec hint → **writer-tests** → **writer-code** |
| Low | Main agent reads src first → writes spec → **writer-tests** → **writer-code** |

### reviewer-correctness bugs

| Confidence | Route |
|---|---|
| High | Regression spec hint → **writer-tests** → **writer-code** |
| Low | Main agent investigates → writes spec → **writer-tests** → **writer-code** |

### reviewer-quality and reviewer-bevy-api findings

| Finding type | Route |
|---|---|
| Style/idiom issue (reviewer-quality) | Main agent fixes inline — low risk, no test needed |
| Deprecated API (reviewer-bevy-api) | Main agent fixes inline — follow stated replacement pattern |
| Logic-adjacent issue (wrong query filter, etc.) | Treat as correctness issue — write regression spec if testable |

### guard-security findings

| Severity | Route |
|---|---|
| Critical (unsound unsafe, known CVE) | Main agent writes regression spec → **writer-tests** → **writer-code** |
| Warning (unwrap on file data, unused deps) | Main agent fixes inline |
| Info (observation for future phases) | Note and move on |

### guard-dependencies findings

| Category | Route |
|---|---|
| Unused dependency | Main agent removes from Cargo.toml |
| Outdated (security patch) | Main agent bumps version |
| Outdated (feature update) | Main agent evaluates and bumps if appropriate |
| License issue | Main agent evaluates — may need to replace the crate |
| Duplicate transitive | Main agent evaluates — may need to pin or unify |

### writer-scenarios output

| Result | Route |
|---|---|
| Scenarios created, all pass | Done — committed with the feature |
| Scenarios created, some fail | Investigate — may indicate a real bug (route to writer-tests → writer-code) |
| Compilation failure | Fix scenario code — may need **researcher-rust-errors** |
