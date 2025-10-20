use bevy::prelude::*;
use crate::GameplaySystem;
use crate::sun_system::{Satellite, Sun, SolarSystemAssets};
use crate::collision::HitBox;

#[derive(Component)]
pub struct BrickCollector {
    pub bricks: u32,
}

#[derive(Component)]
pub struct Brick;

#[derive(Component)]
pub struct SunStation {
    pub active: bool,
}

#[derive(Component)]
pub struct DysonSphere {
    pub completion: f32,
}

#[derive(Resource)]
struct BrickSpawnTimer(Timer);

impl Default for BrickSpawnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(30.0, TimerMode::Repeating))
    }
}

#[derive(Resource, Default)]
pub struct DysonSphereProgress {
    pub sun_stations: u32,
    pub total_bricks: u32,
}

pub struct DysonSpherePlugin;

impl Plugin for DysonSpherePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BrickSpawnTimer>();
        app.init_resource::<DysonSphereProgress>();
        app.add_systems(Update, (
            spawn_bricks,
            collect_bricks,
            activate_sun_stations,
            check_dyson_sphere_completion,
            draw_dyson_sphere,
            draw_brick_indicators,
        ).in_set(GameplaySystem));
    }
}

pub fn plugin(app: &mut App) {
    app.add_plugins(DysonSpherePlugin);
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
        
        // Spawn brick in orbit around sun
        let angle = (time.elapsed_secs() * 0.5).sin() * std::f32::consts::TAU;
        let distance = 35.0;
        let position = sun_pos + Vec2::from_angle(angle) * distance;
        
        commands.spawn((
            Brick,
            Transform::from_translation(position.extend(0.0))
                .with_scale(Vec3::splat(0.008)),
            Sprite {
                color: Color::srgb(0.9, 0.9, 0.3),
                custom_size: Some(Vec2::splat(50.0)),
                ..default()
            },
            HitBox { radius: 2.0 },
        ));
        
        info!("Brick spawned at distance {} from sun", distance);
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
                info!("Brick collected! Satellite now has {} bricks", collector.bricks);
                break;
            }
        }
    }
}

fn activate_sun_stations(
    mut query: Query<(&mut BrickCollector, &mut SunStation, &mut Sprite), With<Satellite>>,
    mut progress: ResMut<DysonSphereProgress>,
) {
    for (mut collector, mut station, mut sprite) in query.iter_mut() {
        if collector.bricks >= 3 && !station.active {
            station.active = true;
            collector.bricks -= 3;
            progress.total_bricks += 3;
            
            // Change sprite color to indicate sun station
            sprite.color = Color::srgb(1.0, 0.8, 0.2);
            
            info!("Sun Station activated! Total stations: {}", progress.sun_stations + 1);
        }
    }
}

fn check_dyson_sphere_completion(
    mut commands: Commands,
    sun_station_query: Query<(Entity, &SunStation), With<Satellite>>,
    sun_query: Query<&Transform, With<Sun>>,
    mut progress: ResMut<DysonSphereProgress>,
    dyson_query: Query<(), With<DysonSphere>>,
) {
    let active_stations = sun_station_query.iter().filter(|(_, s)| s.active).count();
    progress.sun_stations = active_stations as u32;
    
    // Check if Dyson sphere already exists
    if dyson_query.iter().count() > 0 {
        return;
    }
    
    if active_stations >= 4 {
        let Ok(sun_transform) = sun_query.single() else { return; };
        
        // Spawn Dyson Sphere (don't despawn stations in simplified version)
        commands.spawn((
            DysonSphere { completion: 1.0 },
            Transform::from_translation(sun_transform.translation),
        ));
        
        info!("🎉 DYSON SPHERE COMPLETE! 🎉");
    }
}

fn draw_dyson_sphere(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &DysonSphere)>,
    sun_query: Query<&HitBox, With<Sun>>,
) {
    for (transform, sphere) in query.iter() {
        let Ok(sun_hitbox) = sun_query.single() else { return; };
        let position = transform.translation.truncate();
        let radius = sun_hitbox.radius * 1.8;
        
        // Draw sphere segments
        let segments = 32;
        let completion_segments = (segments as f32 * sphere.completion) as usize;
        
        for i in 0..completion_segments {
            let angle_start = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let angle_end = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
            
            let start = position + Vec2::from_angle(angle_start) * radius;
            let end = position + Vec2::from_angle(angle_end) * radius;
            
            let color = Color::srgb(0.9, 0.9, 0.3).with_alpha(0.8);
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
            
            // Draw small indicators for collected bricks
            for i in 0..collector.bricks.min(3) {
                let offset = Vec2::new((i as f32 - 1.0) * 3.0, 8.0);
                let brick_pos = position + offset;
                let iso = Isometry2d::from_translation(brick_pos);
                gizmos.circle_2d(iso, 1.0, Color::srgb(0.9, 0.9, 0.3));
            }
        }
    }
}