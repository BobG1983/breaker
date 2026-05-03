//! `tick_phantom_breaker_lifespan` — decrements phantom lifespan and emits `DespawnEntity`.

use bevy::prelude::*;

use crate::{
    breaker::components::{Breaker, PhantomBreaker},
    prelude::*,
    shared::Lifespan,
};

type PhantomBreakerLifespanQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static mut Lifespan), (With<Breaker>, With<PhantomBreaker>)>;

pub(crate) fn tick_phantom_breaker_lifespan(
    time: Res<Time<Fixed>>,
    mut query: PhantomBreakerLifespanQuery,
    mut writer: MessageWriter<DespawnEntity>,
) {
    let dt = time.delta_secs();
    for (entity, mut lifespan) in &mut query {
        lifespan.remaining -= dt;
        if lifespan.remaining <= 0.0 {
            writer.write(DespawnEntity { entity });
        }
    }
}
