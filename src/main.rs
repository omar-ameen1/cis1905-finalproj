mod player_module;
mod input_handler;
mod network_manager;
mod projectile;
mod utilities;
mod barriers;

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_ggrs::*;
use bevy_matchbox::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_roll_safe::prelude::*;

use crate::player_module::*;
use crate::projectile::*;
use crate::barriers::*;
use crate::utilities::*;
use crate::input_handler::*;
use crate::network_manager::*;

/// Configuration for GGRS (Good Game Rollback System)
type GameConfig = GgrsConfig<u32, PeerId>;

/// Holds handles to game textures
#[derive(AssetCollection, Resource)]
struct GameTextures {
    #[asset(path = "bullet.png")]
    projectile_image: Handle<Image>,

    #[asset(path = "barrier.png")]
    barrier_image: Handle<Image>,

    #[asset(path = "gun.png")]
    gun_image: Handle<Image>,
}

/// Represents the different states of the game
#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    Connecting,
    InGame,
}

/// Different phases during gameplay
#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default, Reflect)]
enum GamePhase {
    /// When players are actively playing
    #[default]
    ActiveRound,
    /// After a round ends, transitioning to next
    RoundOver,
}

pub const WORLD_SIZE: u32 = 41;
pub const GRID_LINE_WIDTH: f32 = 0.05;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            player_module::plugin,
            network_manager::plugin,
            GgrsPlugin::<GameConfig>::default(),
        ))
        .init_state::<AppState>()
        .init_resource::<RoundTimer>()
        .init_resource::<MousePosition>()
        .init_ggrs_state::<GamePhase>()
        .add_loading_state(
            LoadingState::new(AppState::Loading)
                .load_collection::<GameTextures>()
                .continue_to_state(AppState::Connecting),
        )
        // Register components and resources for rollback
        .rollback_component_with_clone::<Transform>()
        .rollback_resource_with_clone::<RoundTimer>()
        .rollback_resource_with_clone::<PlayerScores>()
        .rollback_component_with_copy::<CanAttack>()
        .rollback_component_with_copy::<MovementDirection>()
        .rollback_component_with_copy::<Projectile>()
        .rollback_component_with_copy::<Player>()
        .rollback_component_with_copy::<Barrier>()
        // Set the background color
        .insert_resource(ClearColor(Color::srgb(0.53, 0.53, 0.53)))
        // Systems for when entering the Connecting state
        .add_systems(OnEnter(AppState::Connecting), (initialize_game))
        // Systems for when a new round starts
        .add_systems(OnEnter(GamePhase::ActiveRound), create_world)
        // Main game systems scheduled by GGRS
        .add_systems(
            GgrsSchedule,
            (
                player_module::move_players,
                handle_barrier_collisions.after(player_module::move_players),
                projectile_barrier_collisions.after(move_projectile),
                projectile::reload_projectile,
                projectile::fire_projectile
                    .after(player_module::move_players)
                    .after(projectile::reload_projectile)
                    .after(handle_barrier_collisions),
                move_projectile.after(projectile::fire_projectile),
                check_player_collisions
                    .after(move_projectile)
                    .after(player_module::move_players),
            )
                .after(bevy_roll_safe::apply_state_transition::<GamePhase>)
                .run_if(in_state(GamePhase::ActiveRound)),
        )
        // Systems for when the round has ended
        .add_systems(
            GgrsSchedule,
            round_over_timer
                .ambiguous_with(check_player_collisions)
                .run_if(in_state(GamePhase::RoundOver))
                .after(bevy_roll_safe::apply_state_transition::<GamePhase>),
        )
        // Additional game systems
        .add_systems(
            GgrsSchedule,
            place_barrier_on_click.after(move_players),
        )
        .add_systems(
            Update,
            (
                camera_follow.run_if(in_state(AppState::InGame)),
                update_mouse_position.run_if(in_state(AppState::InGame)),
            ),
        )
        .add_systems(ReadInputs, input_handler::collect_player_inputs)
        .run();
}

/// Initializes the game setup
fn initialize_game(mut commands: Commands) {
    // Set up the main camera with fixed vertical scaling
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::FixedVertical(15.0);
    commands.spawn((camera_bundle, Name::new("Main Camera")));

    // Draw horizontal grid lines
    for i in 0..=WORLD_SIZE {
        commands.spawn(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(
                0.,
                i as f32 - WORLD_SIZE as f32 / 2.,
                0.,
            )),
            sprite: Sprite {
                color: Color::srgb(0.27, 0.27, 0.27),
                custom_size: Some(Vec2::new(WORLD_SIZE as f32, GRID_LINE_WIDTH)),
                ..default()
            },
            ..default()
        });
    }
    // Draw vertical grid lines
    for i in 0..=WORLD_SIZE {
        commands.spawn(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(
                i as f32 - WORLD_SIZE as f32 / 2.,
                0.,
                0.,
            )),
            sprite: Sprite {
                color: Color::srgb(0.27, 0.27, 0.27),
                custom_size: Some(Vec2::new(GRID_LINE_WIDTH, WORLD_SIZE as f32)),
                ..default()
            },
            ..default()
        });
    }

    let player_scores = PlayerScores::new();
    commands.insert_resource(player_scores);
}

/// Makes the camera follow the local player_module
fn camera_follow(
    local_players: Res<LocalPlayers>,
    player_query: Query<(&Player, &Transform)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    for (player, player_transform) in &player_query {
        // Only follow the local player_module
        if !local_players.0.contains(&player.handle) {
            continue;
        }

        let position = player_transform.translation;

        for mut transform in &mut camera_query {
            transform.translation.x = position.x;
            transform.translation.y = position.y;
        }
    }
}