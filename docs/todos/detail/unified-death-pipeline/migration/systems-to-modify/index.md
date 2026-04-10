# Systems to Modify

Existing systems that currently send domain-specific death messages (`RequestBoltDestroyed`, etc.) and need to send `KillYourself<T>` instead.

- [tick-bolt-lifespan.md](tick-bolt-lifespan.md) — Bolt lifespan expiry: `RequestBoltDestroyed` → `KillYourself<Bolt>`
- [bolt-lost.md](bolt-lost.md) — Bolt lost (off-screen/below breaker): `RequestBoltDestroyed` → `KillYourself<Bolt>`
