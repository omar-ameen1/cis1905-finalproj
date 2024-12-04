use bevy::prelude::*;
use bevy_ggrs::prelude::*;
use crate::{GameConfig, GameTextures};
use crate::input_handler::is_shooting;
use crate::player_module::{CanAttack, MovementDirection, Player, PROJECTILE_RADIUS, PLAYER_RADIUS};

#[derive(Component, Clone, Copy)]
pub struct Projectile;

pub fn fire_projectile(
    mut commands: Commands,
    inputs: Res<PlayerInputs<GameConfig>>,
    images: Res<GameTextures>,
    mut players: Query<(&Transform, &Player, &mut CanAttack, &MovementDirection)>
) {
    for (transform, player, mut attack_ready, movement_direction) in &mut players {
        let (input, _) = inputs[player.handle];
        if is_shooting(input) && attack_ready.0 {
            let player_pos = transform.translation.xy();
            let pos = player_pos + movement_direction.0 * PLAYER_RADIUS + PROJECTILE_RADIUS;
            commands
                .spawn((
                    Projectile,
                    *movement_direction,
                SpriteBundle {
                    transform: Transform::from_translation(pos.extend(200.0)),
                    texture: images.projectile_image.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(1.0, 1.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                )).add_rollback();
            attack_ready.0 = false;
        }
    }
}

pub fn reload_projectile(
    inputs: Res<PlayerInputs<GameConfig>>,
    mut players: Query<(&mut CanAttack, &Player)>
) {
    for (mut can_attack, player) in players.iter_mut() {
        let (input, _) = inputs[player.handle];
        if !is_shooting(input) {
            can_attack.0 = true;
        }
    }
}

pub fn move_projectile(
    mut projectiles: Query<(&mut Transform, &MovementDirection), With<Projectile>>,
    time: Res<Time>
) {
    for (mut transform, move_dir) in &mut projectiles {
        let speed = 20.0;
        let delta = move_dir.0 * speed * time.delta_seconds();
        transform.translation += delta.extend(0.0);
    }
}