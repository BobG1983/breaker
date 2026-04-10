# Name
RandomEffect

# Parameters
`[(RandomWeight, Effect), ...]` -- weighted pool

# Description
Picks exactly one effect from a weighted pool and fires it. Each entry in the pool has a weight that determines its probability of being selected. Higher weights are more likely. Only one effect fires per activation -- unlike EntropyEngine which can fire multiple.
