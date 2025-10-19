use bevy::audio::Volume;
use bevy::prelude::*;
use crate::collision::FatalCollisionEvent;
use crate::screens::Screen;
use crate::sun_system::{SolarSystemAssets, Sun, Satellite};
use crate::sun_system::asteroids::AsteroidSwarm;
use crate::sun_system::navigation_instruments::NavigationInstruments;

pub(crate) struct SoundPlugin;

// Tag for the looping background music entity
#[derive(Component)]
pub(crate) struct Music;

// Tag for radio comms sound
#[derive(Component)]
struct RadioCommsPlaying;

// Event to toggle music on/off
#[derive(Message, Default)]
pub(crate) struct ToggleMusic;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<ToggleMusic>()
            .add_systems(OnEnter(Screen::Gameplay), setup_sound)
            .add_systems(Update, toggle_music.run_if(resource_exists::<SolarSystemAssets>))
            .add_systems(Update, play_radio_comms_on_hover)
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
    sun_query: Query<(), With<Sun>>,
) {
    if let Ok(asteroid_swarm_entity) = asteroid_swarm_query.single() {
        if event.destroyed == asteroid_swarm_entity {
            return;
        }
    }
    // Mute crash SFX when swallowed by the sun
    if sun_query.get(event.other).is_ok() {
        return;
    }
    commands.spawn((
        AudioPlayer::new(solar_system_assets.crash_sound.clone()),
        PlaybackSettings::DESPAWN,
    ));

}

fn play_radio_comms_on_hover(
    mut commands: Commands,
    satellite_query: Query<Entity, (With<Satellite>, With<NavigationInstruments>, Changed<NavigationInstruments>)>,
    radio_query: Query<Entity, With<RadioCommsPlaying>>,
    assets: Res<SolarSystemAssets>,
) {
    // Check if a satellite was just selected (NavigationInstruments was just added)
    for _entity in satellite_query.iter() {
        // Stop any currently playing radio comms
        for radio_entity in radio_query.iter() {
            commands.entity(radio_entity).despawn();
        }
        
        // Play Japanese radio comms sound (using warning_sound as placeholder)
        // TODO: Replace with actual Japanese radio comms audio file
        commands.spawn((
            AudioPlayer::new(assets.warning_sound.clone()),
            PlaybackSettings::DESPAWN,
            RadioCommsPlaying,
        ));
        
        // Only play once per selection
        break;
    }
}