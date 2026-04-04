//! Transition type and kind enums.

use std::{any::TypeId, sync::Arc};

use bevy::ecs::world::World;

use super::traits::{InTransition, OneShotTransition, OutTransition};

/// Describes which effect(s) to play during a state transition.
///
/// Each variant wraps one or two `Arc`-ed trait objects. `Arc` is used
/// (not `Box`) because `TransitionType` must be cheaply clonable for
/// storage in routes and the registry.
///
/// Compile-time safety: `TransitionType::Out` only accepts `dyn OutTransition`,
/// `TransitionType::In` only accepts `dyn InTransition`, etc. A type that
/// only implements `Transition` (not the specific sub-trait) will NOT compile
/// in the wrong variant.
pub enum TransitionType {
    /// Hide current content, then change state.
    Out(Arc<dyn OutTransition>),
    /// Change state, then reveal new content.
    In(Arc<dyn InTransition>),
    /// Hide current content, change state, then reveal new content.
    OutIn {
        /// The effect that hides the current content.
        out_e: Arc<dyn OutTransition>,
        /// The effect that reveals the new content.
        in_e: Arc<dyn InTransition>,
    },
    /// Change state while playing an effect over both old and new content.
    OneShot(Arc<dyn OneShotTransition>),
}

impl TransitionType {
    /// Returns the `TypeId`s of the concrete effect types contained in this
    /// transition.
    ///
    /// - `Out` / `In` / `OneShot`: returns a single-element vec.
    /// - `OutIn`: returns two elements — out first, in second.
    #[must_use]
    pub fn type_ids(&self) -> Vec<TypeId> {
        match self {
            Self::Out(t) => vec![(**t).type_id()],
            Self::In(t) => vec![(**t).type_id()],
            Self::OutIn { out_e, in_e } => vec![(**out_e).type_id(), (**in_e).type_id()],
            Self::OneShot(t) => vec![(**t).type_id()],
        }
    }
}

impl Clone for TransitionType {
    fn clone(&self) -> Self {
        match self {
            Self::Out(t) => Self::Out(Arc::clone(t)),
            Self::In(t) => Self::In(Arc::clone(t)),
            Self::OutIn { out_e, in_e } => Self::OutIn {
                out_e: Arc::clone(out_e),
                in_e: Arc::clone(in_e),
            },
            Self::OneShot(t) => Self::OneShot(Arc::clone(t)),
        }
    }
}

/// Internal representation of how a route's transition is configured.
///
/// `None` means no transition (instant state change, preserving existing
/// behavior). `Static` means a fixed `TransitionType`. `Dynamic` means
/// the `TransitionType` is computed at dispatch time from `&World`.
#[doc(hidden)]
pub enum TransitionKind {
    /// No transition effect — instant state change.
    None,
    /// Fixed transition effect.
    Static(TransitionType),
    /// Transition effect computed at dispatch time.
    Dynamic(Box<dyn Fn(&World) -> TransitionType + Send + Sync>),
}

#[cfg(test)]
mod tests {
    use std::{any::TypeId, sync::Arc};

    use super::*;
    use crate::transition::traits::{InTransition, OneShotTransition, OutTransition, Transition};

    struct TestEffectOut;
    impl Transition for TestEffectOut {}
    impl OutTransition for TestEffectOut {}

    struct TestEffectIn;
    impl Transition for TestEffectIn {}
    impl InTransition for TestEffectIn {}

    struct TestEffectOneShot;
    impl Transition for TestEffectOneShot {}
    impl OneShotTransition for TestEffectOneShot {}

    struct TestEffectBoth;
    impl Transition for TestEffectBoth {}
    impl InTransition for TestEffectBoth {}
    impl OutTransition for TestEffectBoth {}

    // --- Section A2 behavior 1: OutTransition wraps into TransitionType::Out ---

    #[test]
    fn out_transition_wraps_in_arc_into_transition_type_out() {
        let tt = TransitionType::Out(Arc::new(TestEffectOut));
        assert!(matches!(tt, TransitionType::Out(_)));
    }

    // --- Section A2 behavior 2: InTransition wraps into TransitionType::In ---

    #[test]
    fn in_transition_wraps_in_arc_into_transition_type_in() {
        let tt = TransitionType::In(Arc::new(TestEffectIn));
        assert!(matches!(tt, TransitionType::In(_)));
    }

    // --- Section A2 behavior 3: OutIn requires both out and in ---

    #[test]
    fn outin_requires_both_out_and_in_effects() {
        let tt = TransitionType::OutIn {
            out_e: Arc::new(TestEffectOut),
            in_e: Arc::new(TestEffectIn),
        };
        assert!(matches!(tt, TransitionType::OutIn { .. }));
    }

    #[test]
    fn outin_accepts_type_implementing_both_traits_for_both_fields() {
        let tt = TransitionType::OutIn {
            out_e: Arc::new(TestEffectBoth),
            in_e: Arc::new(TestEffectBoth),
        };
        assert!(matches!(tt, TransitionType::OutIn { .. }));
    }

    // --- Section A2 behavior 4: OneShotTransition wraps into TransitionType::OneShot ---

    #[test]
    fn oneshot_transition_wraps_in_arc_into_transition_type_oneshot() {
        let tt = TransitionType::OneShot(Arc::new(TestEffectOneShot));
        assert!(matches!(tt, TransitionType::OneShot(_)));
    }

    // --- Section A2 behavior 5: TransitionType variant can be matched ---

    #[test]
    fn transition_type_matches_correct_variant() {
        let tt = TransitionType::Out(Arc::new(TestEffectOut));
        assert!(matches!(tt, TransitionType::Out(_)));
        assert!(!matches!(tt, TransitionType::In(_)));
    }

    // --- Section A2 behavior 6: type_ids returns TypeIds of contained effects ---

    #[test]
    fn type_ids_returns_single_type_id_for_out() {
        let tt = TransitionType::Out(Arc::new(TestEffectOut));
        let ids = tt.type_ids();
        assert_eq!(ids, vec![TypeId::of::<TestEffectOut>()]);
    }

    #[test]
    fn type_ids_returns_two_type_ids_for_outin() {
        let tt = TransitionType::OutIn {
            out_e: Arc::new(TestEffectOut),
            in_e: Arc::new(TestEffectIn),
        };
        let ids = tt.type_ids();
        assert_eq!(
            ids,
            vec![TypeId::of::<TestEffectOut>(), TypeId::of::<TestEffectIn>()]
        );
    }
}
