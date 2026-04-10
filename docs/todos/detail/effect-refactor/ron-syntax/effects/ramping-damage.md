# Name
RampingDamage

# Parameters
`f32` -- damage per trigger

# Description
Adds a flat damage bonus that grows each time the effect fires. The first activation adds the base value, the second adds it again (cumulative), and so on. This creates an escalating damage pattern -- the longer a bolt stays in play hitting things, the harder it hits. The accumulated damage resets at the start of each node.
