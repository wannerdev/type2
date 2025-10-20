use bevy::prelude::*;
use crate::GameplaySystem;
use crate::sun_system::navigation_instruments::NavigationInstruments;
use crate::sun_system::Satellite;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, draw_selection_glow.in_set(GameplaySystem));
}

fn draw_selection_glow(
    mut gizmos: Gizmos,
    query: Query<&GlobalTransform, (With<Satellite>, With<NavigationInstruments>)>,
    time: Res<Time>,
) {
    for transform in query.iter() {
        let position = transform.translation().truncate();
        
        // Pulsing glow effect
        let pulse = (time.elapsed_secs() * 3.0).sin() * 0.5 + 0.5; // 0.0 to 1.0
        let radius = 6.0 + pulse * 2.0; // 6.0 to 8.0
        let alpha = 0.6 + pulse * 0.4; // 0.6 to 1.0
        
        // Cyan/blue glow for selected satellites
        let color = Color::srgb(0.2, 0.8, 1.0).with_alpha(alpha);
        
        let iso = Isometry2d::from_translation(position);
        gizmos.circle_2d(iso, radius, color);
    }
}