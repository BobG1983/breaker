# Name
RampingDamage

# Parameters
`RampingDamageConfig` — See [RampingDamageConfig](../configs/ramping-damage-config.md)

# Description
Adds a flat damage bonus that grows each time the effect fires. The first activation adds the base increment, the second adds it again (cumulative), and so on. This creates an escalating damage pattern -- the longer a bolt stays in play hitting things, the harder it hits. The accumulated damage resets at the start of each node.
