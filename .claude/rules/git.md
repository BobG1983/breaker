# Git Workflow (git-flow-next)

## Setup

For fresh clones, initialize git-flow-next and configure custom topic types:

```bash
git flow init --preset=classic --defaults
git flow config add topic refactor develop --prefix=refactor/
git flow config edit topic bugfix --prefix=fix/
```

This establishes:
- **Base branches**: `main` (production), `develop` (integration)
- **Topic types**: `feature/*`, `fix/*`, `refactor/*`, `release/*`, `hotfix/*`

## Branching

All work branches are created via `git flow <type> start`:

| Work type | Command | Branches from |
|-----------|---------|---------------|
| New feature | `git flow feature start <name>` | `develop` |
| Bug fix | `git flow bugfix start <name>` | `develop` |
| Refactor | `git flow refactor start <name>` | `develop` |
| Release | `git flow release start <version>` | `develop` |
| Hotfix | `git flow hotfix start <name>` | `main` |

- **Never** use raw `git checkout -b`, `git branch`, or `git merge` for workflow branching
- Start a topic branch before doing any work

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

## Finishing

`git flow <type> finish` replaces manual merge + delete:

| Work type | Command | Effect |
|-----------|---------|--------|
| Feature | `git flow feature finish` | Merges to `develop`, deletes branch |
| Bug fix | `git flow bugfix finish` | Merges to `develop`, deletes branch |
| Refactor | `git flow refactor finish` | Merges to `develop`, deletes branch |
| Release | `git flow release finish --tag` | Merges to `main`, tags, merges back to `develop`, deletes branch |
| Hotfix | `git flow hotfix finish --tag` | Merges to `main`, tags, merges back to `develop`, deletes branch |

## Publishing

Push topic branches to remote for collaboration or CI:

```bash
git flow <type> publish
```

## Releases

```bash
git flow release start <version>     # Create release branch from develop
# ... bump version, update changelog, commit ...
git flow release finish --tag        # Merge to main, tag, merge to develop, delete branch
git push --all --tags                # Push everything
```

## Hotfixes

```bash
git flow hotfix start <name>         # Branch from main
# ... fix, commit ...
git flow hotfix finish --tag         # Merge to main, tag, merge to develop, delete branch
git push --all --tags                # Push everything
```

## History Hygiene

- Never rewrite shared history without explicit approval
- Keep `main` and `develop` always clean and passing (fmt, clippy, tests)
- If a pre-commit hook fails, fix the issue and create a NEW commit (don't amend)
