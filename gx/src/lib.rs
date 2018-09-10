#[macro_use]
extern crate bitflags;
extern crate fate_gl;
extern crate fate_math as math;

pub use fate_gl::gl45_core as gl;

pub mod attrib;
pub use attrib::{Attrib, ProgramAttribs};
#[macro_use]
pub mod error;
pub use error::*;
pub mod get;
pub use self::get::*;
pub mod object;
pub use self::object::*;
pub mod buffer;
pub use self::buffer::*;
pub mod shader;
pub mod program;
pub use self::program::*;
pub mod texture_unit;
pub use self::texture_unit::*;
pub mod missing_bits;
pub mod utilities;
pub use self::utilities::*;
pub mod context_summary;
pub use context_summary::ContextSummary;
pub mod debug;
pub use self::debug::*;
pub mod version;
pub use self::version::*;
pub mod extensions;
pub use extensions::*;
pub mod meminfo;
pub use meminfo::*;
pub mod query;
pub use query::*;
pub mod boot;
pub use boot::*;
pub mod topology;
pub use topology::*;

