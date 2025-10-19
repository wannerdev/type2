use bevy::prelude::*;
use crate::GameplaySystem;
use crate::launching::{LaunchArmed, LaunchPad};
use bevy::input::touch::TouchInput;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, draw_launch_arrow.in_set(GameplaySystem));
}

fn draw_launch_arrow(
    mut gizmos: Gizmos,
    launch_armed: Res<LaunchArmed>,
    launch_pad_query: Query<&Transform, With<LaunchPad>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_q: Query<&Window>,
    mut er_touch: EventReader<TouchInput>,
) {
    // Only show arrow when launch is armed (earth is selected on mobile)
    if !launch_armed.0 {
        return;
    }
    
    let Ok(launch_pad_transform) = launch_pad_query.single() else { return; };
    let launch_position = launch_pad_transform.translation.truncate();
    
    let Ok((camera, cam_gt)) = camera_query.single() else { return; };
    let Ok(window) = window_q.single() else { return; };
    
    // Get the last touch position if available
    let mut touch_pos: Option<Vec2> = None;
    for touch in er_touch.read() {
        touch_pos = Some(touch.position);
    }
    
    // If we have a touch position, draw arrow from earth to touch
    if let Some(screen_pos) = touch_pos {
        if let Ok(world_pos) = camera.viewport_to_world_2d(cam_gt, screen_pos) {
            let direction = (world_pos - launch_position).normalize_or_zero();
            
            if direction != Vec2::ZERO {
                // Draw main arrow line
                let arrow_length = 15.0;
                let arrow_end = launch_position + direction * arrow_length;
                
                // Orange/amber color to match game theme
                let color = Color::srgb(1.0, 0.6, 0.2).with_alpha(0.8);
                
                // Main arrow shaft
                gizmos.line_2d(launch_position, arrow_end, color);
                
                // Arrow head (two lines forming a V)
                let perpendicular = Vec2::new(-direction.y, direction.x);
                let head_size = 3.0;
                let head_angle = 0.4; // angle of arrow head
                
                let head_left = arrow_end - direction * head_size + perpendicular * head_size * head_angle;
                let head_right = arrow_end - direction * head_size - perpendicular * head_size * head_angle;
                
                gizmos.line_2d(arrow_end, head_left, color);
                gizmos.line_2d(arrow_end, head_right, color);
            }
        }
    }
}