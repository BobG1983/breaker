# Spec Workflow

Read this before creating or reviewing specs. This is the process that produces clean specs before the RED phase begins.

See `.claude/rules/spec-formats.md` for the spec templates and quality rules.
See `.claude/rules/tdd.md` for the TDD cycle that specs feed into.

## Phase 1 — Research and Spec Creation

Before writing any code, resolve unknowns and produce specs:

| Trigger | Agent |
|---------|-------|
| Feature touches 2+ domains (query conflicts, message flow, ordering) | **researcher-system-dependencies** |
| Unfamiliar Bevy 0.18 API or pattern | **researcher-bevy-api** |
| Modifying existing types or systems (ripple effects) | **researcher-impact** |
| Need to understand current behavior before modifying it | **researcher-codebase** |
| Choosing between Rust idiom alternatives | **researcher-rust-idioms** |
| Feature ready for spec writing | **planner-spec** |
| Specs produced — novel mechanic, cross-domain, or uncertain scope | **planner-review** |

## Spec Revision Loop

After planner-review produces findings, the main agent:

1. Triages findings (dismiss false positives, note valid issues)
2. Sends valid feedback back to **planner-spec** to produce corrected specs
3. Re-launches **planner-review** on the corrected specs if needed (skip if only MINOR findings remain)
4. Only proceeds to writer-tests once specs are confirmed clean

planner-review produces BLOCKING/IMPORTANT/MINOR findings. Do NOT launch writer-tests until the spec revision loop is complete. Do NOT skip this step even for "obvious" specs.

**Never launch writer-tests with unreviewed or uncorrected specs.** The cost of a bad spec propagating through writer-tests → writer-code is high (rework). The cost of one revision loop is low.
