# Systems to Modify

Existing systems that currently use direct despawn or domain-specific death messages and need to send `KillYourself<T>` instead.

## Bolt domain
- [tick-bolt-lifespan.md](tick-bolt-lifespan.md) — Bolt lifespan expiry: `RequestBoltDestroyed` → `KillYourself<Bolt>`
- [bolt-lost.md](bolt-lost.md) — Bolt lost (off-screen/below breaker): `RequestBoltDestroyed` → `KillYourself<Bolt>`

## Wall domain (effect systems)
- [tick-shield-duration.md](tick-shield-duration.md) — Shield wall timer expiry: `commands.despawn()` → `KillYourself<Wall>`
- [despawn-second-wind-on-contact.md](despawn-second-wind-on-contact.md) — Second-wind wall after bounce: `commands.despawn()` → `KillYourself<Wall>`
