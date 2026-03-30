# Hint Formats

Standardized output formats that verification agents produce and failure routing consumes. Each agent emits a specific block; the main agent passes these verbatim when launching fix agents.

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

reviewer-file-length produces a split spec at `.claude/specs/file-splits.md` — no hint format needed. The orchestrator launches parallel writer-code agents to execute the splits.

## Dependency finding (guard-dependencies)

```
**Dependency finding:**
- Category: unused | outdated | duplicate | license | feature-flag
- Crate: [crate name and version]
- Issue: [what's wrong]
- Fix: [specific Cargo.toml change]
- Delegate: main agent applies Cargo.toml changes directly
```
