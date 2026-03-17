# Sub-Agent Development Flow

When to launch which agents, how to interpret their output, and how failures chain to fixes. For the test-writer → code-writer delegation pair specifically, see @.claude/rules/delegated-implementation.md.

## Phase 1 — Before Writing Code (sequential, blocks implementation)

| Trigger | Agent |
|---------|-------|
| Unfamiliar Bevy 0.18 API or pattern | **bevy-api-expert** |

## Phase 2 — Post-Implementation (single parallel wave)

Launch all applicable agents in a **single message** with multiple Agent tool calls. They are independent and must run in parallel — separate messages make them sequential.

### Always launch

- **test-runner** — fmt, clippy, tests
- **scenario-runner** — gameplay invariant validation
- **correctness-reviewer** — logic bugs, ECS pitfalls, state machine holes, math
- **quality-reviewer** — idioms, vocabulary, test coverage, documentation
- **bevy-api-reviewer** — Bevy API correctness for exact version

### Launch conditionally (add to the same single message)

| Condition | Agent |
|-----------|-------|
| New system, plugin, or module added | **architecture-guard** |
| 3+ systems added, or cross-plugin data flow | **system-dependency-mapper** |
| New components or systems touching many entities | **perf-guard** |
| New gameplay mechanic or upgrade designed | **game-design-guard** |
| Phase complete or significant structural change | **doc-guard** |

## Phase 3 — Failure Routing (sequential, reactive)

React to output from Phase 2. Each failure type routes differently.

### test-runner failures

Each failing test includes a **Fix spec hint** block:

```
**Fix spec hint:**
- Failing test: `path/to/file.rs::tests::test_name`
- Expected: [what the test requires]
- Got: [what actually happened]
- System under test: likely `path/to/system.rs`
- Delegate: code-writer can fix directly from this — no test-writer needed (test already exists)
```

| Failure type | Route |
|---|---|
| Existing test broke | hint → **code-writer** (test exists, skip test-writer) |
| Build failure (compiler error) | hint → **rust-error-decoder** → **code-writer** |
| No test exists for broken behavior | hint → **test-writer** (regression spec) → **code-writer** |

### scenario-runner failures

Each failing scenario includes a **Regression spec hint** block:

```
**Regression spec hint:**
- Broken behavior: [what should happen that doesn't]
- Concrete values: [position, velocity, frame, entity — from violation message]
- Suspected location: `path/to/file.rs:line` (confidence: high/medium/low)
- Test type: unit | scenario
- Test file: `path/to/file.rs` or `scenarios/regressions/<name>.scenario.ron`
- Delegate: main agent can hand this directly to test-writer if confidence is high
```

| Confidence | Route |
|---|---|
| High | hint → **test-writer** (regression spec) → **code-writer** |
| Low | Main agent reads src first → writes spec → **test-writer** → **code-writer** |

### correctness-reviewer bugs

Each confirmed bug includes a **Regression spec hint** block:

```
**Regression spec hint:**
- Broken behavior: [what the code does wrong vs. what it should do]
- Location: `path/to/file.rs:line` (confidence: high/medium/low)
- Correct behavior: Given [state], When [trigger], Then [expected outcome]
- Concrete values: [inputs/state that expose the bug]
- Test type: unit | integration
- Test file: `path/to/system_file.rs`
- Delegate: main agent can hand this directly to test-writer if confidence is high
```

| Confidence | Route |
|---|---|
| High | hint → **test-writer** (regression spec) → **code-writer** |
| Low | Main agent investigates further → writes spec → **test-writer** → **code-writer** |

### quality-reviewer and bevy-api-reviewer findings

These agents describe fixes but never produce test-writer specs.

| Finding type | Route |
|---|---|
| Style/idiom issue (quality-reviewer) | Main agent fixes inline — low risk, no test needed |
| Deprecated API (bevy-api-reviewer) | Main agent fixes inline — follow stated replacement pattern |
| Logic-adjacent issue (wrong query filter, etc.) | Treat as correctness issue — write regression spec if behavior is testable |
