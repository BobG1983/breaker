# Name
EntropyConfig

# Parameters
- max_effects: u32
- pool: Vec<(f32, Effect)>

# Description
- max_effects: Cap on how many effects fire per activation (counter can't exceed this)
- pool: Weighted list of effects to randomly select from -- each entry is (weight, effect)
