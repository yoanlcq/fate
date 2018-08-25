use std::fmt::{self, Formatter, Debug};
use std::ops::{Deref, DerefMut};
use task::{Task, UntypedTask};

pub struct TaskBox(Box<UntypedTask + Send + Sync>);

impl Debug for TaskBox {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<TaskBox>")
    }
}

impl<T: Task + Send + Sync + 'static> From<T> for TaskBox {
    fn from(t: T) -> Self {
        TaskBox(Box::new(t))
    }
}

impl Deref for TaskBox {
    type Target = Box<UntypedTask + Send + Sync>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TaskBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
