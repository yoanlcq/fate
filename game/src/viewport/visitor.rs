use fate::math::Rect;
use super::*;

pub trait ViewportVisitor {
    fn accept_leaf_viewport(&mut self, AcceptLeafViewport) {}
    fn accept_split_viewport(&mut self, AcceptSplitViewport) {}
}

#[derive(Debug)]
pub struct AcceptLeafViewport<'a> {
    pub id: ViewportNodeID,
    pub rect: Rect<u32, u32>, 
    pub info: &'a mut LeafViewport,
    pub parent: Option<ViewportNodeID>,
    pub border_px: u32,
}
#[derive(Debug)]
pub struct AcceptSplitViewport<'a> {
    pub id: ViewportNodeID,
    pub rect: Rect<u32, u32>, 
    pub split_direction: SplitDirection,
    pub distance_from_left_or_bottom_px: &'a mut u32,
    pub parent: Option<ViewportNodeID>,
    pub border_px: u32,
}

