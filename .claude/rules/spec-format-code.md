# Implementation Spec Format

Template and quality rules for implementation specs (consumed by writer-code). Produced by planning-writer-specs-code **after the RED gate passes** — the failing tests on disk are the authoritative contract; the impl spec describes how to satisfy them.

See `.claude/rules/spec-workflow.md` for the revision loop specs must pass before reaching writers, and for the briefing the orchestrator provides (test spec path, failing test paths, RED gate confirmation).

## Template

```markdown
## Implementation Spec: [Domain] — [Feature]

### Domain
src/[domain]/

### Failing Tests
List EVERY failing test by file path AND test function name. Count them — if there are 11, list 11.
- `src/[domain]/[file].rs::[test_fn_name]` — [one-line description of what it checks]
- `src/[domain]/[file].rs::[test_fn_name]` — ...

### Test Spec Reference
- Behavioral test spec: `.claude/specs/[wave]-[feature]-tests.md`

### What to Implement
- [System/component/resource name]: [one-line description]
- [System/component/resource name]: [one-line description]

### Patterns to Follow
- Follow the pattern in `src/[domain]/systems/[existing].rs`
- [Specific convention or constraint]

### RON Data (if applicable)
- Add fields to `assets/[file].ron`: [field names and types]
- Add to defaults: [defaults resource path]

### Schedule
- [System] runs in [FixedUpdate/Update/OnEnter(State)]
- [Ordering constraints: after X, before Y]

### Test Harness Updates
List every test-harness change needed to keep existing tests green AND make new tests reachable
(e.g., `app.init_resource::<NewlyRequiredRes>()` in existing harness builders, system-param
extensions that affect every existing test in the file, etc.).

### Constraints
- Do NOT modify: [files explicitly off-limits]
- Do NOT add: [features explicitly out of scope]
```

## Rules for Good Implementation Specs

1. **List EVERY failing test by name.** The writer-code reads them first; the spec must enumerate the contract, not summarize. A count of "9 tests" when 11 exist is a defect.
2. **Reference the failing tests, not just the test spec.** The tests are on disk; they are the authoritative contract. Cite their file paths.
3. **Cross-check stub declarations against the test spec.** If the test spec defined a component with `#[derive(Component, Debug, Clone, Copy)]` and `pub(super)`, the impl spec must use the same derives and visibility. Conflicts here cause writer-code to fight writer-tests.
4. **Name what to implement.** System names, component names, resource names.
5. **Cover existing-test breakage.** If a system param change adds a new required `Res<T>`, every existing test that exercises that system needs `T` inserted in its harness. Spec it explicitly under "Test Harness Updates" — don't leave writer-code to discover it via panic.
6. **Point to reference patterns.** The writer-code should match existing code.
7. **Specify schedule placement.** FixedUpdate vs Update vs OnEnter.
8. **Specify ordering.** After which system sets, before which.
9. **State what's off-limits.** Especially shared files and other domains.
