# Spec Workflow

Read this before creating or reviewing specs. This is the process that produces clean specs before the RED phase begins.

See `.claude/rules/spec-formats.md` for the spec templates and quality rules.
See `.claude/rules/tdd.md` for the TDD cycle that specs feed into.

## Phase 1 — Research and Spec Creation

Before writing any code, resolve unknowns and produce specs. See `.claude/rules/sub-agents.md` for the full agent directory — research agents for pre-planning, pipeline agents for spec production.

1. Launch applicable **research agents** in parallel (see Research Agents in `sub-agents.md`)
2. Feed research results into **planner-spec** to produce test + implementation specs - write the specs into ephemeral memory with a meaningful name
3. Launch **planner-review** to pressure-test specs (especially for novel mechanics, cross-domain, or uncertain scope) - update the specs in ephemeral memory

## Spec Revision Loop

After planner-review produces findings, the main agent:

1. Triages findings (dismiss false positives, note valid issues)
2. Sends valid feedback back to **planner-spec** to produce corrected specs
3. Re-launches **planner-review** on the corrected specs if needed (skip if only MINOR findings remain)
4. Only proceeds to writer-tests once specs are confirmed clean

planner-review produces BLOCKING/IMPORTANT/MINOR findings. Do NOT launch writer-tests until the spec revision loop is complete. Do NOT skip this step even for "obvious" specs.

**Never launch writer-tests with unreviewed or uncorrected specs.** The cost of a bad spec propagating through writer-tests → writer-code is high (rework). The cost of one revision loop is low.
