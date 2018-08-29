use dmc;
pub use dmc::SystemCursor;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum MouseCursor {
    System(dmc::SystemCursor),
}

impl Default for MouseCursor {
    fn default() -> Self {
        MouseCursor::System(SystemCursor::Arrow)
    }
}

