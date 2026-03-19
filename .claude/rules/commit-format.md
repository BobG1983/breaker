# Commit Format

## Conventional Commit Types

`feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:` — with optional scope.

## Message Style

```
feat: short imperative summary under 72 chars

Optional body explaining why, not what. The diff shows what changed.
```

- Lead with the verb: add, fix, update, remove, extract, rename, wire
- No trailing period on the subject line
- Body only when the "why" isn't obvious from the subject
- Use HEREDOC syntax for multi-line commit messages
- **NEVER** include `Co-Authored-By` lines or any AI attribution in commit messages
