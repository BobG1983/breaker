# Hint Formats

Standardized output formats that verification agents produce and failure routing consumes. See `.claude/rules/routing-failures.md` Hint Passthrough Rule for how the main agent handles these.

## Fix spec hint (runner-linting)

```
**Fix spec hint:**
- Lint: `path/to/file.rs:line` — `clippy::lint_name`
- Issue: [what the code does wrong]
- Fix: [specific change]
- Delegate: writer-code can apply directly
```

## Fix spec hint (runner-tests)

```
**Fix spec hint:**
- Failing test: `path/to/file.rs::tests::test_name`
- Expected: [what the test requires]
- Got: [what actually happened]
- System under test: likely `path/to/system.rs`
- Delegate: writer-code can fix directly from this — no writer-tests needed (test already exists)
```

## Regression spec hint (runner-scenarios)

```
**Regression spec hint:**
- Broken behavior: [what should happen that doesn't]
- Concrete values: [position, velocity, frame, entity — from violation message]
- Suspected location: `path/to/file.rs:line` (confidence: high/medium/low)
- Test type: unit | scenario
- Test file: `path/to/file.rs` or `scenarios/regressions/<name>.scenario.ron`
- Delegate: main agent can hand this directly to writer-tests if confidence is high
```

## Regression spec hint (reviewer-correctness)

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

## Test revision hint (reviewer-tests)

```
**Test revision hint:**
- Test file: `path/to/test_file.rs`
- Spec behavior: [which numbered behavior from the spec]
- Finding: [what's wrong — missing coverage, wrong values, production logic in stub]
- Severity: BLOCKING | IMPORTANT | MINOR
- Fix: [specific change needed]
- Delegate: main agent routes back to writer-tests with test revision spec
```

## Completeness finding (reviewer-completeness)

```
**Completeness finding:**
- Category: MISSING | PARTIAL | SCOPE_NARROWED | DIVERGED
- Item: [what was promised]
- Source: [todo detail | plan wave item N]
- Expected: [what should exist — file, system, type, test]
- Found: [nothing | stub at file:line | different approach at file:line]
- Delegate: orchestrator must implement (MISSING/PARTIAL/SCOPE_NARROWED) or document decision revision (DIVERGED)
```

## Security finding (guard-security)

```
**Security finding:**
- Severity: critical | warning | info
- Location: `path/to/file.rs:line`
- Issue: [what the security concern is]
- Fix: [specific remediation]
- Delegate: main agent fixes inline (warning/info) or writer-code (critical with test coverage)
```

## reviewer-file-length

reviewer-file-length returns its findings inline to the orchestrator: a summary table plus, per HIGH/MEDIUM file, a refactor spec hint of the form below. It does NOT write to `docs/todos/`. The orchestrator launches background sub-agents (forks — `Agent` without `subagent_type`) to perform the splits in the current branch (per `.claude/rules/file-splitting.md`) before merge — never via `/implement`, `/quickfix`, or a todo. Multiple splits run in parallel; Basic Verification Tier runs after all complete.

```
**Refactor spec hint:**
- Source file: `path/to/original_file.rs`
- Total lines: N (prod: N, tests: N)
- Strategy: A | B | C
- Target structure:
  ```
  path/to/
    new_dir/
      mod.rs      // [exact contents]
      system.rs   // [what goes here]
      tests.rs    // [what goes here, or tests/ breakdown]
  ```
- Test groups (for sub-splitting):
  - `group_name.rs`: test_fn_1, test_fn_2, ... (N tests, ~M lines)
- Imports needed: [use statements the split files will need]
- Re-exports needed: [what mod.rs must re-export to maintain public API]
- Delegate: orchestrator executes the split inline in the current branch
```

## Dependency finding (guard-dependencies)

```
**Dependency finding:**
- Category: unused | outdated | duplicate | license | feature-flag
- Crate: [crate name and version]
- Issue: [what's wrong]
- Fix: [specific Cargo.toml change]
- Delegate: main agent applies Cargo.toml changes directly
```
