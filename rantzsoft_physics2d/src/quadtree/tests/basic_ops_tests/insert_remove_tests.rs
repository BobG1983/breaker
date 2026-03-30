use super::super::{small_aabb, spawn_entities, test_tree};
use crate::collision_layers::CollisionLayers;

#[test]
fn new_creates_empty_tree() {
    let tree = test_tree();
    assert_eq!(tree.len(), 0);
    assert!(tree.is_empty());
}

#[test]
fn insert_increases_len() {
    let mut tree = test_tree();
    let entities = spawn_entities(1);
    tree.insert(
        entities[0],
        small_aabb(10.0, 10.0),
        CollisionLayers::default(),
    );
    assert_eq!(tree.len(), 1);
    assert!(!tree.is_empty());
}

#[test]
#[expect(
    clippy::cast_precision_loss,
    reason = "test loop index is always small"
)]
fn insert_multiple_entities() {
    let mut tree = test_tree();
    let entities = spawn_entities(5);
    for (i, &e) in entities.iter().enumerate() {
        tree.insert(
            e,
            small_aabb(i as f32 * 10.0, 0.0),
            CollisionLayers::default(),
        );
    }
    assert_eq!(tree.len(), 5);
}

#[test]
fn remove_decreases_len() {
    let mut tree = test_tree();
    let entities = spawn_entities(3);
    tree.insert(
        entities[0],
        small_aabb(0.0, 0.0),
        CollisionLayers::default(),
    );
    tree.insert(
        entities[1],
        small_aabb(10.0, 0.0),
        CollisionLayers::default(),
    );
    tree.insert(
        entities[2],
        small_aabb(20.0, 0.0),
        CollisionLayers::default(),
    );

    let removed = tree.remove(entities[1]);
    assert!(removed);
    assert_eq!(tree.len(), 2);
}

#[test]
fn remove_returns_false_for_missing_entity() {
    let mut tree = test_tree();
    let entities = spawn_entities(2);
    tree.insert(
        entities[0],
        small_aabb(0.0, 0.0),
        CollisionLayers::default(),
    );
    // entities[1] was never inserted
    let removed = tree.remove(entities[1]);
    assert!(!removed);
    assert_eq!(tree.len(), 1);
}
