use super::helpers::*;
use crate::{prelude::*, state::run::node::definition::NodePool};

// --- A6: NodeLayout.sequences RON integration ---

/// Layout with a single sequence group placing (0,0) at position 0 and (0,1)
/// at position 1 should attach `SequenceCell`, `SequenceGroup(7)`, and
/// `SequencePosition` to the matching cells.
#[test]
fn sequences_layout_field_attaches_group_and_position_components() {
    use std::collections::HashMap;

    use crate::{
        cells::behaviors::sequence::components::{SequenceCell, SequenceGroup, SequencePosition},
        state::run::node::definition::SequenceMap,
    };

    let mut sequences: SequenceMap = HashMap::new();
    sequences.insert(7, vec![(0, 0), (0, 1)]);
    let layout = NodeLayout {
        name:            "sequence_layout".to_owned(),
        timer_secs:      60.0,
        cols:            2,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("N"), s("N")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           None,
        sequences:       Some(sequences),
    };
    let mut app = behavior_test_app(layout, behavior_registry());
    app.update();

    let members: Vec<(u32, u32)> = app
        .world_mut()
        .query::<(&SequenceCell, &SequenceGroup, &SequencePosition)>()
        .iter(app.world())
        .map(|(_, group, position)| (group.0, position.0))
        .collect();
    assert_eq!(
        members.len(),
        2,
        "both cells in the group should have sequence components attached"
    );
    assert!(
        members.contains(&(7, 0)),
        "position 0 in group 7 should be attached, got {members:?}"
    );
    assert!(
        members.contains(&(7, 1)),
        "position 1 in group 7 should be attached, got {members:?}"
    );
}

/// Cells outside the `sequences` map must not receive sequence components.
#[test]
fn cells_outside_sequences_map_do_not_receive_sequence_components() {
    use std::collections::HashMap;

    use crate::{
        cells::behaviors::sequence::components::SequenceCell,
        state::run::node::definition::SequenceMap,
    };

    let mut sequences: SequenceMap = HashMap::new();
    sequences.insert(0, vec![(0, 0)]);
    let layout = NodeLayout {
        name:            "mixed_sequence".to_owned(),
        timer_secs:      60.0,
        cols:            2,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("N"), s("N")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           None,
        sequences:       Some(sequences),
    };
    let mut app = behavior_test_app(layout, behavior_registry());
    app.update();

    let sequence_count = app
        .world_mut()
        .query::<&SequenceCell>()
        .iter(app.world())
        .count();
    assert_eq!(
        sequence_count, 1,
        "only the cell at (0,0) should be a sequence member"
    );
}
