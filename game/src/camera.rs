use fate::math::{Mat4, Vec3, Extent2, FrustumPlanes, Vec2, Rect};
use xform::Xform;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum CameraProjectionMode {
    Perspective,
    Ortho,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Camera {
    pub projection_mode: CameraProjectionMode,
    pub fov_y_radians: f32,
    pub near: f32,
    pub far: f32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct View {
    pub xform: Xform,
    pub camera: Camera,
    pub viewport: Rect<u32, u32>,
}

pub fn aspect_ratio(size: Extent2<u32>) -> f32 {
    let Extent2 { w, h } = size;
    assert_ne!(w, 0, "Zero width while computing aspect ratio!");
    assert_ne!(h, 0, "Zero height while computing aspect ratio!");
    w as f32 / h as f32
}

impl View {
    // !!! Must be normalized
    pub fn up_vector_for_lookat(&self) -> Vec3<f32> {
        Vec3::up()
    }
    pub fn aspect_ratio(&self) -> f32 {
        aspect_ratio(self.viewport.extent())
    }
    pub fn ortho_frustum_planes(&self) -> FrustumPlanes<f32> {
        let aspect_ratio = self.aspect_ratio();
        FrustumPlanes {
            right: aspect_ratio,
            left: -aspect_ratio,
            top: 1.,
            bottom: -1.,
            near: self.camera.near,
            far: self.camera.far,
        }
    }
    pub fn view_matrix(&self) -> Mat4<f32> {
        self.xform.view_matrix_with_up(self.up_vector_for_lookat())
    } 
    pub fn proj_matrix(&self) -> Mat4<f32> {
        match self.camera.projection_mode {
            CameraProjectionMode::Perspective => {
                Mat4::perspective_lh_no(self.camera.fov_y_radians, self.aspect_ratio(), self.camera.near, self.camera.far)
            },
            CameraProjectionMode::Ortho => {
                Mat4::orthographic_lh_no(self.ortho_frustum_planes())
            },
        }
    }
    pub fn viewport_to_world(&self, p: Vec2<i32>, z: f32) -> Vec3<f32> {
        let y = self.viewport.h as i32 - p.y;
        let v = Vec3::new(p.x as f32 + 0.5, y as f32 + 0.5, 0.);
        let mut w = Mat4::viewport_to_world_no(v, self.view_matrix(), self.proj_matrix(), self.viewport.map(|p| p as f32, |e| e as f32));
        w.z = z;
        w
    }
    pub fn world_to_viewport(&self, o: Vec3<f32>) -> (Vec2<i32>, f32) {
        let v = Mat4::world_to_viewport_no(o, self.view_matrix(), self.proj_matrix(), self.viewport.map(|p| p as f32, |e| e as f32));
        let (mut z, mut v) = (v.z, Vec2::from(v.map(|x| x.round() as i32)));
        if z.abs() <= 0.0001 {
            z = 0.;
        }
        v.y = self.viewport.h as i32 - v.y;
        (v, z)
    }
    pub fn viewport_to_ugly_ndc(&self, mut p: Vec2<i32>) -> Vec3<f32> {
        let vp_size = self.viewport.extent().map(|x| x as f32);
        p.y = self.viewport.h as i32 - p.y;
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
