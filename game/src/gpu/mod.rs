pub mod cmd;
pub use self::cmd::GpuCmd;
pub mod end_frame;
pub use self::end_frame::GpuEndFrame;
pub mod img;
pub use self::img::{GpuTextureInternalFormat, CpuImgPixelType, CpuImgFormat, CpuSubImage3D, CpuSubImage2D};


pub fn into_bytes_vec<T>(mut v: Vec<T>) -> Vec<u8> {
    let (ptr, len, cap, sz) = (v.as_mut_ptr(), v.len(), v.capacity(), ::std::mem::size_of::<T>());
    ::std::mem::forget(v);
    unsafe { Vec::from_raw_parts(ptr as *mut u8, len * sz, cap * sz) }
}

