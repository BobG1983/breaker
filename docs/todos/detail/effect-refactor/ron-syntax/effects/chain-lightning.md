# Name
ChainLightning

# Parameters
`ChainLightningConfig`

# Description
Launches a chain of lightning arcs that jump between cells. The first arc strikes a random cell within range of the source entity instantly. Each subsequent arc travels visually from the last-hit cell to a new random cell within range, dealing damage on arrival. Each cell can only be hit once per chain. If no valid target is in range, the chain ends early. The number of jumps, range, damage, and arc travel speed are all configurable. See [ChainLightningConfig](../configs/chain-lightning-config.md).
