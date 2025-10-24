use bevy::prelude::*;
use crate::GameplaySystem;
use crate::sun_system::{Satellite, Level};
use std::collections::HashMap;

#[derive(Component)]
pub struct SatelliteLabel;

#[derive(Default, Resource)]
pub struct SatelliteSpawnStats {
    pub global_spawn_idx: u64,
    pub per_level_spawn_idx: HashMap<u32, u32>, // level -> count
    pub japanese_name_cursor: usize,
}

// Fixed codenames for the first satellite per level
fn first_of_level_codename(level: f32) -> Option<&'static str> {
    match level as i32 {
        1 => Some("tiny-1"),
        2 => Some("traily"),
        3 => Some("boinc"),
        _ => None,
    }
}

// Japanese codenames (using romaji for ASCII compatibility)
static JAPANESE_CODENAMES: &[&str] = &[
    "Sakura",
    "Kitsune",
    "Kumo",
    "Hoshi",
    "Tsuki",
    "Hikari",
    "Yami",
    "Kaze",
    "Mizu",
    "Inazuma",
    "Tora",
    "Ryuu",
    "Kaguya",
    "Akari",
    "Kuro",
];

fn next_japanese_codename(stats: &mut SatelliteSpawnStats) -> &'static str {
    let name = JAPANESE_CODENAMES[stats.japanese_name_cursor % JAPANESE_CODENAMES.len()];
    stats.japanese_name_cursor = stats.japanese_name_cursor.wrapping_add(1);
    name
}

// Decide codename for a satellite being spawned
pub fn codename_for_satellite(
    level: f32,
    level_spawn_idx: u32,
    global_spawn_idx: u64,
    stats: &mut SatelliteSpawnStats,
) -> Option<String> {
    if level_spawn_idx == 1 {
        return first_of_level_codename(level).map(|s| s.to_string());
    }
    if global_spawn_idx % 10 == 0 {
        return Some(next_japanese_codename(stats).to_string());
    }
    None
}

pub fn plugin(app: &mut App) {
    app.init_resource::<SatelliteSpawnStats>();
    app.add_systems(Update, update_satellite_labels.in_set(GameplaySystem));
}

fn update_satellite_labels(
    satellite_query: Query<(&Transform, &Children), With<Satellite>>,
    mut label_query: Query<&mut Transform, (With<SatelliteLabel>, Without<Satellite>)>,
) {
    for (sat_transform, children) in satellite_query.iter() {
        for &child in children.iter() {
            if let Ok(mut label_transform) = label_query.get_mut(child) {
                // Keep label positioned below satellite
                // The label is already a child, so we just need to maintain relative position
                label_transform.translation = Vec3::new(0.0, -800.0, 1.0);
            }
        }
    }
}