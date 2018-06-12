use std::mem;
use std::ptr;
use std::ffi::CString;
use std::ops::Range;
use super::ProgramAttribs;
use gl::{self, types::*};
use gx::{self, Object};
use grx;

#[derive(Debug)]
pub struct VertexArray<Program: ProgramAttribs> {
    buffer_usage: gx::BufferUsage,
    pub vertices: Vec<Program::Vertex>,
    vbo: gx::Buffer,
    vao: gx::VertexArray,
}

impl<Program: ProgramAttribs> VertexArray<Program> {
    pub fn vao(&self) -> &gx::VertexArray { &self.vao }
    pub fn vbo(&self) -> &gx::Buffer { &self.vbo }
    pub fn update_vbo_range(&self, range: Range<usize>) {
        assert!(range.start <= range.end);
        assert!(range.end <= self.vertices.capacity());
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo.gl_id());
            // NOTE: Be careful not to actually index the Vec with range.
            // Will cause panics because we care about the capacity, not the length!
            let offset = range.start * mem::size_of::<Program::Vertex>();
            let size = (range.end - range.start) * mem::size_of::<Program::Vertex>();
            let data = self.vertices.as_ptr().add(range.start);
            gl::BufferSubData(gl::ARRAY_BUFFER, offset as _, size as _, data as _);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
    pub fn update_and_resize_vbo(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo.gl_id());
            gl::BufferData(gl::ARRAY_BUFFER, (self.vertices.capacity() * mem::size_of::<Program::Vertex>()) as _, self.vertices.as_ptr() as _, self.buffer_usage as _);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
    pub fn from_vertices(
        prog: &Program,
        label: &str,
        buffer_usage: gx::BufferUsage,
        vertices: Vec<Program::Vertex>
    ) -> Self
    {
        let vao = gx::VertexArray::new();
        let vbo = gx::Buffer::new();
        unsafe {
            gl::BindVertexArray(vao.gl_id());
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo.gl_id());
            grx::set_label(&vao, &CString::new(label.to_owned() + " VAO").unwrap().into_bytes_with_nul());
            grx::set_label(&vbo, &CString::new(label.to_owned() + " VBO").unwrap().into_bytes_with_nul());

            for attrib in prog.attribs() {
                gl::EnableVertexAttribArray(attrib.location);
                gl::VertexAttribPointer(
                    attrib.location,
                    attrib.nb_components as _,
                    attrib.gl_type,
                    attrib.normalize as GLboolean,
                    attrib.stride as _,
                    ptr::null_mut::<GLvoid>().add(attrib.offset)
                );
            }

            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        let array = Self {
            vertices, vbo, vao, buffer_usage,
        };
        array.update_and_resize_vbo();
        array
    }
}

