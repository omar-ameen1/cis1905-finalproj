use bevy::prelude::*;
use bevy_ggrs::{AddRollbackCommandExtension, PlayerInputs};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::barriers::create_world;
use crate::input_handler::direction;
use crate::network_manager::*;
use crate::projectile::Projectile;
use crate::{GameConfig, GamePhase, WORLD_SIZE, GameTextures};
use crate::utilities::PlayerScores;

pub const PLAYER_RADIUS: f32 = 0.5;
pub const PROJECTILE_RADIUS: f32 = 0.025;

/// Registers the player module systems to the app
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GamePhase::ActiveRound),
        initialize_players.after(create_world),
    );
}

/// Component representing a player entity
#[derive(Component, Clone, Copy)]
pub struct Player {
    pub(crate) speed: f32,
    pub(crate) handle: usize,
    pub(crate) color: Color,
}

/// Component indicating if the player can attack
#[derive(Component, Clone, Copy)]
pub struct CanAttack(pub bool);

/// Component for storing movement direction
#[derive(Component, Clone, Copy)]
pub struct MovementDirection(pub Vec2);

/// Component representing a gun entity
#[derive(Component)]
pub struct Gun;

/// Initializes players at the start of a new round
fn initialize_players(
    mut commands: Commands,
    existing_players: Query<(Entity, &Transform), With<Player>>,
    existing_projectiles: Query<Entity, With<Projectile>>,
    random_seed: Res<RandomSeed>,
    player_scores: Res<PlayerScores>,
    game_textures: Res<GameTextures>,
) {
    // Sum up the x positions of all existing players
    let total_x: f32 = existing_players
        .iter()
        .map(|(_, transform)| transform.translation.x)
        .sum();

    // Despawn all existing players
    for (entity, _) in existing_players.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Despawn all existing projectiles
    for entity in existing_projectiles.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Initialize RNG with a combined seed
    let seed_value = (1234u64 + total_x as u64 + player_scores.get(0) +
        player_scores.get(1)) ^ **random_seed;
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed_value);

    let half_world_size = WORLD_SIZE as f32 * 0.5;

    // Generate random positions for the players
    let mut player_positions = Vec::with_capacity(NUM_PLAYERS);
    for _ in 0..NUM_PLAYERS {
        player_positions.push(Vec2::new(
            rng.gen_range(-half_world_size..half_world_size),
            rng.gen_range(-half_world_size..half_world_size),
        ));
    }

    // Spawn in players
    for i in 0..NUM_PLAYERS {
        let color = Color::srgb(
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
        );

        let initial_direction = Vec2::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        ).normalize_or_zero();

        create_player(
            &mut commands,
            player_positions[i],
            i,
            color,
            initial_direction,
            game_textures.gun_image.clone(),
        );
    }
}

/// Helper function to create a player entity
fn create_player(
    commands: &mut Commands,
    position: Vec2,
    handle: usize,
    color: Color,
    initial_direction: Vec2,
    gun_image: Handle<Image>,
) {
    let player_entity = commands
        .spawn((
            Player {
                speed: 10.0,
                handle,
                color
            },
            CanAttack(true),
            MovementDirection(initial_direction),
            SpriteBundle {
                transform: Transform::from_translation(position.extend(100.0)),
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(1.0, 1.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .add_rollback()
        .id();

    // Spawn the gun as a child of the player
    commands.entity(player_entity).with_children(|parent| {
        parent.spawn((
            Gun,
            SpriteBundle {
                texture: gun_image,
                transform: Transform {
                    translation: Vec3::new(0.5, 0.0, 1.0), // Offset to position in front of the player
                    rotation: Quat::from_rotation_z(initial_direction.y.atan2(initial_direction.x)),
                    scale: Vec3::new(0.005, 0.005, 1.0),
                },
                ..Default::default()
            },
        ));
    });
}

/// Moves players based on their input and updates their position
pub fn move_players(
    mut player_query: Query<(&mut Transform, &mut MovementDirection, &Player), With<Player>>,
    mut gun_query: Query<&mut Transform, (With<Gun>, Without<Player>)>,
    inputs: Res<PlayerInputs<GameConfig>>,
    time: Res<Time>,
) {
    for (mut transform, mut movement_direction, player) in &mut player_query {
        let (input_bits, _) = inputs[player.handle];

        let direction_vector = direction(input_bits);

        if direction_vector == Vec2::ZERO {
            continue;
        }

        movement_direction.0 = direction_vector;

        let movement_delta = direction_vector * player.speed * time.delta_seconds();

        let current_position = transform.translation.xy();
        let boundary_limit = Vec2::splat(WORLD_SIZE as f32 * 0.5 - 0.5);
        let new_position = (current_position + movement_delta)
            .clamp(-boundary_limit, boundary_limit);

        transform.translation.x = new_position.x;
        transform.translation.y = new_position.y;

        // Update gun position and rotation
        for mut gun_transform in &mut gun_query {
            gun_transform.translation = Vec3::new(
                direction_vector.x * 0.5,
                direction_vector.y * 0.5,
                gun_transform.translation.z,
            );
            gun_transform.rotation =
                Quat::from_rotation_z(direction_vector.y.atan2(direction_vector.x));
        }
    }
}

/// Checks for collisions between players and projectiles
pub fn check_player_collisions(
    mut commands: Commands,
    player_query: Query<(Entity, &Transform, &Player), (With<Player>, Without<Projectile>)>,
    projectile_query: Query<&Transform, With<Projectile>>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut playerscores: ResMut<PlayerScores>
) {
    for (player_entity, player_transform, player) in &player_query {
        let player_pos = player_transform.translation.xy();
        for projectile_transform in &projectile_query {
            let projectile_pos = projectile_transform.translation.xy();
            if is_colliding(player_pos, projectile_pos, PLAYER_RADIUS, PROJECTILE_RADIUS) {
                commands.entity(player_entity).despawn_recursive();
                println!("Player killed!");
                if NUM_PLAYERS > 2 {
                    if player_query.iter().count() == 1 {
                        next_state.set(GamePhase::RoundOver);
                        break;
                    }
                } else {
                    next_state.set(GamePhase::RoundOver);
                    if player.handle == 0 {
                        let score = playerscores.get(1);
                        playerscores.set(1, score + 1);
                    } else {
                        let score = playerscores.get(0);
                        playerscores.set(0, score + 1);
                    }
                    break;
                }
            }
        }
    }
}


/// Determines if two circles are colliding
fn is_colliding(pos1: Vec2, pos2: Vec2, radius1: f32, radius2: f32) -> bool {
    Vec2::distance(pos1, pos2) < radius1 + radius2
}
