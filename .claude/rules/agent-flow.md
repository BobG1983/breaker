# Sub-Agent Development Flow

When to launch which agents, how to interpret their output, and how failures chain to fixes. For the writer-tests → writer-code delegation pair specifically, see @.claude/rules/delegated-implementation.md.

## Phase 1 — Before Writing Code (sequential, blocks implementation)

| Trigger | Agent |
|---------|-------|
| Unfamiliar Bevy 0.18 API or pattern | **researcher-bevy-api** |

## Phase 2 — Post-Implementation (single parallel wave)

Launch all applicable agents in a **single message** with multiple Agent tool calls. They are independent and must run in parallel — separate messages make them sequential.

### Always launch

- **runner-linting** — fmt (auto-formats in place) and clippy; Fix spec hints for clippy errors → writer-code
- **runner-tests** — cargo dtest; Fix spec hints for failures → writer-code or writer-tests
- **runner-scenarios** — gameplay invariant validation (release build via `cargo scenario`)
- **reviewer-correctness** — logic bugs, ECS pitfalls, state machine holes, math
- **reviewer-quality** — idioms, vocabulary, test coverage, documentation
- **reviewer-bevy-api** — Bevy API correctness for exact version

### Launch conditionally (add to the same single message)

| Condition | Agent |
|-----------|-------|
| New system, plugin, or module added | **guard-architecture** |
| 3+ systems added, or cross-plugin data flow | **researcher-system-dependencies** |
| New components or systems touching many entities | **guard-performance** |
| New gameplay mechanic or upgrade designed | **guard-game-design** |
| Phase complete or significant structural change | **guard-docs** |

## Phase 3 — Failure Routing (sequential, reactive)

React to output from Phase 2. Each failure type routes differently.

### runner-linting failures

Each clippy error includes a **Fix spec hint** block:

```
**Fix spec hint:**
- Lint: `path/to/file.rs:line` — `clippy::lint_name`
- Issue: [what the code does wrong]
- Fix: [specific change]
- Delegate: writer-code can apply directly
```

| Failure type | Route |
|---|---|
| Clippy errors | hint → **writer-code** (no writer-tests needed) |
| Format failures | runner-linting auto-formats — no further routing needed |

### runner-tests failures

Each failing test includes a **Fix spec hint** block:

```
**Fix spec hint:**
- Failing test: `path/to/file.rs::tests::test_name`
- Expected: [what the test requires]
- Got: [what actually happened]
- System under test: likely `path/to/system.rs`
- Delegate: writer-code can fix directly from this — no writer-tests needed (test already exists)
```

| Failure type | Route |
|---|---|
| Existing test broke | hint → **writer-code** (test exists, skip writer-tests) |
| Build failure (compiler error) | hint → **researcher-rust-errors** → **writer-code** |
| No test exists for broken behavior | hint → **writer-tests** (regression spec) → **writer-code** |

### runner-scenarios failures

Each failing scenario includes a **Regression spec hint** block:

```
**Regression spec hint:**
- Broken behavior: [what should happen that doesn't]
- Concrete values: [position, velocity, frame, entity — from violation message]
- Suspected location: `path/to/file.rs:line` (confidence: high/medium/low)
- Test type: unit | scenario
- Test file: `path/to/file.rs` or `scenarios/regressions/<name>.scenario.ron`
- Delegate: main agent can hand this directly to writer-tests if confidence is high
```

| Confidence | Route |
|---|---|
| High | hint → **writer-tests** (regression spec) → **writer-code** |
| Low | Main agent reads src first → writes spec → **writer-tests** → **writer-code** |

### reviewer-correctness bugs

Each confirmed bug includes a **Regression spec hint** block:

```
**Regression spec hint:**
- Broken behavior: [what the code does wrong vs. what it should do]
- Location: `path/to/file.rs:line` (confidence: high/medium/low)
- Correct behavior: Given [state], When [trigger], Then [expected outcome]
- Concrete values: [inputs/state that expose the bug]
- Test type: unit | integration
- Test file: `path/to/system_file.rs`
- Delegate: main agent can hand this directly to writer-tests if confidence is high
```

| Confidence | Route |
|---|---|
| High | hint → **writer-tests** (regression spec) → **writer-code** |
| Low | Main agent investigates further → writes spec → **writer-tests** → **writer-code** |

### reviewer-quality and reviewer-bevy-api findings

These agents describe fixes but never produce writer-tests specs.

| Finding type | Route |
|---|---|
| Style/idiom issue (reviewer-quality) | Main agent fixes inline — low risk, no test needed |
| Deprecated API (reviewer-bevy-api) | Main agent fixes inline — follow stated replacement pattern |
| Logic-adjacent issue (wrong query filter, etc.) | Treat as correctness issue — write regression spec if behavior is testable |
