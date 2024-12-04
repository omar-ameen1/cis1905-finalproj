use bevy::prelude::*;
use bevy_ggrs::{AddRollbackCommandExtension, PlayerInputs};
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use rand::{Rng};
use crate::{WORLD_SIZE, GameTextures};
use crate::network_manager::{RandomSeed};
use crate::GameConfig;
use crate::player_module::{Player, PLAYER_RADIUS};
use crate::projectile::Projectile;
use crate::input_handler::*;
use crate::utilities::PlayerScores;

#[derive(Component, Clone, Copy)]
pub struct Barrier {
    pub(crate) player_placed: bool,
}

pub fn create_world(
    mut commands: Commands,
    barriers: Query<Entity, With<Barrier>>,
    session_seed: Res<RandomSeed>,
    images: Res<GameTextures>,
    playerscores: Res<PlayerScores>
) {
    // Clear existing barriers
    for barrier in &barriers {
        commands.entity(barrier).despawn_recursive();
    }

    let mut rng = Xoshiro256PlusPlus::seed_from_u64((1234u64 + playerscores.get(0) + playerscores.get(1)) ^ **session_seed);

    // Generate walls
    for _ in 0..20 {
        let max_box_size = WORLD_SIZE / 4;
        let width = rng.gen_range(1..max_box_size);
        let height = rng.gen_range(1..max_box_size);

        let cell_x = rng.gen_range(0..=(WORLD_SIZE - width));
        let cell_y = rng.gen_range(0..=(WORLD_SIZE - height));

        for dx in 0..width {
            for dy in 0..height {
                // Calculate the position of each tile in the grid
                let tile_x = cell_x + dx;
                let tile_y = cell_y + dy;

                // Convert the grid position to world position
                let world_pos = Vec3::new(
                    tile_x as f32 + 0.5 - WORLD_SIZE as f32 / 2.,
                    tile_y as f32 + 0.5 - WORLD_SIZE as f32 / 2.,
                    10.,
                );

                commands.spawn((
                    Barrier {
                        player_placed: false,
                    },
                    SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(Vec2::ONE), // Each tile is 1x1
                            ..default()
                        },
                        texture: images.barrier_image.clone(),
                        transform: Transform::from_translation(world_pos),
                        ..default()
                    },
                ));
            }
        }
    }
}

pub fn handle_barrier_collisions(
    mut players: Query<&mut Transform, With<Player>>,
    barriers: Query<(&Transform, &Sprite), (With<Barrier>, Without<Player>)>,
) {
    for mut player_transform in &mut players {
        for (barrier_transform, barrier_sprite) in &barriers {
            let barrier_size = barrier_sprite.custom_size.expect("Barrier has no size");
            let barrier_pos = barrier_transform.translation.xy();
            let player_pos = player_transform.translation.xy();

            let barrier_to_player = player_pos - barrier_pos;

            let barrier_corner_to_player = barrier_to_player.abs() - barrier_size / 2.;

            let corner_to_corner = barrier_corner_to_player - Vec2::splat(PLAYER_RADIUS);

            if corner_to_corner.x > 0. || corner_to_corner.y > 0. {
                continue;
            }

            if corner_to_corner.x > corner_to_corner.y {
                player_transform.translation.x -= barrier_to_player.x.signum() * corner_to_corner.x;
            } else {
                player_transform.translation.y -= barrier_to_player.y.signum() * corner_to_corner.y;
            }
        }
    }
}

pub fn projectile_barrier_collisions(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform), With<Projectile>>,
    barriers: Query<(Entity, &Barrier, &Transform, &Sprite), (With<Barrier>, Without<Projectile>)>,
) {
    let half_map_limit = WORLD_SIZE as f32 * 0.5;

    for (proj_entity, proj_transform) in projectiles.iter() {
        let proj_pos = proj_transform.translation.xy();

        // Remove projectile if it's beyond the map boundaries
        if proj_pos.x.abs() > half_map_limit || proj_pos.y.abs() > half_map_limit {
            commands.entity(proj_entity).despawn_recursive();
            continue;
        }

        // Check collision with barriers
        for (bar_entity, barrier_comp, bar_transform, bar_sprite) in barriers.iter() {
            let Some(bar_size) = bar_sprite.custom_size else {
                panic!("Barrier is missing size information");
            };
            let bar_pos = bar_transform.translation.xy();

            // Calculate the distance between projectile and barrier centers
            let delta = proj_pos - bar_pos;
            let abs_delta = delta.abs();

            // Determine overlap by subtracting half the barrier size
            let overlap = abs_delta - (bar_size * 0.5);

            // Check if projectile is inside the barrier
            if overlap.x <= 0.0 && overlap.y <= 0.0 {
                // Despawn the barrier if it was placed by a player
                if barrier_comp.player_placed {
                    commands.entity(bar_entity).despawn_recursive();
                }
                // Despawn the projectile upon collision
                commands.entity(proj_entity).despawn_recursive();
                break; // No need to check other barriers
            }
        }
    }
}

pub fn place_barrier_on_click(
    mut commands: Commands,
    inputs: Res<PlayerInputs<GameConfig>>,
    players: Query<&Player>,
) {
    for player in &players {
        let (input, _) = inputs[player.handle];
        // check_mouse_click returns Some((cell_x as u8, cell_y as u8))
        if let Some((cell_x, cell_y)) = get_click_position(input) {
            let size = Vec2::new(1., 1.);

            commands.spawn((
                Barrier {
                    player_placed: true,
                },
                SpriteBundle {
                    sprite: Sprite {
                        color: player.color,
                        custom_size: Some(size),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        cell_x as f32 - WORLD_SIZE as f32 / 2. + size.x / 2.,
                        cell_y as f32 - WORLD_SIZE as f32 / 2. + size.y / 2.,
                        10.,
                    )),
                    ..default()
                },
            )).add_rollback();
        }
    }
}