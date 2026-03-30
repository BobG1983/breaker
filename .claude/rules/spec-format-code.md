# Implementation Spec Format

Template and quality rules for implementation specs (consumed by writer-code). Produced by planning-writer-specs-code. Specs are written to files under `.claude/specs/`.

See `.claude/rules/spec-workflow.md` for the revision loop specs must pass before reaching writers.

## Template

```markdown
## Implementation Spec: [Domain] — [Feature]

### Domain
src/[domain]/

### Failing Tests
- `src/[domain]/[file].rs` — [N] tests to satisfy

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

### Constraints
- Do NOT modify: [files explicitly off-limits]
- Do NOT add: [features explicitly out of scope]
```

## Rules for Good Implementation Specs

1. **Point to the failing tests.** The writer-code reads them first.
2. **Name what to implement.** System names, component names, resource names.
3. **Point to reference patterns.** The writer-code should match existing code.
4. **Specify schedule placement.** FixedUpdate vs Update vs OnEnter.
5. **Specify ordering.** After which system sets, before which.
6. **State what's off-limits.** Especially shared files and other domains.
