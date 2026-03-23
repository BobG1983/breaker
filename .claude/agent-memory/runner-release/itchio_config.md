---
name: itch.io configuration
description: itch.io username, game slug, channel names, and butler setup status
type: project
---

itch.io username: rantzgames
Game slug: breaker
Channels: mac, windows, linux

Butler push targets:
- rantzgames/breaker:mac       — macOS ARM64 (aarch64-apple-darwin)
- rantzgames/breaker:windows   — Windows x64 (x86_64-pc-windows-msvc)
- rantzgames/breaker:linux     — Linux x64 (x86_64-unknown-linux-gnu)

**Why:** Confirmed by user when setting up initial release infrastructure (2026-03-23).

**How to apply:** Use these values verbatim in butler push commands. The publish-itch job in release.yml already has them hardcoded. Do not ask the user for itch.io config again.
