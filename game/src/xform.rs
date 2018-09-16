use fate::math::{Vec3, Quaternion, Mat4};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Xform {
    pub position: Vec3<f32>,
    pub orientation: Quaternion<f32>,
    pub scale: Vec3<f32>,
}

impl Default for Xform {
    fn default() -> Self {
        Self {
            position: Vec3::zero(),
            orientation: Quaternion::identity(),
            scale: Vec3::one(),
        }
    }
}

impl Xform {
    pub fn forward(&self) -> Vec3<f32> {
        (self.orientation * Vec3::forward_lh()).normalized()
    }
    pub fn up(&self) -> Vec3<f32> {
        (self.orientation * Vec3::up()).normalized()
    }
    pub fn right(&self) -> Vec3<f32> {
        (self.orientation * Vec3::right()).normalized()
    }
    pub fn view_matrix(&self) -> Mat4<f32> {
        self.view_matrix_with_up(self.up())
    }
    pub fn view_matrix_with_up(&self, up: Vec3<f32>) -> Mat4<f32> {
        let zoom = Mat4::<f32>::scaling_3d(self.scale.recip());
        let look = Mat4::look_at(self.position, self.position + self.forward(), up);
        zoom * look
    }
}
