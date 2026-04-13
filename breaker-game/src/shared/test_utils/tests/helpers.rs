use bevy::prelude::*;

use crate::{
    bolt::{definition::BoltDefinition, messages::BoltLost},
    cells::{
        definition::{CellTypeDefinition, Toughness},
        messages::DamageCell,
    },
};

// ── Test helper types ──────────────────────────────────────────────

#[derive(Resource, Default)]
pub(super) struct Counter(pub u32);

#[derive(Resource, Default)]
pub(super) struct Order(pub String);

#[derive(Resource)]
pub(super) struct ShouldSend(pub bool);

pub(super) fn increment(mut counter: ResMut<Counter>) {
    counter.0 += 1;
}

pub(super) fn damage_sender_system(mut writer: MessageWriter<DamageCell>) {
    writer.write(DamageCell {
        cell:        Entity::PLACEHOLDER,
        damage:      25.0,
        source_chip: None,
    });
}

pub(super) fn conditional_damage_sender(
    flag: Res<ShouldSend>,
    mut writer: MessageWriter<DamageCell>,
) {
    if flag.0 {
        writer.write(DamageCell {
            cell:        Entity::PLACEHOLDER,
            damage:      10.0,
            source_chip: None,
        });
    }
}

pub(super) fn triple_damage_sender(mut writer: MessageWriter<DamageCell>) {
    for i in 0_i16..3 {
        writer.write(DamageCell {
            cell:        Entity::PLACEHOLDER,
            damage:      f32::from(i + 1),
            source_chip: None,
        });
    }
}

pub(super) fn damage_and_bolt_lost_sender(
    mut damage_writer: MessageWriter<DamageCell>,
    mut bolt_lost_writer: MessageWriter<BoltLost>,
) {
    damage_writer.write(DamageCell {
        cell:        Entity::PLACEHOLDER,
        damage:      5.0,
        source_chip: None,
    });
    bolt_lost_writer.write(BoltLost {
        bolt:    Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    });
}

pub(super) fn first_system(mut order: ResMut<Order>) {
    if !order.0.is_empty() {
        order.0.push(',');
    }
    order.0.push_str("first");
}

pub(super) fn second_system(mut order: ResMut<Order>) {
    if !order.0.is_empty() {
        order.0.push(',');
    }
    order.0.push_str("second");
}

/// Helper: constructs a `BoltDefinition` with all required fields.
pub(super) fn make_bolt_definition(name: &str, base_speed: f32) -> BoltDefinition {
    BoltDefinition {
        name: name.to_string(),
        base_speed,
        min_speed: 200.0,
        max_speed: 800.0,
        radius: 8.0,
        base_damage: 10.0,
        effects: vec![],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    }
}

/// Helper: constructs a `CellTypeDefinition` with all required fields.
pub(super) fn make_cell_definition(alias: &str) -> CellTypeDefinition {
    CellTypeDefinition {
        id:                alias.to_lowercase(),
        alias:             alias.to_string(),
        toughness:         Toughness::Standard,
        color_rgb:         [1.0, 1.0, 1.0],
        required_to_clear: true,
        damage_hdr_base:   2.0,
        damage_green_min:  0.1,
        damage_blue_range: 0.5,
        damage_blue_base:  0.2,
        behaviors:         None,
        effects:           None,
    }
}
