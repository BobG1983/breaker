use super::super::*;

// =========================================================================
// RootEffect — construction and serde (Part B)
// =========================================================================

#[test]
fn root_effect_on_deserializes_from_ron() {
    let ron_str = "On(target: Breaker, then: [When(trigger: OnBoltLost, then: [Do(LoseLife)])])";
    let root: RootEffect = ron::de::from_str(ron_str).expect("RootEffect On RON should parse");
    match &root {
        RootEffect::On { target, then } => {
            assert_eq!(*target, Target::Breaker);
            assert_eq!(then.len(), 1);
            assert!(matches!(
                &then[0],
                EffectNode::When {
                    trigger: Trigger::BoltLost,
                    ..
                }
            ));
        }
    }
}

#[test]
fn root_effect_rejects_non_on_variant() {
    let ron_str = "When(trigger: OnPerfectBump, then: [Do(LoseLife)])";
    let result = ron::de::from_str::<RootEffect>(ron_str);
    assert!(
        result.is_err(),
        "RootEffect should reject non-On variants like When"
    );
}
