# Name
EntropyEngine

# Parameters
`EntropyConfig`

# Description
Escalating chaos. Each time the effect fires, an internal counter increments. The effect then fires a number of random effects from a weighted pool equal to the counter (up to a configured maximum). So the first activation fires 1 random effect, the second fires 2, and so on until the cap. The counter resets at the start of each node. This creates an accelerating cascade -- the longer the node goes, the more chaotic the effects become. See [EntropyConfig](../configs/entropy-config.md).
