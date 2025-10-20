use bevy::prelude::*;
use crate::GameplaySystem;
use crate::sun_system::{Satellite, Level, SolarSystemAssets, Sun};
use crate::physics::velocity::Velocity;
use crate::physics::calc_gravity::Attractee;
use crate::collision::HitBox;

#[derive(Component, Debug, Copy, Clone)]
pub struct Shields {
    pub current: f32,
    pub max: f32,
    pub regen_per_sec: f32,
}

impl Shields {
    pub fn new(level: f32) -> Self {
        let (max, regen) = match level as i32 {
            1 => (30.0, 5.0),
            2 => (50.0, 8.0),
            3 => (75.0, 12.0),
            _ => (30.0, 5.0),
        };
        Self { current: max, max, regen_per_sec: regen }
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Hull {
    pub hp: f32,
    pub max_hp: f32,
}

impl Hull {
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

#[derive(Component)]
struct ShieldHit {
    elapsed: f32,
    duration: f32,
}

#[derive(Component)]
struct SunExposure {
    time_in_heat: f32,
}

#[derive(Component, Default)]
pub struct DemoteCooldown(pub Timer);

impl DemoteCooldown {
    pub fn new(_level: f32) -> Self {
        Self(Timer::from_seconds(0.0, TimerMode::Once))
    }
}

pub struct DamageSystemShieldsPlugin;

impl Plugin for DamageSystemShieldsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>();
        app.add_systems(Update, (
            apply_sun_radiation_damage,
            handle_damage_events,
            regenerate_shields,
            update_shield_hit_effects,
            draw_shield_effects,
            draw_hull_damage_effects,
        ).in_set(GameplaySystem));
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
    let heat_radius = sun_hitbox.radius * 1.5;
    
    for (entity, sat_transform, exposure_opt) in satellite_query.iter_mut() {
        let sat_pos = sat_transform.translation.truncate();
        let distance = sun_pos.distance(sat_pos);
        
        if distance < heat_radius {
            if let Some(mut exposure) = exposure_opt {
                exposure.time_in_heat += time.delta_secs();
                
                if exposure.time_in_heat >= 1.0 {
                    exposure.time_in_heat = 0.0;
                    commands.trigger(DamageEvent {
                        target: entity,
                        amount: 5.0,
                        source: entity,
                    });
                }
            } else {
                commands.entity(entity).insert(SunExposure { time_in_heat: 0.0 });
            }
        } else {
            if exposure_opt.is_some() {
                commands.entity(entity).remove::<SunExposure>();
            }
        }
    }
}

fn handle_damage_events(
    mut evr: EventReader<DamageEvent>,
    mut q: Query<(&mut Shields, &mut Hull, &mut Level, &mut Sprite, &mut DemoteCooldown, &Transform), With<Attractee>>,
    source_q: Query<&Transform>,
    mut velocity_q: Query<&mut Velocity>,
    assets: Res<SolarSystemAssets>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for ev in evr.read() {
        if let Ok((mut shields, mut hull, mut lvl, mut spr, mut cd, transform)) = q.get_mut(ev.target) {
            // Apply damage to shields first
            let shield_damage = ev.amount.min(shields.current);
            shields.current -= shield_damage;
            
            let remaining_damage = ev.amount - shield_damage;
            
            // Add shield hit effect if shields absorbed damage
            if shield_damage > 0.0 {
                commands.entity(ev.target).insert(ShieldHit {
                    elapsed: 0.0,
                    duration: 0.3,
                });
                
                // Apply knockback if shields absorbed damage
                if let Ok(source_transform) = source_q.get(ev.source) {
                    if let Ok(mut velocity) = velocity_q.get_mut(ev.target) {
                        let direction = (transform.translation - source_transform.translation)
                            .normalize_or_zero()
                            .truncate();
                        let knockback_force = shield_damage * 0.5;
                        velocity.0 += direction * knockback_force;
                    }
                }
            }
            
            // Apply remaining damage to hull
            if remaining_damage > 0.0 {
                cd.0.tick(time.delta());
                if !cd.0.finished() && cd.0.elapsed_secs() < 0.5 {
                    continue;
                }
                
                hull.hp = (hull.hp - remaining_damage).max(0.0);
                let pct = hull.hp / hull.max_hp;
                
                let new_lvl = if pct == 0.0 {
                    0.0
                } else if pct <= 0.33 {
                    1.0
                } else if pct <= 0.66 {
                    2.0
                } else {
                    3.0
                };
                
                if new_lvl != lvl.level {
                    let old_lvl = lvl.level;
                    lvl.level = new_lvl;
                    
                    *spr = match new_lvl as i32 {
                        3 => Sprite::from(assets.collector3.clone()),
                        2 => Sprite::from(assets.collector2.clone()),
                        1 => Sprite::from(assets.collector.clone()),
                        _ => spr.clone(),
                    };
                    
                    cd.0 = Timer::from_seconds(0.5, TimerMode::Once);
                    
                    info!("Satellite demoted from level {} to level {} (Hull: {:.1}/{:.1})", 
                          old_lvl, new_lvl, hull.hp, hull.max_hp);
                }
                
                if hull.hp <= 0.0 {
                    info!("Satellite destroyed");
                    commands.entity(ev.target).despawn_recursive();
                }
            }
        }
    }
}

fn regenerate_shields(
    mut query: Query<&mut Shields, With<Satellite>>,
    time: Res<Time>,
) {
    for mut shields in query.iter_mut() {
        if shields.current < shields.max {
            shields.current = (shields.current + shields.regen_per_sec * time.delta_secs())
                .min(shields.max);
        }
    }
}

fn update_shield_hit_effects(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ShieldHit)>,
    time: Res<Time>,
) {
    for (entity, mut hit) in query.iter_mut() {
        hit.elapsed += time.delta_secs();
        if hit.elapsed >= hit.duration {
            commands.entity(entity).remove::<ShieldHit>();
        }
    }
}

fn draw_shield_effects(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Shields, Option<&ShieldHit>), With<Satellite>>,
    time: Res<Time>,
) {
    for (transform, shields, hit_opt) in query.iter() {
        let position = transform.translation.truncate();
        
        // Draw shield ring
        let shield_pct = shields.current / shields.max;
        let alpha = shield_pct * 0.4;
        let color = Color::srgb(0.3, 0.7, 1.0).with_alpha(alpha);
        let iso = Isometry2d::from_translation(position);
        gizmos.circle_2d(iso, 6.0, color);
        
        // Flicker effect on hit
        if let Some(hit) = hit_opt {
            let flicker = ((hit.elapsed * 20.0).sin() * 0.5 + 0.5);
            let hit_color = Color::srgb(1.0, 1.0, 1.0).with_alpha(flicker * 0.8);
            gizmos.circle_2d(iso, 6.5, hit_color);
        }
    }
}

fn draw_hull_damage_effects(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Hull, &Shields), With<Satellite>>,
) {
    for (transform, hull, shields) in query.iter() {
        // Only show hull damage when shields are down
        if shields.current <= 0.0 {
            let hp_pct = hull.hp / hull.max_hp;
            
            if hp_pct < 0.66 {
                let intensity = 1.0 - hp_pct;
                let color = Color::srgb(1.0, 0.0, 0.0).with_alpha(intensity * 0.5);
                let position = transform.translation.truncate();
                let iso = Isometry2d::from_translation(position);
                gizmos.circle_2d(iso, 5.0, color);
            }
        }
    }
}