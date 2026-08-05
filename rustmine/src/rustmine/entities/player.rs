use crate::engine::input::controller::{CameraController, ControllerAction};
use crate::engine::rendering::camera::Camera;
use crate::rustmine::generation::types::{BlockType, PlayerPos};
use crate::rustmine::generation::world::World;
use cgmath::Vector3;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

pub(crate) struct Player {
    pub(crate) pos: PlayerPos,
    pub camera: Camera,
    pub(crate) camera_controller: CameraController,
}

impl Player {
    pub(crate) fn new(pos: PlayerPos, camera: Camera, camera_controller: CameraController) -> Self {
        Self {
            pos,
            camera,
            camera_controller,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        self.camera_controller.handle_key(code, is_pressed)
    }

    pub fn handle_mouse_button(&mut self, button: MouseButton, is_pressed: bool) -> bool {
        self.camera_controller
            .handle_mouse_button(button, is_pressed)
    }

    pub fn handle_mouse(&mut self, dx: f64, dy: f64) {
        self.camera_controller.handle_mouse(dx, dy);
    }

    /// Updates camera movement, collisions, and returns any world action (destroy/place)
    pub fn update(&mut self, world: &World, is_generated: bool) -> ControllerAction {
        self.camera_controller.is_generated = is_generated;
        let action = self
            .camera_controller
            .update_camera(&mut self.camera, world);

        self.pos = PlayerPos::new(self.camera.eye.x, self.camera.eye.y, self.camera.eye.z);

        action
    }

    pub fn raycast_block(&self, world: &World) -> Option<(Vector3<i32>, Vector3<i32>)> {
        use cgmath::InnerSpace;

        const MAX_DISTANCE: f32 = 5.0;
        const STEP: f32 = 0.05;

        let origin = self.camera.eye;
        let direction = (self.camera.target - self.camera.eye).normalize();
        let step_vec = direction * STEP;

        let mut point = origin;
        let mut prev_block_pos = Vector3::new(
            origin.x.round() as i32,
            origin.y.round() as i32,
            origin.z.round() as i32,
        );

        let mut traveled = 0.0f32;
        while traveled < MAX_DISTANCE {
            let block_pos = Vector3::new(
                point.x.round() as i32,
                point.y.round() as i32,
                point.z.round() as i32,
            );

            if world.get_block(block_pos).is_some() {
                return Some((block_pos, prev_block_pos));
            }

            prev_block_pos = block_pos;
            point += step_vec;
            traveled += STEP;
        }

        None
    }

    pub fn destroy_targeted_block(&self, world: &mut World) {
        if let Some((pos, _)) = self.raycast_block(world) {
            log::info!("Destroying block at {:?}", pos);
            world.remove_block(pos);
        }
    }

    pub fn create_targeted_block(&self, world: &mut World, block_type: BlockType) {
        if let Some((_, place_pos)) = self.raycast_block(world) {
            if world.get_block(place_pos).is_some()
                || self
                    .camera_controller
                    .block_overlaps_player(self.pos, place_pos)
            {
                return;
            }
            world.set_block(place_pos, block_type);
        }
    }

    // Chat UI functions
    pub fn teleport(&mut self, x: f32, y: f32, z: f32) {
        let new_eye = cgmath::Point3::new(x, y, z);
        let delta = new_eye - self.camera.eye;
        self.camera.eye = new_eye;
        self.camera.target += delta;
        self.pos = PlayerPos::new(x, y, z);
    }
}
