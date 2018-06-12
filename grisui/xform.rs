use v::{self, Vec2, Vec3, Mat4};

pub type Xform3D = v::Transform<f32, f32, f32>;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Xform2D {
    pub position: Vec3<f32>,
    pub rotation_z_radians: f32,
    pub scale: Vec2<f32>,
}

impl Default for Xform2D {
    fn default() -> Self {
        Self {
            position: Vec3::zero(),
            rotation_z_radians: 0.,
            scale: Vec2::one(),
        }
    }
}

impl Xform2D {
    pub fn up(&self) -> Vec2<f32> {
        Vec2::unit_y().rotated_z(self.rotation_z_radians)
    }
    pub fn down(&self) -> Vec2<f32> {
        -self.up()
    }
    pub fn right(&self) -> Vec2<f32> {
        self.up().rotated_z(-90_f32.to_radians())
    }
    pub fn left(&self) -> Vec2<f32> {
        -self.right()
    }
    pub fn forward(&self) -> Vec3<f32> {
        Vec3::unit_z()
    }
    pub fn back(&self) -> Vec3<f32> {
        -self.forward()
    }
    pub fn model_matrix(&self) -> Mat4<f32> {
        Mat4::scaling_3d(self.scale).rotated_z(self.rotation_z_radians).translated_3d(self.position)
    }
}

