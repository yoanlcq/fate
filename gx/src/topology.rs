use gl;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(u32)]
pub enum Topology {
     Points                 = gl::POINTS,
     LineStrip              = gl::LINE_STRIP,
     LineLoop               = gl::LINE_LOOP,
     Lines                  = gl::LINES,
     LineStripAdjacency     = gl::LINE_STRIP_ADJACENCY,
     LinesAdjacency         = gl::LINES_ADJACENCY,
     TriangleStrip          = gl::TRIANGLE_STRIP,
     TriangleFan            = gl::TRIANGLE_FAN,
     Triangles              = gl::TRIANGLES,
     TriangleStripAdjacency = gl::TRIANGLE_STRIP_ADJACENCY,
     TrianglesAdjacency     = gl::TRIANGLES_ADJACENCY,
     Patches                = gl::PATCHES,
}