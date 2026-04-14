//! Track armed fire command — records a fired participant on the owner's
//! `ArmedFiredParticipants` component so the Shape D disarm path can
//! reverse effects on the exact participants they were fired on.

use bevy::prelude::*;

use crate::effect_v3::storage::ArmedFiredParticipants;

/// Deferred command that appends a fired participant to the owner's
/// `ArmedFiredParticipants` component under the given armed source key.
///
/// Creates the component on first use. Does nothing if the owner entity
/// has been despawned between fire-time and command flush.
///
/// Visibility is restricted to `effect_v3` — callers outside the domain
/// must go through `EffectCommandsExt::track_armed_fire`.
pub(in crate::effect_v3) struct TrackArmedFireCommand {
    pub owner:        Entity,
    pub armed_source: String,
    pub participant:  Entity,
}

impl Command for TrackArmedFireCommand {
    fn apply(self, world: &mut World) {
        if world.get_entity(self.owner).is_err() {
            return;
        }
        if let Some(mut tracked) = world.get_mut::<ArmedFiredParticipants>(self.owner) {
            tracked.track(self.armed_source, self.participant);
        } else {
            let mut tracked = ArmedFiredParticipants::default();
            tracked.track(self.armed_source, self.participant);
            world.entity_mut(self.owner).insert(tracked);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_armed_fire_command_inserts_component_on_first_use() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let bolt = world.spawn_empty().id();

        TrackArmedFireCommand {
            owner,
            armed_source: "chip_redirect#armed[0]".to_owned(),
            participant: bolt,
        }
        .apply(&mut world);

        let tracked = world
            .get::<ArmedFiredParticipants>(owner)
            .expect("ArmedFiredParticipants should be inserted on first use");
        let vec = tracked
            .0
            .get("chip_redirect#armed[0]")
            .expect("key should be present");
        assert_eq!(vec.len(), 1);
        assert_eq!(vec[0], bolt);
    }

    #[test]
    fn track_armed_fire_command_appends_to_existing_component() {
        let mut world = World::new();
        let owner = world.spawn(ArmedFiredParticipants::default()).id();
        let bolt_a = world.spawn_empty().id();
        let bolt_b = world.spawn_empty().id();

        TrackArmedFireCommand {
            owner,
            armed_source: "chip_redirect#armed[0]".to_owned(),
            participant: bolt_a,
        }
        .apply(&mut world);

        TrackArmedFireCommand {
            owner,
            armed_source: "chip_redirect#armed[0]".to_owned(),
            participant: bolt_b,
        }
        .apply(&mut world);

        let tracked = world.get::<ArmedFiredParticipants>(owner).unwrap();
        let vec = tracked.0.get("chip_redirect#armed[0]").unwrap();
        assert_eq!(vec.len(), 2);
        assert_eq!(vec[0], bolt_a);
        assert_eq!(vec[1], bolt_b);
    }

    #[test]
    fn track_armed_fire_command_does_not_panic_when_owner_despawned() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let bolt = world.spawn_empty().id();
        world.despawn(owner);

        TrackArmedFireCommand {
            owner,
            armed_source: "chip_redirect#armed[0]".to_owned(),
            participant: bolt,
        }
        .apply(&mut world);
        // No panic — pass.
    }
}
