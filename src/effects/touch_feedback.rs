use bevy::prelude::*;
use bevy::input::touch::{TouchInput, TouchPhase};
use crate::GameplaySystem;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (spawn_touch_feedback, update_touch_feedback).in_set(GameplaySystem));
}

#[derive(Component)]
struct TouchFeedback {
    elapsed: f32,
    duration: f32,
}

fn spawn_touch_feedback(
    mut commands: Commands,
    mut er_touch: EventReader<TouchInput>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_q: Query<&Window>,
) {
    let Ok((camera, cam_gt)) = camera_query.single() else { return; };
    let Ok(_window) = window_q.single() else { return; };
    
    for touch in er_touch.read() {
        // Only create feedback on touch start
        if touch.phase == TouchPhase::Started {
            // Convert screen position to world position
            if let Ok(world_pos) = camera.viewport_to_world_2d(cam_gt, touch.position) {
                commands.spawn((
                    TouchFeedback {
                        elapsed: 0.0,
                        duration: 0.5,
                    },
                    Transform::from_translation(world_pos.extend(0.0)),
                ));
            }
        }
    }
}

fn update_touch_feedback(
    mut commands: Commands,
    mut gizmos: Gizmos,
    mut query: Query<(Entity, &mut TouchFeedback, &Transform)>,
    time: Res<Time>,
) {
    for (entity, mut feedback, transform) in query.iter_mut() {
        feedback.elapsed += time.delta_secs();
        let t = (feedback.elapsed / feedback.duration).clamp(0.0, 1.0);
        
        // Expanding circle effect
        let radius = 2.0 + t * 8.0; // Expands from 2.0 to 10.0
        let alpha = (1.0 - t).powf(2.0); // Fades out
        
        // Cyan/blue color for touch feedback
        let color = Color::srgb(0.3, 0.9, 1.0).with_alpha(0.7 * alpha);
        
        let position = transform.translation.truncate();
        let iso = Isometry2d::from_translation(position);
        gizmos.circle_2d(iso, radius, color);
        
        // Despawn when animation is complete
        if feedback.elapsed >= feedback.duration {
            commands.entity(entity).despawn();
        }
    }
}