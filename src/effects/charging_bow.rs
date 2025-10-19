use bevy::prelude::*;
use crate::GameplaySystem;
use crate::launching::{LaunchArmed, LaunchPad, LaunchState};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, draw_charging_bow.in_set(GameplaySystem));
}

fn draw_charging_bow(
    mut gizmos: Gizmos,
    launch_armed: Res<LaunchArmed>,
    launch_state: Res<LaunchState>,
    launch_pad_query: Query<&Transform, With<LaunchPad>>,
    time: Res<Time>,
) {
    // Only show bow when launch is armed and touch is being held
    if !launch_armed.0 {
        return;
    }
    
    let Some(launch_start_time) = launch_state.launched_at_time else { return; };
    
    let Ok(launch_pad_transform) = launch_pad_query.single() else { return; };
    let earth_position = launch_pad_transform.translation.truncate();
    
    // Calculate how long the touch has been held (capped at 1.0 seconds)
    let held_duration = (time.elapsed_secs_f64() - launch_start_time).min(1.0) as f32;
    let charge_percent = held_duration; // 0.0 to 1.0
    
    // Only draw if there's some charge
    if charge_percent < 0.05 {
        return;
    }
    
    // Draw a bow/arc behind Earth that grows with charge
    let bow_radius = 8.0 + charge_percent * 6.0; // 8.0 to 14.0
    let bow_thickness = 1.0 + charge_percent * 2.0; // Visual thickness effect
    
    // Color transitions from orange to bright yellow as it charges
    let color = Color::srgb(
        1.0,
        0.4 + charge_percent * 0.6, // 0.4 to 1.0 (orange to yellow)
        0.0
    ).with_alpha(0.6 + charge_percent * 0.4); // 0.6 to 1.0
    
    // Draw multiple arcs for thickness effect
    for i in 0..3 {
        let offset = i as f32 * 0.5;
        let current_radius = bow_radius + offset;
        
        // Draw arc segments (180 degrees behind Earth)
        let segments = 20;
        for j in 0..segments {
            let angle_start = std::f32::consts::PI + (j as f32 / segments as f32) * std::f32::consts::PI;
            let angle_end = std::f32::consts::PI + ((j + 1) as f32 / segments as f32) * std::f32::consts::PI;
            
            let start = earth_position + Vec2::from_angle(angle_start) * current_radius;
            let end = earth_position + Vec2::from_angle(angle_end) * current_radius;
            
            let alpha_mult = 1.0 - (i as f32 * 0.3); // Outer arcs are more transparent
            gizmos.line_2d(start, end, color.with_alpha(color.alpha() * alpha_mult));
        }
    }
    
    // Add pulsing energy particles at the bow tips when fully charged
    if charge_percent > 0.9 {
        let pulse = (time.elapsed_secs() * 8.0).sin() * 0.5 + 0.5;
        let particle_color = Color::srgb(1.0, 1.0, 0.3).with_alpha(0.8 * pulse);
        
        // Left tip
        let left_tip = earth_position + Vec2::from_angle(std::f32::consts::PI) * bow_radius;
        let iso_left = Isometry2d::from_translation(left_tip);
        gizmos.circle_2d(iso_left, 1.5 + pulse * 0.5, particle_color);
        
        // Right tip
        let right_tip = earth_position + Vec2::from_angle(0.0) * bow_radius;
        let iso_right = Isometry2d::from_translation(right_tip);
        gizmos.circle_2d(iso_right, 1.5 + pulse * 0.5, particle_color);
    }
}