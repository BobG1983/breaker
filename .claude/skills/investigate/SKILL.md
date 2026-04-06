---
name: investigate
description: Methodical problem-solving workflow for debugging issues. Use when attempting to resolve bugs, errors, failing tests, failing scenarios, or unexpected behavior.
---

# Systematic Debugging

A structured approach to debugging that prevents premature solutions and ensures root cause identification.

## Purpose

Replace ad-hoc debugging with a systematic process that:
1. Gathers evidence before making changes
2. Identifies root cause, not just symptoms
3. Prevents introducing new bugs while fixing
4. Documents findings for future reference

## When to Use

- Bug reports with unclear cause
- Errors that don't make sense
- Issues that have "already been fixed" before
- Problems spanning multiple components
- Performance issues
- Failing Unit Tests
- Failing Scenarios
- Escalation from `/verify`, `/implement`, or `/quickfix` after circuit breaking (3 failed attempts)

## When NOT to Use

- The fix is obvious and you know exactly what's wrong — use `/quickfix` instead
- You haven't tried fixing it yet — try the normal routing in `/verify` first, then escalate to `/investigate` if it keeps failing
- You're planning new work, not debugging existing behavior — use `/plan` then `/implement`

## The DEBUG Protocol

### D - Define the Problem

Before touching code, clearly define:

```markdown
## Problem Definition

**Observed Behavior**: [What is actually happening]
**Expected Behavior**: [What should happen]
**Reproduction Steps**:
1. [Step 1]
2. [Step 2]
3. [Result]

**Environment**: [Tests, Game, Crate, Scenario Runner]
**Frequency**: [Always / Sometimes / Rare]
**Recent Changes**: [What changed recently that might relate]
```

### E - Explore the Evidence

Gather information systematically:

**FIRST ALWAYS:** Check the full console output, or related log files

**THEN:** Use Research Agents or Explore (where no dedicated researcher exists) to trace execution paths, resources used, etc.

1. **Console/Logs**: Check the console, or log files
2. **Network**: Inspect API requests/responses
3. **Database**: Query relevant data state
4. **Code Path**: Trace the execution path
5. **User Context**: Check user role, permissions, session

```
@reasearcher-codebase: What is the data flow for this function?
@researcher-git: What is the edit history for this file?
```

### B - Build Hypotheses

Generate multiple possible causes:

```markdown
## Hypotheses (ranked by likelihood)

1. **[Most likely]**: [Description]
   - Evidence for: [...]
   - Evidence against: [...]
   - Test: [What test could we write that would fail if this hypothesis was true?]

2. **[Second likely]**: [Description]
   - Evidence for: [...]
   - Evidence against: [...]
   - Test: [What test could we write that would fail if this hypothesis was true?]

3. **[Less likely]**: [Description]
   - Evidence for: [...]
   - Evidence against: [...]
   - Test: [What test could we write that would fail if this hypothesis was true?]
```

### U - Uncover Root Cause

Test hypotheses systematically:

1. Start with most likely hypothesis
2. Write a test that represents the desired behavior
3. Execute that test only, record results
4. If the test passes, move to next hypothesis - a passing test means things are behaving correctly
5. Continue until root cause confirmed or you run out of hypotheses
6. If you run out of hypotheses start the whole skill over again

**The Five Whys**:
- Why did this happen? → [Answer 1]
- Why did [Answer 1] happen? → [Answer 2]
- Why did [Answer 2] happen? → [Answer 3]
- Why did [Answer 3] happen? → [Answer 4]
- Why did [Answer 4] happen? → [Probably Root Cause - **This is a new hypothesis**]

### G - Generate Fix

Only after root cause is confirmed:

```markdown
## Fix Plan

**Root Cause**: [Confirmed cause]
**Fix Approach**: [How to fix]
**Files to Change**:
- [file1]: [change]
- [file2]: [change]

**Risk Assessment**:
- Blast radius: [What else might be affected]
- Rollback plan: [How to undo if needed]

**Verification**:
- [ ] Original issue resolved
- [ ] No new issues introduced
- [ ] Related functionality still works
- [ ] New tests added to confirm regression never happens
```

**ALWAYS**:
- Use research agents to confirm blast radius.
- Use research agents to confirm API use is correct.
- Use the full TDD cycle to implement the fix

## Anti-Patterns to Avoid

| Anti-Pattern | Why Bad | Instead |
|--------------|---------|---------|
| **Shotgun debugging** | Random changes, new bugs | Systematic hypothesis testing |
| **Assuming the cause** | Fix wrong thing | Gather evidence first |
| **Only fixing symptom** | Bug returns | Find root cause |
| **No verification** | Incomplete fix | Test thoroughly |
| **No documentation** | Future confusion | Document findings |

## Quick Reference

```
1. DEFINE: What's happening vs what should happen?
2. EXPLORE: Console, network, database, code path
3. BUILD: 2-3 hypotheses ranked by likelihood
4. UNCOVER: Test hypotheses, use Five Whys
5. GENERATE: Fix only after root cause confirmed
```

## Output Format

**ALWAYS:**
- Write this output to .claude/fixes/[timestamp]-[short_name].md
- Update the file as information changes, hypotheses are tested, tests written, fixes generated, etc.


```markdown
## Debug Report: [Issue Title]

### Problem
- **Observed**: [symptom]
- **Expected**: [correct behavior]
- **Repro**: [steps]

### Investigation
- **Console**: [findings]
- **Network**: [findings]
- **Code Path**: [findings]

### Root Cause
[Confirmed root cause with evidence]

### Fix
- **Approach**: [how fixed]
- **Files Changed**: [list]
- **Verification**: [how verified]

### Prevention
[How to prevent this class of bug in future]
```
