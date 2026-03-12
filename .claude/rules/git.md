# Git Workflow

## Branching

- Create a feature branch before starting any work: `feature/*`, `fix/*`, `refactor/*`
- Branch off the current branch (usually `main`)
- After merging, delete the branch locally and from the remote
- Never leave stale branches

## Commits

- Use conventional commit format: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`
- Commit after tests pass, not before
- Group related changes into logical commits when multiple concerns are addressed in one session
- Use HEREDOC syntax for multi-line commit messages
- **NEVER** include `Co-Authored-By` lines or any AI attribution in commit messages

## Commit Message Style

```
feat: short imperative summary under 72 chars

Optional body explaining why, not what. The diff shows what changed.
```

- Lead with the verb: add, fix, update, remove, extract, rename, wire
- No trailing period on the subject line
- Body only when the "why" isn't obvious from the subject

## Merging

- Fast-forward merge to main when possible (linear history)
- Delete the feature branch immediately after merge (local and remote)
- Push to remote after merge

## History Hygiene

- Never rewrite shared history without explicit approval
- Keep main always clean and passing (fmt, clippy, tests)
- If a pre-commit hook fails, fix the issue and create a NEW commit (don't amend)
