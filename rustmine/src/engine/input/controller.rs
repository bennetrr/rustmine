use crate::engine::rendering::camera::Camera;
use crate::rustmine::generation::types::{BlockPos, BlockType, PlayerPos};
use crate::rustmine::generation::world::World;
use cgmath::{InnerSpace, Vector3};
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

const SPEED: f32 = 0.1;
const MOUSE_SENSITIVITY: f32 = 0.4;
const GRAVITY: f32 = -0.013;
const JUMP_FORCE: f32 = 0.22;
const BLOCK_HALF: f32 = 0.5;
const PLAYER_HALF_W: f32 = 0.2;
const PLAYER_HEIGHT: f32 = 1.8;
const TERMINAL_VELOCITY: f32 = -1.5;
const ACCELERATION: f32 = 0.15;
const SOLID_BLOCK_FRICTION: f32 = 0.4;
const AIR_FRICTION: f32 = 0.15;

pub(crate) enum ControllerAction {
    DestroyBlock,
    CreateBlock(BlockType),
    None,
}

pub(crate) struct CameraController {
    speed: f32,
    sensitivity: f32,
    mouse_delta: (f64, f64),
    vertical_velocity: f32,
    is_grounded: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_jump_pressed: bool,
    is_ctrl_pressed: bool,
    is_shift_pressed: bool,
    is_g_pressed: bool,

    is_mouse_left_pressed: bool,
    is_mouse_right_pressed: bool,

    horizontal_velocity: Vector3<f32>,
    jump_cooldown: f32,

    selected_block: BlockType,
    last_update: std::time::Instant,
    pub is_creative: bool,
    pub is_generated: bool,
}

impl CameraController {
    pub(crate) fn new(is_creative: bool) -> Self {
        Self {
            speed: SPEED,
            sensitivity: MOUSE_SENSITIVITY,
            mouse_delta: (0.0, 0.0),
            vertical_velocity: 0.0,
            is_grounded: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_jump_pressed: false,
            is_ctrl_pressed: false,
            is_shift_pressed: false,
            is_g_pressed: false,

            is_mouse_left_pressed: false,
            is_mouse_right_pressed: false,

            horizontal_velocity: Vector3::new(0.0, 0.0, 0.0),
            jump_cooldown: 0.0,

            selected_block: BlockType::Grass,
            last_update: std::time::Instant::now(),
            is_creative,
            is_generated: false,
        }
    }

    pub(crate) fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::Space => {
                self.is_jump_pressed = is_pressed;
                true
            }
            KeyCode::ControlLeft => {
                self.is_ctrl_pressed = is_pressed;
                true
            }
            KeyCode::ShiftLeft => {
                self.is_shift_pressed = is_pressed;
                true
            }
            KeyCode::KeyG => {
                self.is_g_pressed = is_pressed;
                self.is_creative ^= is_pressed;
                true
            }

            KeyCode::Digit1 => {
                if is_pressed {
                    self.selected_block = BlockType::Grass;
                }
                true
            }
            KeyCode::Digit2 => {
                if is_pressed {
                    self.selected_block = BlockType::Dirt;
                }
                true
            }
            KeyCode::Digit3 => {
                if is_pressed {
                    self.selected_block = BlockType::Cobblestone;
                }
                true
            }
            KeyCode::Digit4 => {
                if is_pressed {
                    self.selected_block = BlockType::Oak;
                }
                true
            }
            KeyCode::Digit5 => {
                if is_pressed {
                    self.selected_block = BlockType::Spruce;
                }
                true
            }
            KeyCode::Digit6 => {
                if is_pressed {
                    self.selected_block = BlockType::Birch;
                }
                true
            }
            KeyCode::Digit7 => {
                if is_pressed {
                    self.selected_block = BlockType::LeavesOak;
                }
                true
            }
            KeyCode::Digit8 => {
                if is_pressed {
                    self.selected_block = BlockType::LeavesSpruce;
                }
                true
            }
            KeyCode::Digit9 => {
                if is_pressed {
                    self.selected_block = BlockType::LeavesBirch;
                }
                true
            }
            _ => false,
        }
    }

    pub(crate) fn handle_mouse(&mut self, dx: f64, dy: f64) {
        self.mouse_delta = (dx, dy);
    }

    pub(crate) fn handle_mouse_button(&mut self, button: MouseButton, is_pressed: bool) -> bool {
        match button {
            MouseButton::Left => {
                self.is_mouse_left_pressed = is_pressed;
                true
            }
            MouseButton::Right => {
                self.is_mouse_right_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub(crate) fn update_camera(&mut self, camera: &mut Camera, world: &World) -> ControllerAction {
        // Time
        let now = std::time::Instant::now();
        let mut dt = now.duration_since(self.last_update).as_secs_f32();
        dt = dt.min(1.0 / 30.0);
        self.last_update = now;

        // Mouse
        let (dx, dy) = self.mouse_delta;
        self.mouse_delta = (0.0, 0.0);

        if dx != 0.0 || dy != 0.0 {
            let yaw = cgmath::Rad((-dx as f32) * self.sensitivity * dt);
            let pitch = cgmath::Rad((-dy as f32) * self.sensitivity * dt);

            let forward = (camera.target - camera.eye).normalize();
            let right = forward.cross(camera.up).normalize();

            let yaw_rot = cgmath::Matrix3::from_axis_angle(Vector3::unit_y(), yaw);
            let after_yaw = yaw_rot * forward;

            let pitch_rot = cgmath::Matrix3::from_axis_angle(right, pitch);
            let after_both = (pitch_rot * after_yaw).normalize();

            let new_dot = after_both.dot(Vector3::unit_y()).abs();
            let final_dir = if new_dot < 0.996 {
                after_both
            } else {
                after_yaw.normalize()
            };

            camera.target = camera.eye + final_dir;
        }

        // Horizontal movement
        let forward_3d = (camera.target - camera.eye).normalize();
        let forward_flat = Vector3::new(forward_3d.x, 0.0, forward_3d.z).normalize();
        let right = forward_flat.cross(camera.up).normalize();
        let mut input_direction = Vector3::new(0.0, 0.0, 0.0);

        if self.is_forward_pressed {
            input_direction += forward_flat;
        }
        if self.is_backward_pressed {
            input_direction -= forward_flat;
        }
        if self.is_right_pressed {
            input_direction += right;
        }
        if self.is_left_pressed {
            input_direction -= right;
        }

        let speed_multiplier = if self.is_shift_pressed && !self.is_creative {
            0.7
        } else if self.is_ctrl_pressed && self.is_forward_pressed {
            1.3
        } else {
            1.0
        };

        let target_fovy = if self.is_ctrl_pressed && self.is_forward_pressed {
            90.0
        } else {
            80.0
        };

        let fov_lerp_factor = 1.0 - (1.0 - 10.0 * dt).clamp(0.0, 1.0);
        camera.fovy += (target_fovy - camera.fovy) * fov_lerp_factor;

        let target_velocity = if input_direction.magnitude2() > 0.0 {
            input_direction.normalize() * self.speed * speed_multiplier
        } else {
            Vector3::new(0.0, 0.0, 0.0)
        };

        let smoothing = if input_direction.magnitude2() > 0.0 {
            ACCELERATION
        } else if input_direction.magnitude2() <= 0.0 && self.is_grounded {
            SOLID_BLOCK_FRICTION
        } else {
            AIR_FRICTION
        };
        let lerp_factor = 1.0 - (1.0 - smoothing).powf(dt * 60.0);
        self.horizontal_velocity += (target_velocity - self.horizontal_velocity) * lerp_factor;

        camera.eye += self.horizontal_velocity * dt * 60.0;
        camera.target += self.horizontal_velocity * dt * 60.0;

        if !self.is_grounded && self.is_generated {
            self.vertical_velocity += GRAVITY * dt * 60.0;
            if self.vertical_velocity < TERMINAL_VELOCITY {
                self.vertical_velocity = TERMINAL_VELOCITY;
            }
        }

        // Jump
        if self.is_jump_pressed && self.is_grounded && self.jump_cooldown <= 0.0 {
            self.vertical_velocity = JUMP_FORCE;
            self.is_grounded = false;
            self.jump_cooldown = 0.03;
        }
        if self.jump_cooldown > 0.0 && self.is_grounded {
            self.jump_cooldown -= dt;
        }

        // "Creative"
        if self.is_creative {
            self.vertical_velocity = 0.0;
        }
        if self.is_creative && self.is_jump_pressed {
            self.vertical_velocity = JUMP_FORCE;
            self.is_grounded = false;
        }

        if self.is_creative && self.is_shift_pressed && !self.is_grounded {
            camera.eye.y -= JUMP_FORCE * dt * 60.0;
            camera.target.y -= JUMP_FORCE * dt * 60.0;
        }

        camera.eye.y += self.vertical_velocity * dt * 60.0;
        camera.target.y += self.vertical_velocity * dt * 60.0;

        // Collision
        if self.is_creative {
            self.is_grounded = false;
        } else {
            self.is_grounded = false;
            Self::resolve_collisions(
                camera,
                world,
                &mut self.is_grounded,
                &mut self.vertical_velocity,
            );
        }

        // Destroy block
        if self.is_mouse_left_pressed {
            self.is_mouse_left_pressed = false;
            return ControllerAction::DestroyBlock;
        }

        // Create block
        if self.is_mouse_right_pressed {
            self.is_mouse_right_pressed = false;
            return ControllerAction::CreateBlock(self.selected_block);
        }

        ControllerAction::None
    }

    fn resolve_collisions(
        camera: &mut Camera,
        world: &World,
        is_grounded: &mut bool,
        vertical_velocity: &mut f32,
    ) {
        const PLAYER_HALF_H: f32 = PLAYER_HEIGHT * 0.5;
        const SEARCH_RADIUS: i32 = 2;

        let player_center_x = camera.eye.x;
        let player_center_y = camera.eye.y - PLAYER_HALF_H;
        let player_center_z = camera.eye.z;

        let center_block_x = player_center_x.round() as i32;
        let center_block_y = player_center_y.round() as i32;
        let center_block_z = player_center_z.round() as i32;

        let mut correction_x = 0.0;
        let mut correction_y = 0.0;
        let mut correction_z = 0.0;

        for x in (center_block_x - SEARCH_RADIUS)..=(center_block_x + SEARCH_RADIUS) {
            for y in (center_block_y - SEARCH_RADIUS)..=(center_block_y + SEARCH_RADIUS) {
                for z in (center_block_z - SEARCH_RADIUS)..=(center_block_z + SEARCH_RADIUS) {
                    let pos = Vector3::new(x, y, z);

                    let Some(block) = world.get_block(pos) else {
                        continue;
                    };

                    if *block == BlockType::TallGrass {
                        continue;
                    }

                    let bx = pos.x as f32;
                    let by = pos.y as f32;
                    let bz = pos.z as f32;

                    let cx = player_center_x + correction_x;
                    let cy = player_center_y + correction_y;
                    let cz = player_center_z + correction_z;

                    let dx = cx - bx;
                    let dy = cy - by;
                    let dz = cz - bz;

                    let overlap_x = PLAYER_HALF_W + BLOCK_HALF - dx.abs();
                    let overlap_y = PLAYER_HALF_H + BLOCK_HALF - dy.abs();
                    let overlap_z = PLAYER_HALF_W + BLOCK_HALF - dz.abs();

                    if overlap_x <= 0.0 || overlap_y <= 0.0 || overlap_z <= 0.0 {
                        continue;
                    }

                    if overlap_y <= overlap_x && overlap_y <= overlap_z {
                        if dy > 0.0 {
                            correction_y += overlap_y;
                            *is_grounded = true;
                        } else {
                            correction_y -= overlap_y;
                        }
                        *vertical_velocity = 0.0;
                    } else if overlap_x <= overlap_z {
                        let push = if dx > 0.0 { overlap_x } else { -overlap_x };
                        correction_x += push;
                    } else {
                        let push = if dz > 0.0 { overlap_z } else { -overlap_z };
                        correction_z += push;
                    }
                }
            }
        }

        camera.eye.x += correction_x;
        camera.target.x += correction_x;
        camera.eye.y += correction_y;
        camera.target.y += correction_y;
        camera.eye.z += correction_z;
        camera.target.z += correction_z;
    }

    pub(crate) fn block_overlaps_player(&self, player_pos: PlayerPos, place_pos: BlockPos) -> bool {
        let block_min_x = place_pos.x as f32;
        let block_max_x = block_min_x + 1.0;
        let block_min_y = place_pos.y as f32;
        let block_max_y = block_min_y + 1.0;
        let block_min_z = place_pos.z as f32;
        let block_max_z = block_min_z + 1.0;

        let player_min_x = player_pos.x - PLAYER_HALF_W;
        let player_max_x = player_pos.x + PLAYER_HALF_W;
        let player_min_y = player_pos.y - PLAYER_HEIGHT;
        let player_max_y = player_pos.y;
        let player_min_z = player_pos.z - PLAYER_HALF_W;
        let player_max_z = player_pos.z + PLAYER_HALF_W;

        block_min_x < player_max_x
            && block_max_x > player_min_x
            && block_min_y < player_max_y
            && block_max_y > player_min_y
            && block_min_z < player_max_z
            && block_max_z > player_min_z
    }

    pub(crate) fn reset_input(&mut self) {
        self.is_forward_pressed = false;
        self.is_backward_pressed = false;
        self.is_left_pressed = false;
        self.is_right_pressed = false;
        self.is_jump_pressed = false;
        self.is_ctrl_pressed = false;
        self.is_shift_pressed = false;
        self.is_g_pressed = false;

        self.is_mouse_left_pressed = false;
        self.is_mouse_right_pressed = false;

        // refresh time
        self.last_update = std::time::Instant::now();
        self.mouse_delta = (0.0, 0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Checks whether the key inputs correctly change the internal state
    #[test]
    fn test_handle_key_input() {
        let mut controller = CameraController::new(true);

        controller.handle_key(KeyCode::KeyW, true);
        assert!(controller.is_forward_pressed);

        controller.handle_key(KeyCode::KeyW, false);
        assert!(!controller.is_forward_pressed);
    }

    // Checks the toggling of creative mode
    #[test]
    fn test_creative_mode_toggle() {
        let mut controller = CameraController::new(true);
        assert!(controller.is_creative);

        controller.handle_key(KeyCode::KeyG, true);
        assert!(!controller.is_creative);

        controller.handle_key(KeyCode::KeyG, true);
        assert!(controller.is_creative);
    }

    // Checks the block selection (Digit 1–4)
    #[test]
    fn test_block_selection() {
        let mut controller = CameraController::new(true);

        controller.handle_key(KeyCode::Digit3, true);
        assert_eq!(controller.selected_block, BlockType::Cobblestone);

        controller.handle_key(KeyCode::Digit1, true);
        assert_eq!(controller.selected_block, BlockType::Grass);
    }
}
