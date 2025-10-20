use bevy::prelude::*;
use crate::GameplaySystem;
use crate::sun_system::{Satellite, Sun};
use crate::collision::HitBox;

#[derive(Component)]
pub struct BrickCollector {
    pub bricks: u32,
}

#[derive(Component)]
pub struct Brick;

#[derive(Component)]
pub struct SunStation {
    pub tier: u32, // 0 = inactive, 1-3 = upgrade tiers
}

#[derive(Component)]
pub struct DysonSphere {
    pub completion: f32, // 0.0 to 1.0
    construction_timer: f32,
}

#[derive(Resource)]
struct BrickSpawnTimer(Timer);

impl Default for BrickSpawnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(25.0, TimerMode::Repeating))
    }
}

#[derive(Resource, Default)]
pub struct DysonSphereProgress {
    pub total_bricks_invested: u32,
}

pub struct DysonSphereProgressivePlugin;

impl Plugin for DysonSphereProgressivePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BrickSpawnTimer>();
        app.init_resource::<DysonSphereProgress>();
        app.add_systems(Update, (
            spawn_bricks,
            collect_bricks,
            upgrade_sun_stations,
            build_dyson_sphere_progressively,
            animate_sphere_construction,
            draw_dyson_sphere_progressive,
            draw_brick_indicators,
            draw_station_tier_indicators,
        ).in_set(GameplaySystem));
    }
}

pub fn plugin(app: &mut App) {
    app.add_plugins(DysonSphereProgressivePlugin);
}

fn spawn_bricks(
    mut commands: Commands,
    mut timer: ResMut<BrickSpawnTimer>,
    time: Res<Time>,
    sun_query: Query<&Transform, With<Sun>>,
) {
    timer.0.tick(time.delta());
    
    if timer.0.just_finished() {
        let Ok(sun_transform) = sun_query.single() else { return; };
        let sun_pos = sun_transform.translation.truncate();
        
        let angle = (time.elapsed_secs() * 0.3).cos() * std::f32::consts::TAU;
        let distance = 40.0;
        let position = sun_pos + Vec2::from_angle(angle) * distance;
        
        commands.spawn((
            Brick,
            Transform::from_translation(position.extend(0.0))
                .with_scale(Vec3::splat(0.008)),
            Sprite {
                color: Color::srgb(0.8, 0.9, 0.4),
                custom_size: Some(Vec2::splat(50.0)),
                ..default()
            },
            HitBox { radius: 2.0 },
        ));
    }
}

fn collect_bricks(
    mut commands: Commands,
    brick_query: Query<(Entity, &Transform, &HitBox), With<Brick>>,
    mut satellite_query: Query<(&Transform, &HitBox, &mut BrickCollector), With<Satellite>>,
) {
    for (sat_transform, sat_hitbox, mut collector) in satellite_query.iter_mut() {
        let sat_pos = sat_transform.translation.truncate();
        
        for (brick_entity, brick_transform, brick_hitbox) in brick_query.iter() {
            let brick_pos = brick_transform.translation.truncate();
            let distance = sat_pos.distance(brick_pos);
            
            if distance < (sat_hitbox.radius + brick_hitbox.radius) {
                collector.bricks += 1;
                commands.entity(brick_entity).despawn();
                info!("Brick collected! Total: {}", collector.bricks);
                break;
            }
        }
    }
}

fn upgrade_sun_stations(
    mut query: Query<(&mut BrickCollector, &mut SunStation, &mut Sprite), With<Satellite>>,
    mut progress: ResMut<DysonSphereProgress>,
) {
    for (mut collector, mut station, mut sprite) in query.iter_mut() {
        // Tier 1: 2 bricks
        if collector.bricks >= 2 && station.tier == 0 {
            station.tier = 1;
            collector.bricks -= 2;
            progress.total_bricks_invested += 2;
            sprite.color = Color::srgb(1.0, 0.9, 0.3);
            info!("Sun Station Tier 1 activated!");
        }
        // Tier 2: 3 more bricks (5 total)
        else if collector.bricks >= 3 && station.tier == 1 {
            station.tier = 2;
            collector.bricks -= 3;
            progress.total_bricks_invested += 3;
            sprite.color = Color::srgb(1.0, 0.8, 0.2);
            info!("Sun Station Tier 2 activated!");
        }
        // Tier 3: 5 more bricks (10 total)
        else if collector.bricks >= 5 && station.tier == 2 {
            station.tier = 3;
            collector.bricks -= 5;
            progress.total_bricks_invested += 5;
            sprite.color = Color::srgb(1.0, 0.7, 0.1);
            info!("Sun Station Tier 3 activated!");
        }
    }
}

fn build_dyson_sphere_progressively(
    mut commands: Commands,
    sun_station_query: Query<&SunStation, With<Satellite>>,
    sun_query: Query<&Transform, With<Sun>>,
    mut dyson_query: Query<&mut DysonSphere>,
    progress: Res<DysonSphereProgress>,
) {
    let total_tier = sun_station_query.iter().map(|s| s.tier).sum::<u32>();
    
    // Start building sphere at tier 4 total (e.g., 2 tier-2 stations)
    if total_tier >= 4 {
        if dyson_query.iter().count() == 0 {
            let Ok(sun_transform) = sun_query.single() else { return; };
            commands.spawn((
                DysonSphere { 
                    completion: 0.0,
                    construction_timer: 0.0,
                },
                Transform::from_translation(sun_transform.translation),
            ));
            info!("Dyson Sphere construction started!");
        }
    }
}

fn animate_sphere_construction(
    mut query: Query<&mut DysonSphere>,
    sun_station_query: Query<&SunStation, With<Satellite>>,
    time: Res<Time>,
) {
    for mut sphere in query.iter_mut() {
        let total_tier = sun_station_query.iter().map(|s| s.tier).sum::<u32>();
        let target_completion = (total_tier as f32 / 12.0).min(1.0); // Max at tier 12
        
        if sphere.completion < target_completion {
            sphere.construction_timer += time.delta_secs();
            // Build 10% per second
            sphere.completion = (sphere.completion + time.delta_secs() * 0.1).min(target_completion);
            
            if sphere.completion >= 1.0 {
                info!("🎉 DYSON SPHERE 100% COMPLETE! 🎉");
            }
        }
    }
}

fn draw_dyson_sphere_progressive(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &DysonSphere)>,
    sun_query: Query<&HitBox, With<Sun>>,
    time: Res<Time>,
) {
    for (transform, sphere) in query.iter() {
        let Ok(sun_hitbox) = sun_query.single() else { return; };
        let position = transform.translation.truncate();
        let radius = sun_hitbox.radius * 1.8;
        
        let segments = 32;
        let completion_segments = (segments as f32 * sphere.completion) as usize;
        
        for i in 0..completion_segments {
            let angle_start = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let angle_end = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
            
            let start = position + Vec2::from_angle(angle_start) * radius;
            let end = position + Vec2::from_angle(angle_end) * radius;
            
            // Pulsing effect on construction edge
            let is_edge = i == completion_segments - 1;
            let pulse = if is_edge {
                ((time.elapsed_secs() * 3.0).sin() * 0.3 + 0.7)
            } else {
                0.8
            };
            
            let color = Color::srgb(0.9, 0.9, 0.3).with_alpha(pulse);
            gizmos.line_2d(start, end, color);
        }
    }
}

fn draw_brick_indicators(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &BrickCollector), With<Satellite>>,
) {
    for (transform, collector) in query.iter() {
        if collector.bricks > 0 {
            let position = transform.translation.truncate();
            
            for i in 0..collector.bricks.min(5) {
                let offset = Vec2::new((i as f32 - 2.0) * 2.5, 8.0);
                let brick_pos = position + offset;
                let iso = Isometry2d::from_translation(brick_pos);
                gizmos.circle_2d(iso, 0.8, Color::srgb(0.8, 0.9, 0.4));
            }
        }
    }
}

fn draw_station_tier_indicators(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &SunStation), With<Satellite>>,
) {
    for (transform, station) in query.iter() {
        if station.tier > 0 {
            let position = transform.translation.truncate();
            
            // Draw tier rings
            for i in 0..station.tier {
                let radius = 7.0 + i as f32 * 1.5;
                let alpha = 0.3 + (i as f32 * 0.1);
                let color = Color::srgb(1.0, 0.8, 0.2).with_alpha(alpha);
                let iso = Isometry2d::from_translation(position);
                gizmos.circle_2d(iso, radius, color);
            }
        }
    }
}