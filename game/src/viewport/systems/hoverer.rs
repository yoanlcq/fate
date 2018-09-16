use fate::math::Vec2;
use super::super::*;
use system::*;

#[derive(Debug)]
pub struct ViewportHoverer {
    pub pos: Vec2<u32>,
    pub found: Option<ViewportNodeID>,
    pub on_border: Option<ViewportNodeID>,
}


impl ViewportVisitor for ViewportHoverer {
    fn accept_leaf_viewport(&mut self, args: AcceptLeafViewport) {
        if args.rect.contains_point(self.pos) {
            self.found = Some(args.id);
        }
    }
    fn accept_split_viewport(&mut self, args: AcceptSplitViewport) {
        if self.on_border.is_some() || !args.rect.contains_point(self.pos) {
            return;
        }
        let x = self.pos.x.saturating_sub(args.rect.x);
        let y = self.pos.y.saturating_sub(args.rect.y);
        let v = match args.split_direction {
            SplitDirection::Horizontal => y,
            SplitDirection::Vertical => x,
        };
        if (v as i32 - *args.distance_from_left_or_bottom_px as i32).abs() <= args.border_px as i32 {
            self.on_border = Some(args.id);
        }
    }
}

