//! Group D — `IRON_CURTAIN_SENTINEL` drift guard (Behavior 26).
//!
//! Pins the exact literal value of the sentinel const. Any refactor that
//! renames or retypes the const must also update this test.

use super::super::system::IRON_CURTAIN_SENTINEL;

#[test]
fn iron_curtain_sentinel_value_is_protocol_iron_curtain() {
    assert_eq!(
        IRON_CURTAIN_SENTINEL, "protocol:iron_curtain",
        "IRON_CURTAIN_SENTINEL drift guard — must be exactly \
         \"protocol:iron_curtain\"; got {IRON_CURTAIN_SENTINEL:?}"
    );
}
