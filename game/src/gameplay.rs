#[derive(Debug)]
pub struct SceneLogicSystem;

impl SceneLogicSystem {
    pub fn new() -> Self {
        SceneLogicSystem
    }
}

impl System for SceneLogicSystem {
    fn on_canvas_resized(&mut self, g: &mut G, size: Extent2<u32>) {
        for camera in g.scene.cameras.values_mut() {
            camera.viewport_size = size;
        }
        g.scene.gui_camera.viewport_size = size;
    }
    fn on_key(&mut self, g: &mut G, key: Key, state: KeyState) {
        match key.sym {
            Some(Keysym::T) if state.is_down() && g.scene.skybox_selector.tab > 0 => g.scene.skybox_selector.tab -= 1,
            Some(Keysym::Y) if state.is_down() => g.scene.skybox_selector.tab += 1,
            Some(Keysym::U) if state.is_down() && g.scene.skybox_selector.layer > 0 => g.scene.skybox_selector.layer -= 1,
            Some(Keysym::I) if state.is_down() => g.scene.skybox_selector.layer += 1,
            Some(Keysym::O) if state.is_down() => g.scene.skybox_min_mag_filter = match g.scene.skybox_min_mag_filter {
                gl::LINEAR => gl::NEAREST,
                gl::NEAREST => gl::LINEAR,
                _ => gl::LINEAR,
            },
            _ => (),
        }
    }
    fn draw(&mut self, g: &mut G, draw: &Draw) {
        for i in g.scene.mesh_instances.values_mut() {
            i.xform.orientation.rotate_x(90_f32.to_radians() * draw.dt);
        }
        for camera in g.scene.cameras.values_mut() {
            // Translate
            let input = g.input.debug_camera_keyboard_dpad();
            let is_freefly = true; // Otherwise, it is "look at target"
            if input != Vec3::zero() { // Testing inequality is fine because it's a D-pad
                let camera_speed = 10.;
                let tx = camera.right() * input.x;
                let ty = camera.up() * input.y;
                let tz = camera.forward() * input.z;
                let t = (tx + ty + tz) * camera_speed * draw.dt;
                if is_freefly {
                    camera.position += t;
                    camera.target += t;
                }
            }

            // Rotate
            let disp = g.input.mouse_displacement();
            if g.input.mouse_button(MouseButton::Left).is_down() && disp != Vec2::zero() {

                let degrees_per_screen_width = 180_f32 * 4.;
                let disp = disp.map(|x| (x * x * x.signum()) as f32) * degrees_per_screen_width.to_radians() / camera.viewport_size.w as f32;

                let mut self_to_target = camera.target - camera.position;
                let rx = Quaternion::rotation_3d(disp.y, camera.right());
                let ry = Quaternion::rotation_3d(disp.x, camera.up());
                self_to_target = rx * ry * self_to_target;
                if is_freefly {
                    camera.target = camera.position + self_to_target;
                } else {
                    camera.position = camera.target - self_to_target;
                }
            }
        }
    }
}