---
name: CI/CD infrastructure status
description: What release infrastructure has been created and what secrets are still needed
type: project
---

## GitHub Actions workflow

File: `.github/workflows/release.yml`
Created: 2026-03-23
Status: Created (not yet triggered — no tags exist)

Triggers:
- `push` on `v*` tags — full pipeline (build + GitHub Release + itch.io publish)
- `workflow_dispatch` with `tag` input — build + GitHub Release only (no itch.io)

Build targets:
- macOS ARM64: `aarch64-apple-darwin` on `macos-latest`
- Windows x64: `x86_64-pc-windows-msvc` on `windows-latest`
- Linux x64: `x86_64-unknown-linux-gnu` on `ubuntu-latest`

Build command: `cargo build --release -p breaker --no-default-features --target <target>`
No `bevy/dynamic_linking`. No `dev` feature. Matches `[profile.release]` in workspace Cargo.toml.

Rust toolchain: nightly (via `dtolnay/rust-toolchain@nightly`) — matches CI.
Cache: `Swatinem/rust-cache@v2` with `release-<runner>` shared keys (separate from CI keys).

## Secrets required

- `BUTLER_API_KEY` — NOT YET CONFIRMED added to repo secrets
  Get from: https://itch.io/user/settings/api-keys
  Add at: GitHub repo Settings → Secrets → Actions

## Other infrastructure

- `CHANGELOG.md`: created at repo root (header only, no release section yet)
- `main.rs`: `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` added

**Why:** First-time release infrastructure setup (2026-03-23). No version bump or release branch — infrastructure only.

**How to apply:** On next release, check BUTLER_API_KEY secret is in place before pushing a tag.
