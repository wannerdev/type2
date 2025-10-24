use bevy::prelude::*;
use crate::GameplaySystem;
use crate::sun_system::{Satellite, Sun};

pub struct AchievementsPlugin;

#[derive(Event, Debug, Copy, Clone)]
pub struct FullOrbitAchieved {
    pub entity: Entity,
}

#[derive(Event, Debug, Copy, Clone)]
pub struct ClosestOrbitAchieved {
    pub entity: Entity,
    pub distance: f32,
}

impl Plugin for AchievementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_full_orbit);
        app.add_observer(on_closest_orbit);
        app.add_systems(Update, (update_neon_circle_fx, track_closest_orbit).in_set(GameplaySystem));
        app.insert_resource(ClosestOrbitRecord { distance: f32::MAX, entity: None });
    }
}

#[derive(Resource)]
struct ClosestOrbitRecord {
    distance: f32,
    entity: Option<Entity>,
}

#[derive(Component, Debug, Copy, Clone)]
pub struct FullOrbitAwarded;

#[derive(Component, Debug, Copy, Clone)]
pub struct ClosestOrbitAwarded;

#[derive(Component, Debug, Copy, Clone)]
struct NeonCircleFx {
    center: Vec2,
    elapsed: f32,
    duration: f32,
    color: Color,
}

fn on_full_orbit(
    ev: On<FullOrbitAchieved>,
    q_t: Query<&Transform>,
    mut commands: Commands,
) {
    if let Ok(t) = q_t.get(ev.entity) {
        commands.spawn(NeonCircleFx { 
            center: t.translation.xy(), 
            elapsed: 0.2,
            duration: 1.0,
            color: Color::srgb(0.2, 0.8, 1.0),
        });
    }
}

fn on_closest_orbit(
    ev: On<ClosestOrbitAchieved>,
    q_t: Query<&Transform>,
    mut commands: Commands,
) {
    if let Ok(t) = q_t.get(ev.entity) {
        // Golden/orange color for closest orbit achievement
        commands.spawn(NeonCircleFx { 
            center: t.translation.xy(), 
            elapsed: 0.2,
            duration: 1.5,
            color: Color::srgb(1.0, 0.6, 0.0),
        });
        info!("Closest orbit achievement! Distance: {:.2}", ev.distance);
    }
}

fn track_closest_orbit(
    satellite_query: Query<(Entity, &Transform), (With<Satellite>, Without<ClosestOrbitAwarded>)>,
    sun_query: Query<&Transform, With<Sun>>,
    mut closest_record: ResMut<ClosestOrbitRecord>,
    mut commands: Commands,
) {
    let Ok(sun_transform) = sun_query.single() else { return; };
    let sun_position = sun_transform.translation;

    for (entity, sat_transform) in satellite_query.iter() {
        let distance = sat_transform.translation.distance(sun_position);
        
        // Track if this is a new record
        if distance < closest_record.distance {
            closest_record.distance = distance;
            closest_record.entity = Some(entity);
            
            // Award achievement
            commands.entity(entity).insert(ClosestOrbitAwarded);
            commands.trigger(ClosestOrbitAchieved { entity, distance });
        }
    }
}

fn update_neon_circle_fx(
    mut commands: Commands,
    mut gizmos: Gizmos,
    mut q: Query<(Entity, &mut NeonCircleFx)>,
    time: Res<Time>,
) {
    for (e, mut fx) in q.iter_mut() {
        fx.elapsed += time.delta_secs();
        let t = (fx.elapsed / fx.duration).clamp(0.0, 1.0);

        let radius = 3.0 + 6.0 * t; // expand
        let alpha = (1.0 - t).powf(2.0); // fade out
        let color = fx.color.with_alpha(0.75 * alpha);

        let iso = Isometry2d::from_translation(fx.center);
        gizmos.circle_2d(iso, radius, color);

        if fx.elapsed >= fx.duration { commands.entity(e).despawn(); }
    }
}