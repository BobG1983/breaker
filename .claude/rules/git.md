# Git Workflow (git-flow-next)

## Setup

For fresh clones, initialize git-flow-next and configure custom topic types:

```bash
git flow init --preset=classic --defaults
git flow config add topic refactor develop --prefix=refactor/
git flow config edit topic bugfix --prefix=fix/
```

Also set the local merge strategy to always create merge commits (preserves full branch topology in git UIs):

```bash
git config --local merge.ff false
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

- Commit only when the full TDD cycle is clean — see `.claude/rules/tdd.md`
- See `.claude/rules/commit-format.md` for message format and style
- Group related changes into logical commits when multiple concerns are addressed in one session

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

## Staying Current

While on a feature/fix/refactor branch, pull changes from `develop` before finishing:

```bash
git flow update       # merges develop into current branch (preserves branch history)
```

Run this before `git flow <type> finish` to prevent merge conflicts at finish time.
If `finish` fails and leaves a stale state, clear it with:

```bash
rm .git/gitflow/state/merge.json
```

## Pre-Merge Guard Gate

Before `git flow <type> finish` (merging to `develop`), run the **Full Verification Tier** — see `.claude/rules/verification-tiers.md` for the complete agent list.

The Full Verification Tier includes the Standard Verification Tier (which must already be passing from pre-commit) plus all guards, scenarios, and structural reviewers. All agents must pass before finishing the branch. Fix any findings before merging.

## History Hygiene

- `merge.ff = false` is set locally — all merges create merge commits, preserving full branch topology in git UIs
- Never rewrite shared history without explicit approval
- Keep `main` and `develop` always clean and passing (fmt, clippy, tests)
- If a pre-commit hook fails, fix the issue and create a NEW commit (don't amend)
- Verify clean working tree (`git status`) before `git flow <type> finish` — background agents may leave dirty files
- Never merge to `main` manually — only the release agent updates `main` via `git flow release finish`
