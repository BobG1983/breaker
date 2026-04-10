# Name
TimePenalty

# Parameters
`TimePenaltyConfig` — See [TimePenaltyConfig](../configs/time-penalty-config.md)

# Description
Subtracts time from the node timer. TimePenalty(TimePenaltyConfig(seconds: 5.0)) removes 5 seconds from the remaining time. If the timer reaches zero, the node is failed.
