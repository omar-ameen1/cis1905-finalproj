use bevy::{prelude::*, utils::HashMap};
use bevy_ggrs::{LocalInputs, LocalPlayers};
use crate::WORLD_SIZE;
use crate::GameConfig;

/// Input flags for player actions
pub(crate) const INPUT_UP: u32 = 1 << 0;
pub(crate) const INPUT_DOWN: u32 = 1 << 1;
pub(crate) const INPUT_LEFT: u32 = 1 << 2;
pub(crate) const INPUT_RIGHT: u32 = 1 << 3;
pub(crate) const INPUT_SHOOT: u32 = 1 << 4;
pub(crate) const INPUT_CLICK: u32 = 1 << 5;

/// Converts an `i32` to `u8`, handling overflows
fn convert_i32_to_u8(value: i32) -> u8 {
    (value as i8) as u8
}

/// Processes and collects inputs from the player
pub(crate) fn collect_player_inputs(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<MousePosition>,
    local_players: Res<LocalPlayers>,
) {
    let mut inputs = HashMap::new();

    for handle in &local_players.0 {
        let mut input_flags = 0u32;

        if mouse_input.pressed(MouseButton::Left) {
            input_flags |= INPUT_CLICK;
            let cursor = cursor_pos.0;
            let cell_x =
                convert_i32_to_u8(cursor.x as i32 + WORLD_SIZE as i32 / 2) as u32;
            let cell_y =
                convert_i32_to_u8(cursor.y as i32 + WORLD_SIZE as i32 / 2) as u32;
            input_flags |= cell_x << 6;
            input_flags |= cell_y << 14;
        } else {
            if keyboard_input.pressed(KeyCode::KeyW) {
                input_flags |= INPUT_UP;
            }

            if keyboard_input.pressed(KeyCode::KeyS) {
                input_flags |= INPUT_DOWN;
            }

            if keyboard_input.pressed(KeyCode::KeyA) {
                input_flags |= INPUT_LEFT;
            }

            if keyboard_input.pressed(KeyCode::KeyD) {
                input_flags |= INPUT_RIGHT;
            }

            if keyboard_input.pressed(KeyCode::Space) {
                input_flags |= INPUT_SHOOT;
            }
        }

        inputs.insert(*handle, input_flags);
    }

    commands.insert_resource(LocalInputs::<GameConfig>(inputs));
}

/// Resource to store the current cursor position
#[derive(Resource)]
pub struct MousePosition(pub(crate) Vec2);

impl Default for MousePosition {
    fn default() -> Self {
        MousePosition(Vec2::ZERO)
    }
}

/// Updates the cursor position based on mouse movement
pub fn update_mouse_position(
    mut cursor_events: EventReader<CursorMoved>,
    mut cursor_position: ResMut<MousePosition>,
    camera_query: Query<(&GlobalTransform, &Camera)>,
) {
    for cursor_moved in cursor_events.read() {
        for (cam_transform, camera) in camera_query.iter() {
            if let Some(world_pos) =
                camera.viewport_to_world_2d(cam_transform, cursor_moved.position)
            {
                *cursor_position = MousePosition(world_pos);
            }
        }
    }
}

/// Calculates the movement direction from input flags
pub fn direction(input: u32) -> Vec2 {
    let mut direction = Vec2::ZERO;

    if input & INPUT_UP != 0 {
        direction.y += 1.0;
    }

    if input & INPUT_DOWN != 0 {
        direction.y -= 1.0;
    }

    if input & INPUT_LEFT != 0 {
        direction.x -= 1.0;
    }

    if input & INPUT_RIGHT != 0 {
        direction.x += 1.0;
    }

    direction.normalize_or_zero()
}

/// Checks if the player is attempting to shoot
pub fn is_shooting(input: u32) -> bool {
    input & INPUT_SHOOT != 0
}

/// Retrieves the mouse click position if applicable
pub fn get_click_position(input: u32) -> Option<(u8, u8)> {
    if input & INPUT_CLICK != 0 {
        let cell_x = (input >> 6) & 0b111111;
        let cell_y = (input >> 14) & 0b111111;
        Some((cell_x as u8, cell_y as u8))
    } else {
        None
    }
}
