#[derive(Debug, Default, Copy, Clone, PartialEq)]
#[repr(C)]
struct Light {
    pub position: Vec4<f32>,
    pub color: Vec4<f32>,
    pub linear: f32,
    pub quadratic: f32,
    pub radius: f32,
    pub padding: f32,
}
