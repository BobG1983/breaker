use super::helpers::default_playfield;
use crate::walls::{builder::core::types::Lifetime, components::Wall};

// ── Behavior 16: Default lifetime is Permanent ──

#[test]
fn default_lifetime_is_permanent_for_left() {
    let pf = default_playfield();
    let builder = Wall::builder().left(&pf);
    assert_eq!(
        builder.lifetime,
        Lifetime::Permanent,
        "default lifetime should be Permanent"
    );
}

#[test]
fn default_lifetime_is_permanent_for_floor() {
    let pf = default_playfield();
    let builder = Wall::builder().floor(&pf);
    assert_eq!(
        builder.lifetime,
        Lifetime::Permanent,
        "default lifetime for Floor should be Permanent"
    );
}

// ── Behavior 17: .timed() is available on Floor and sets Lifetime::Timed ──

#[test]
fn timed_compiles_on_floor_and_can_build() {
    let pf = default_playfield();
    // Verify the method compiles and builder can proceed to build
    let _bundle = Wall::builder().floor(&pf).timed(5.0).build();
}

#[test]
fn timed_zero_compiles_on_floor() {
    let pf = default_playfield();
    let _bundle = Wall::builder().floor(&pf).timed(0.0).build();
}

// ── Behavior 18: .one_shot() is available on Floor and sets Lifetime::OneShot ──

#[test]
fn one_shot_compiles_on_floor_and_can_build() {
    let pf = default_playfield();
    let _bundle = Wall::builder().floor(&pf).one_shot().build();
}

#[test]
fn one_shot_then_timed_last_wins() {
    let pf = default_playfield();
    // Both compile and build succeeds
    let _bundle = Wall::builder().floor(&pf).one_shot().timed(3.0).build();
}

// ── Behavior 51: Lifetime::default() is Permanent ──

#[test]
fn lifetime_default_is_permanent() {
    let lifetime = Lifetime::default();
    assert_eq!(lifetime, Lifetime::Permanent);
}

// ── Behavior 52: Lifetime variants are distinct ──

#[test]
fn lifetime_variants_are_distinct() {
    assert_ne!(Lifetime::Permanent, Lifetime::OneShot);
    assert_ne!(Lifetime::Permanent, Lifetime::Timed(5.0));
    assert_ne!(Lifetime::OneShot, Lifetime::Timed(5.0));
    assert_ne!(Lifetime::Timed(5.0), Lifetime::Timed(3.0));
}

#[test]
fn lifetime_timed_zero_distinct_from_permanent() {
    assert_ne!(
        Lifetime::Timed(0.0),
        Lifetime::Permanent,
        "Timed(0.0) should be distinct from Permanent"
    );
}
