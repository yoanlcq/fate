use super::Progress;

pub trait Loading {
    type Item;
    type Error;
    type Progress: Progress;
    fn poll(&self) -> Self::Progress;
    fn wait(self) -> Result<Self::Item, Self::Error>;
    fn cancel(self); // NOTE: Should this return a Future indicating the progress of cancellation??
}


