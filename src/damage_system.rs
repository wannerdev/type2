use bevy::prelude::*;
use crate::GameplaySystem;
use crate::sun_system::{Satellite, Level, SolarSystemAssets, Sun};
use crate::physics::velocity::Velocity;
use crate::physics::calc_gravity::Attractee;
use crate::collision::HitBox;

#[derive(Component, Debug, Copy, Clone)]
pub struct Health {
    pub hp: f32,
    pub max_hp: f32,
}

impl Health {
    pub fn new(level: f32) -> Self {
        let max_hp = match level as i32 {
            1 => 50.0,
            2 => 85.0,
            3 => 120.0,
            _ => 50.0,
        };
        Self { hp: max_hp, max_hp }
    }
}

#[derive(Event)]
pub struct DamageEvent {
    pub target: Entity,
    pub amount: f32,
    pub source: Entity,
}

#[derive(Component, Default)]
pub struct DemoteCooldown(pub Timer);

impl DemoteCooldown {
    pub fn new(_level: f32) -> Self {
        Self(Timer::from_seconds(0.0, TimerMode::Once))
    }
}

#[derive(Component)]
struct SunExposure {
    time_in_heat: f32,
}

pub struct DamageSystemPlugin;

impl Plugin for DamageSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            apply_sun_radiation_damage,
            draw_damage_effects,
        ).in_set(GameplaySystem));
        app.add_observer(handle_damage_events);
    }
}

fn apply_sun_radiation_damage(
    mut commands: Commands,
    sun_query: Query<(&Transform, &HitBox), With<Sun>>,
    mut satellite_query: Query<(Entity, &Transform, Option<&mut SunExposure>), With<Satellite>>,
    time: Res<Time>,
) {
    let Ok((sun_transform, sun_hitbox)) = sun_query.single() else { return; };
    let sun_pos = sun_transform.translation.truncate();
    let heat_radius = sun_hitbox.radius * 1.5; // 50% larger than sun
    
    for (entity, sat_transform, exposure_opt) in satellite_query.iter_mut() {
        let sat_pos = sat_transform.translation.truncate();
        let distance = sun_pos.distance(sat_pos);
        
        if distance < heat_radius {
            // Inside heat zone
            if let Some(mut exposure) = exposure_opt {
                exposure.time_in_heat += time.delta_secs();
                
                // Apply damage every second
                if exposure.time_in_heat >= 1.0 {
                    exposure.time_in_heat = 0.0;
                    
                    commands.trigger(DamageEvent {
                        target: entity,
                        amount: 5.0,
                        source: entity, // Self-damage from sun
                    });
                }
            } else {
                commands.entity(entity).insert(SunExposure { time_in_heat: 0.0 });
            }
        } else {
            // Outside heat zone - remove exposure component
            if exposure_opt.is_some() {
                commands.entity(entity).remove::<SunExposure>();
            }
        }
    }
}

fn handle_damage_events(
    ev: On<DamageEvent>,
    mut q: Query<(&mut Health, &mut Level, &mut Sprite, &mut DemoteCooldown), With<Attractee>>,
    assets: Res<SolarSystemAssets>,
    time: Res<Time>,
    mut commands: Commands,
) {
    if let Ok((mut hp, mut lvl, mut spr, mut cd)) = q.get_mut(ev.target) {
        // Check i-frame
        cd.0.tick(time.delta());
        if !cd.0.finished() && cd.0.elapsed_secs() < 0.5 {
            return; // Still in i-frame
        }

        // Apply damage
        hp.hp = (hp.hp - ev.amount).max(0.0);
        let pct = hp.hp / hp.max_hp;

        // Calculate new level based on HP percentage
        let new_lvl = if pct == 0.0 {
            0.0
        } else if pct <= 0.33 {
            1.0
        } else if pct <= 0.66 {
            2.0
        } else {
            3.0
        };

        // Handle level change
        if new_lvl != lvl.level {
            let old_lvl = lvl.level;
            lvl.level = new_lvl;

            // Update sprite
            *spr = match new_lvl as i32 {
                3 => Sprite::from(assets.collector3.clone()),
                2 => Sprite::from(assets.collector2.clone()),
                1 => Sprite::from(assets.collector.clone()),
                _ => spr.clone(),
            };

            // Start i-frame cooldown
            cd.0 = Timer::from_seconds(0.5, TimerMode::Once);

            info!(
                "Satellite demoted from level {} to level {} (HP: {:.1}/{:.1})",
                old_lvl, new_lvl, hp.hp, hp.max_hp
            );
        }

        // Destroy if HP reaches 0
        if hp.hp <= 0.0 {
            info!("Satellite destroyed by damage");
            if let Ok(mut ec) = commands.get_entity(ev.target) {
                ec.despawn();
            }
        }
    }
}

fn draw_damage_effects(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Health), With<Satellite>>,
) {
    for (transform, health) in query.iter() {
        let hp_pct = health.hp / health.max_hp;
        
        // Draw damage indicator (red glow intensity based on damage)
        if hp_pct < 0.66 {
            let intensity = 1.0 - hp_pct;
            let color = Color::srgb(1.0, 0.0, 0.0).with_alpha(intensity * 0.5);
            let position = transform.translation.truncate();
            let iso = Isometry2d::from_translation(position);
            gizmos.circle_2d(iso, 5.0, color);
        }
    }
}