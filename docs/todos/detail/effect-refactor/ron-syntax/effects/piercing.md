# Name
Piercing

# Parameters
`PiercingConfig` — See [PiercingConfig](../configs/piercing-config.md)

# Description
Allows the bolt to pass through cells without bouncing off them. Each cell the bolt passes through decrements the piercing counter by one. When the counter reaches zero, the bolt bounces normally on the next hit. Multiple piercing effects stack additively.
