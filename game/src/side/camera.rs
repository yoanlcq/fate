
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum CameraProjectionMode {
    Perspective,
    Ortho,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Camera {
    pub position: Vec3<f32>,
    pub target: Vec3<f32>,
    pub scale: Vec3<f32>,
    pub viewport_size: Extent2<u32>,
    pub projection_mode: CameraProjectionMode,
    pub fov_y_radians: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn forward(&self) -> Vec3<f32> {
        (self.target - self.position).normalized()
    }
    pub fn up(&self) -> Vec3<f32> {
        self.forward().cross(self.right())
    }
    pub fn right(&self) -> Vec3<f32> {
        self.up_vector_for_lookat().cross(self.forward())
    }
    // !!! Must be normalized
    pub fn up_vector_for_lookat(&self) -> Vec3<f32> {
        Vec3::up()
    }
    pub fn aspect_ratio(&self) -> f32 {
        let Extent2 { w, h } = self.viewport_size;
        assert_ne!(w, 0);
        assert_ne!(h, 0);
        w as f32 / h as f32
    }
    pub fn ortho_frustum_planes(&self) -> FrustumPlanes<f32> {
        let aspect_ratio = self.aspect_ratio();
        FrustumPlanes {
            right: aspect_ratio,
            left: -aspect_ratio,
            top: 1.,
            bottom: -1.,
            near: self.near,
            far: self.far,
        }
    }
    pub fn proj_matrix(&self) -> Mat4<f32> {
        match self.projection_mode {
            CameraProjectionMode::Perspective => {
                Mat4::perspective_lh_no(self.fov_y_radians, self.aspect_ratio(), self.near, self.far)
            },
            CameraProjectionMode::Ortho => {
                Mat4::orthographic_lh_no(self.ortho_frustum_planes())
            },
        }
    }
    pub fn view_matrix(&self) -> Mat4<f32> {
        let zoom = Mat4::<f32>::scaling_3d(self.scale.recip());
        let look = Mat4::look_at(self.position, self.target, Vec3::up());
        zoom * look
    }
    pub fn viewport(&self) -> Rect<f32, f32> {
        Rect {
            x: 0., // FIXME: Did you just assume the top-left corner???
            y: 0.,
            w: self.viewport_size.w as _,
            h: self.viewport_size.h as _,
        }
    }
    pub fn viewport_to_world(&self, p: Vec2<i32>, z: f32) -> Vec3<f32> {
        let y = self.viewport_size.h as i32 - p.y;
        let v = Vec3::new(p.x as f32 + 0.5, y as f32 + 0.5, 0.);
        let mut w = Mat4::viewport_to_world_no(v, self.view_matrix(), self.proj_matrix(), self.viewport());
        w.z = z;
        w
    }
    pub fn world_to_viewport(&self, o: Vec3<f32>) -> (Vec2<i32>, f32) {
        let v = Mat4::world_to_viewport_no(o, self.view_matrix(), self.proj_matrix(), self.viewport());
        let (mut z, mut v) = (v.z, Vec2::from(v.map(|x| x.round() as i32)));
        if z.abs() <= 0.0001 {
            z = 0.;
        }
        v.y = self.viewport_size.h as i32 - v.y;
        (v, z)
    }
    pub fn viewport_to_ugly_ndc(&self, mut p: Vec2<i32>) -> Vec3<f32> {
        let vp_size = self.viewport_size.map(|x| x as f32);
        p.y = self.viewport_size.h as i32 - p.y;
        let t = p.map(|x| x as f32) / vp_size;
        let t = (t - 0.5) * 2.;
        t.into()
    }
    pub fn viewport_to_pretty_ndc(&self, p: Vec2<i32>) -> Vec3<f32> {
        let FrustumPlanes { left, right, top, bottom, .. } = self.ortho_frustum_planes();
        debug_assert_eq!(left, -right);
        debug_assert_eq!(bottom, -top);
        self.viewport_to_ugly_ndc(p) * Vec3::new(right, top, 0.)
    }
}