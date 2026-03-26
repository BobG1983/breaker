# Phase 4j: Release Infrastructure

**Goal**: Automated cross-platform build pipeline so the vertical slice can ship off `main` as playable releases.

**Wave**: 4 (capstone) — parallel with 4h and 4i. **Session 8.**

## Dependencies

- None (infrastructure, not gameplay). Can technically run at any point, but placed in Wave 4 so there's something worth releasing.

## What to Build

### GitHub Actions Cross-Compilation Workflow

- Build release binaries for macOS (Apple Silicon), Windows (x86_64), and Linux (x86_64)
- Trigger on `git flow release finish --tag` (i.e., tags pushed to `main`)
- Produce downloadable artifacts per platform
- Use `cargo build --release` (no dynamic linking in release builds)

### itch.io Distribution via Butler

- Set up [butler](https://itch.io/docs/butler/) push targets for each platform
- Automate upload as a step in the release workflow (or manual trigger)
- Channel naming: `windows`, `mac`, `linux`

### Version Bumping & Changelog

- `Cargo.toml` version bump (workspace-level)
- Changelog generation from conventional commits since last tag
- The **runner-release** agent handles this — see CLAUDE.md agent workflow

### Release Process

The `git flow release` workflow from `.claude/rules/git.md` drives this:

```bash
git flow release start <version>     # Branch from develop
# bump version in Cargo.toml, generate changelog, commit
git flow release finish --tag        # Merge to main, tag, merge back to develop
git push --all --tags                # Triggers CI release workflow
```

## Acceptance Criteria

1. Pushing a tag to `main` triggers a GitHub Actions workflow that builds release binaries for macOS/Windows/Linux
2. Release artifacts are downloadable from GitHub Releases
3. Butler push to itch.io works (manual or automated)
4. Version in `Cargo.toml` matches the release tag
5. Changelog covers commits since previous tag
