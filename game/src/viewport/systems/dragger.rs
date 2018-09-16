use fate::math::Vec2;
use super::super::*;
use system::*;

#[derive(Debug)]
pub struct ViewportDragger {
    pub pos: Vec2<u32>,
    pub id: ViewportNodeID,
}

impl ViewportVisitor for ViewportDragger {
    fn accept_split_viewport(&mut self, args: AcceptSplitViewport) {
        if args.id != self.id {
            return;
        }
        let x = self.pos.x.saturating_sub(args.rect.x);
        let y = self.pos.y.saturating_sub(args.rect.y);
        *args.distance_from_left_or_bottom_px = match args.split_direction {
            SplitDirection::Horizontal => y,
            SplitDirection::Vertical => x,
        };
    }
}

