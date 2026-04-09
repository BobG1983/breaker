---
name: todo
description: Manage the development todo list in docs/todos/. Add, complete, reorder, detail, research, or interrogate items. Use when the user wants to view, modify, or add to the todo list, or research and add details to existing items.
---

# Todo

Manage a prioritized development todo list. Captures conversation context so future sessions don't lose what was discussed.

## Rules

- **ALWAYS** read `TODO.md` before any operation — it's the source of truth for ordering and status
- **Never silently reorder** — only reorder when explicitly asked via `/todo reorder`
- **Bound questions** — when adding a new item, ask at most 2 clarifying questions. If you still lack detail, save what you have and mark `[NEEDS DETAIL]`
- **Preserve user ordering** — when adding, insert at a logical position based on dependencies and context, don't shuffle the whole list
- **Number all entries** — backlog items are numbered sequentially starting from 1. Renumber after any add, remove, done, or reorder operation

## When to Use

- User says "add that to todo", "remember this for later", "we should do X eventually"
- User wants to see what's queued up
- User wants to flesh out a vague todo with specifics
- User wants to reprioritize the backlog

## When NOT to Use

- User wants to do the work now — use `/implement` or `/quickfix`
- User is tracking in-progress work — that's session-state, not todos

## Usage

```
/todo <description>              # add a new item
/todo list                       # show current list
/todo detail <number or name>    # show detail for an item
/todo done <number or name>      # mark complete, clean up detail
/todo reorder                    # re-evaluate full list ordering, renumber
/todo interrogate                # loop through [NEEDS DETAIL] items, ask questions
/todo interrogate <number or name>  # interrogate a specific item
/todo research <number or name>  # launch research agents, save output with the todo
```

## File Structure

```
docs/todos/
  TODO.md              # ordered backlog — the active index
  DONE.md              # completed items — historical record, not read routinely
  detail/
    <slug>.md          # rich context per item
    <slug>/            # directory form for items with research or design docs
      <slug>.md        # the detail file (same format)
      research/        # research output from /todo research
      *.md             # additional design docs
```

`TODO.md` is the index. Detail files hold conversation context, design reasoning, scope notes, and anything the agent gathered during discussion. Items with research or extensive design docs use a directory instead of a single file.

## Procedure

### `/todo <description>` — Add

1. Read `TODO.md` (create if it doesn't exist)
2. If the description is vague, ask up to 2 clarifying questions (use AskUserQuestion)
3. Generate a slug from the description (e.g., `bolt-zero-velocity-clamp`)
4. Write `docs/todos/detail/<slug>.md` with:
   - **Summary**: one-line description
   - **Context**: relevant conversation context — what was discussed, why this matters, what was decided
   - **Scope**: what's in and out, if known
   - **Dependencies**: other todos or existing systems this relates to
   - **Status**: `[NEEDS DETAIL]` if information is missing, or `ready` if fully scoped
5. Add an entry to `TODO.md` at a logical position (based on dependencies and development order)
6. Renumber all backlog entries sequentially
7. Report the new item and its number

### `/todo list` — List

1. Read and display `TODO.md`
2. Show status markers: `[NEEDS DETAIL]`, `ready`, `done`

### `/todo detail <item>` — Show Detail

1. Find the item in `TODO.md`
2. Read and display its detail file

### `/todo done <item>` — Complete

1. Find the item in `TODO.md` (by number or name)
2. Remove the entry from `TODO.md`
3. Append `- ~~Short description~~ — one-line summary of what was delivered` to `DONE.md` (create if it doesn't exist)
4. **Promote relevant documentation** from the detail file/directory:
   a. Read the detail file (and any design docs in the directory)
   b. Identify content that belongs in `docs/architecture/` (technical decisions, system design, data structures, ordering, patterns) or `docs/design/` (game design, terminology, player-facing mechanics)
   c. For each piece of promotable content:
      - If a matching architecture/design doc already exists: update it with the new information
      - If no matching doc exists but the content is substantial: create a new doc in the appropriate location
      - If the content is trivial or already covered: skip
   d. Update any `index.md` files that reference the promoted docs
5. After promotion, delete the detail file (or detail directory). The knowledge now lives in the canonical docs, not in todos.
6. Renumber remaining backlog entries sequentially

**Why promote instead of just delete?** Detail files accumulate design decisions, research findings, and architectural context during planning. Deleting them loses that knowledge. Promoting to `docs/architecture/` and `docs/design/` keeps the project documentation evergreen — future sessions can find the decisions without re-deriving them.

### `/todo reorder` — Full Reorder

1. Read `TODO.md` and all detail files
2. Re-evaluate the full list ordering based on:
   - Dependencies (blockers first)
   - Development efficiency (related items grouped)
   - Risk (uncertain items that might change other work go earlier)
   - User's stated priorities (if any)
3. Rewrite `TODO.md` with the new ordering, renumbered sequentially from 1
4. Show the before/after ordering and explain the reasoning

### `/todo research <item>` — Research a Todo

1. Find the item in `TODO.md` (by number or name)
2. Read its detail file to understand the scope and open questions
3. If the detail is a single file, convert to a directory: `detail/<slug>/` with the detail file moved inside and a `research/` subdirectory created
4. Launch applicable research agents (from `sub-agents.md` Research Agents) in parallel (if no relevant agents exist, use a General agent)
   - Tell each agent to write output to `docs/todos/detail/<slug>/research/<topic>.md` if they write to `.claude/research/<topic>.md` instead, move it yourself
5. After research completes, update the detail file with key findings and links to research files
6. If research resolves open questions, update status from `[NEEDS DETAIL]` to `ready` or remove the entry from the `[NEEDS DETAIL]` list in the details file.

### `/todo interrogate` — Fill Missing Details

1. Read `TODO.md` and find all `[NEEDS DETAIL]` items (or the specific item if one was provided)
2. For each item:
   a. Read its detail file
   b. Identify what's missing (scope? dependencies? design decisions?)
   c. Ask focused questions (use AskUserQuestion)
   d. Update the detail file with answers
   e. If fully scoped, change status from `[NEEDS DETAIL]` to `ready`
   f. If the user says "skip" or "next", move to the next item
3. Continue until:
   - All `[NEEDS DETAIL]` items have been addressed, OR
   - The user says "stop" or "that's enough"
4. Report summary: N items updated, M still need detail

## TODO.md Format

```markdown
# Todo

## Backlog

1. **[ready]** Short description — [detail](detail/slug.md)
2. **[NEEDS DETAIL]** Short description — [detail](detail/slug.md)
3. **[ready]** Short description — [detail](detail/slug.md)

```

## Detail File Format

```markdown
# <Title>

## Summary
One-line description.

## Context
What was discussed, why this matters, what decisions were made.
Include concrete details from the conversation that would be lost otherwise.

## Scope
- In: [what's included]
- Out: [what's explicitly excluded]

## Dependencies
- Depends on: [other todos or existing systems]
- Blocks: [what can't start until this is done]

## Notes
Any additional context, open questions, or design considerations.

## Status
`ready` | `[NEEDS DETAIL]` — [what's missing]
```
