use bevy::audio::Volume;
use bevy::prelude::*;
use crate::collision::FatalCollisionEvent;
use crate::screens::Screen;
use crate::sun_system::{SolarSystemAssets, Sun, Satellite};
use crate::sun_system::asteroids::{AsteroidSwarm, Asteroid};

pub(crate) struct SoundPlugin;

// Tag for the looping background music entity
#[derive(Component)]
pub(crate) struct Music;

// Event to toggle music on/off
#[derive(Message, Default)]
pub(crate) struct ToggleMusic;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<ToggleMusic>()
            .add_systems(OnEnter(Screen::Gameplay), setup_sound)
            .add_systems(Update, toggle_music.run_if(resource_exists::<SolarSystemAssets>))
            .add_observer(handle_fatal_collision_event_for_sound)
            .insert_resource(GlobalVolume::new(Volume::Linear(0.1)));
    }
}

fn setup_sound(mut commands: Commands, solar_system_assets: Res<SolarSystemAssets>) {
    commands.spawn((
        AudioPlayer::new(solar_system_assets.music_loop.clone()),
        PlaybackSettings::LOOP,
        Music,
    ));
}

// Toggle by despawning/respawning the Music entity
fn toggle_music(
    mut commands: Commands,
    mut ev_toggle: MessageReader<ToggleMusic>,
    music_q: Query<Entity, With<Music>>,
    assets: Res<SolarSystemAssets>,
) {
    let mut toggled = false;
    for _ in ev_toggle.read() { toggled = true; break; }
    if !toggled { return; }

    if let Ok(e) = music_q.single() {
        commands.entity(e).despawn();
    } else {
        commands.spawn((
            AudioPlayer::new(assets.music_loop.clone()),
            PlaybackSettings::LOOP,
            Music,
        ));
    }
}

fn handle_fatal_collision_event_for_sound(
    event: On<FatalCollisionEvent>,
    mut commands: Commands,
    solar_system_assets: Res<SolarSystemAssets>,
    asteroid_swarm_query: Query<Entity, With<AsteroidSwarm>>,
    asteroid_query: Query<(), With<Asteroid>>,
    sun_query: Query<(), With<Sun>>,
    satellite_query: Query<(), With<Satellite>>,
) {
    // Skip sound for asteroid swarm parent entity (only play for individual asteroids)
    if let Ok(asteroid_swarm_entity) = asteroid_swarm_query.single() {
        if event.destroyed == asteroid_swarm_entity {
            return;
        }
    }
    
    // Check if collision involves the sun
    let is_sun_collision = sun_query.get(event.other).is_ok();
    
    if is_sun_collision {
        // Check if a satellite crashed into the sun (splash sound)
        if satellite_query.get(event.destroyed).is_ok() {
            // Play splash sound for satellite entering sun
            // Using warning sound as placeholder for splash effect
            commands.spawn((
                AudioPlayer::new(solar_system_assets.warning_sound.clone()),
                PlaybackSettings::DESPAWN,
            ));
            info!("Satellite splash into sun!");
        }
        // For asteroids into sun, play no sound (they just disappear)
        return;
    }
    
    // For all other collisions (satellite-satellite, satellite-asteroid), play crash sound
    commands.spawn((
        AudioPlayer::new(solar_system_assets.crash_sound.clone()),
        PlaybackSettings::DESPAWN,
    ));
}