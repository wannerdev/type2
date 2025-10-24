use bevy::prelude::*;
use crate::GameplaySystem;
use crate::sun_system::{Satellite, Level};
use crate::screens::gameplay::CameraZoom;
use std::collections::HashMap;

#[derive(Component)]
pub struct SatelliteLabel;

#[derive(Component)]
pub struct LabelFade {
    pub elapsed: f32,
    pub duration: f32,
    pub fade_in: bool,
}

#[derive(Default, Resource)]
pub struct SatelliteSpawnStats {
    pub global_spawn_idx: u64,
    pub per_level_spawn_idx: HashMap<u32, u32>,
    pub japanese_name_cursor: usize,
}

#[derive(Resource)]
struct LabelPool {
    available: Vec<Entity>,
    in_use: HashMap<Entity, Entity>, // satellite -> label
}

impl Default for LabelPool {
    fn default() -> Self {
        Self {
            available: Vec::new(),
            in_use: HashMap::new(),
        }
    }
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
    "Sakura", "Kitsune", "Kumo", "Hoshi", "Tsuki",
    "Hikari", "Yami", "Kaze", "Mizu", "Inazuma",
    "Tora", "Ryuu", "Kaguya", "Akari", "Kuro",
];

fn next_japanese_codename(stats: &mut SatelliteSpawnStats) -> &'static str {
    let name = JAPANESE_CODENAMES[stats.japanese_name_cursor % JAPANESE_CODENAMES.len()];
    stats.japanese_name_cursor = stats.japanese_name_cursor.wrapping_add(1);
    name
}

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
    app.init_resource::<LabelPool>();
    app.add_systems(Update, (
        update_label_visibility,
        animate_label_fade,
        cleanup_orphaned_labels,
    ).in_set(GameplaySystem));
}

fn update_label_visibility(
    camera_query: Query<&CameraZoom>,
    mut label_query: Query<&mut Visibility, With<SatelliteLabel>>,
) {
    let Ok(zoom) = camera_query.single() else { return; };
    // Only show labels when zoomed in (level 0-2 out of 0-4)
    let show_labels = zoom.level <= 2;
    
    for mut visibility in label_query.iter_mut() {
        *visibility = if show_labels {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn animate_label_fade(
    mut commands: Commands,
    mut query: Query<(Entity, &mut LabelFade, &mut TextColor)>,
    time: Res<Time>,
) {
    for (entity, mut fade, mut color) in query.iter_mut() {
        fade.elapsed += time.delta_secs();
        let t = (fade.elapsed / fade.duration).clamp(0.0, 1.0);
        
        let alpha = if fade.fade_in { t } else { 1.0 - t };
        color.0 = color.0.with_alpha(alpha * 0.9);
        
        if fade.elapsed >= fade.duration {
            commands.entity(entity).remove::<LabelFade>();
            
            // If fading out, hide the label
            if !fade.fade_in {
                commands.entity(entity).insert(Visibility::Hidden);
            }
        }
    }
}

fn cleanup_orphaned_labels(
    mut commands: Commands,
    label_query: Query<(Entity, &Parent), With<SatelliteLabel>>,
    satellite_query: Query<(), With<Satellite>>,
) {
    for (label_entity, parent) in label_query.iter() {
        // Check if parent satellite still exists
        if satellite_query.get(parent.get()).is_err() {
            // Parent satellite was destroyed, despawn label
            commands.entity(label_entity).despawn();
        }
    }
}