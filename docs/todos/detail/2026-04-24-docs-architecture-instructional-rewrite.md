# docs/architecture — rewrite as instructional, not audit

## Problem

`docs/architecture/ordering.md` is 380 lines. `docs/architecture/plugins.md` has a table of every named set, every cross-domain write exception, and every per-system ordering edge. `docs/architecture/messages.md` enumerates every message with producer/consumer rows. All three read like audits of the current codebase rather than guidance for developers.

Audit-style docs go stale fast and are painful to maintain — every refactor requires line-item updates, and readers can't extract the *principles* without drowning in specifics.

## Goal

Docs/architecture should answer these questions, concisely:
- How do I decide what goes in a plugin vs. lives outside?
- How do I decide when to add a `SystemSet` variant vs. a bare `.after()` edge?
- How do I decide between a component, resource, and message?
- What are the cross-domain patterns (messages, shared pipelines, exception flags) and when is each appropriate?
- What constraints does the `rantzsoft_dmg` pipeline impose on damage-emitting systems?

Anything that isn't guidance goes to source doc comments or gets deleted.

## Scope

Three files:
1. `docs/architecture/ordering.md` — collapse the "SystemSet Convention" section into 2–3 examples; delete the exhaustive set table; keep the "loose with key constraints" principle block; maybe keep ONE annotated timeline example showing FixedUpdate flow end-to-end.
2. `docs/architecture/plugins.md` — remove the cross-domain exception sections (they document what IS, not what SHOULD BE); replace with a "when to make an exception" rubric. The per-plugin descriptions can stay if they're describing responsibilities (not systems).
3. `docs/architecture/messages.md` — keep the message-vs-event-vs-component decision guide; drop the per-message producer/consumer table (grep reveals that better than any doc).

## Non-goals

- Don't rewrite design docs (`docs/design/`).
- Don't touch `CLAUDE.md` or agent rules.
- Don't change code — docs only.

## Verification

After the rewrite:
- Any senior engineer can land a new plugin correctly by reading ONLY these three files.
- File lengths drop substantially (target: each under 150 lines).
- Specific facts that were in the audit sections live in source code doc comments instead.

## Not Blocking

No code work depends on this. Can land at any time.
