use mouse_cursor::{MouseCursor, SystemCursor};
use fate::math::Rect;
use g::{ViewportVisitor, ViewportNodeID, ViewportInfo};
use system::*;

#[derive(Debug)]
pub struct ViewportInputHandler;

impl ViewportInputHandler {
    pub fn new() -> Self {
        ViewportInputHandler 
    }
}

impl System for ViewportInputHandler {
    fn on_mouse_motion(&mut self, g: &mut G, pos: Vec2<f64>) {
        // TODO: Update g.hovered_viewport_node and g.focused_viewport_node.
        g.mouse_cursor = MouseCursor::System(SystemCursor::Hand);

        let mut pos = pos.map(|x| x.round() as u32);
        pos.y = g.input.canvas_size().h.saturating_sub(pos.y);
        let mut visitor = ViewportPicker { pos, found: None, on_border: None, };
        g.visit_viewports(&mut visitor);
        g.set_hovered_viewport_node_id(visitor.found);
    }
    fn on_mouse_leave(&mut self, g: &mut G) {
        g.set_hovered_viewport_node_id(None);
    }
    fn on_mouse_button(&mut self, g: &mut G, btn: MouseButton, state: ButtonState) {
        match btn {
            MouseButton::Left if state.is_down() => {
                if let Some(hovered) = g.hovered_viewport_node_id() {
                    g.focus_viewport(hovered);
                }
            },
            _ => {},
        }
    }
    fn on_key(&mut self, g: &mut G, key: Key, state: KeyState) {
        match key.sym {
            Some(Keysym::V) if state.is_down() => g.viewport_split_v(),
            Some(Keysym::H) if state.is_down() => g.viewport_split_h(),
            Some(Keysym::M) if state.is_down() => g.viewport_merge(),
            _ => {},
        }
    }
}

struct ViewportPicker {
    pos: Vec2<u32>,
    found: Option<ViewportNodeID>,
    on_border: Option<ViewportNodeID>,
}

impl ViewportVisitor for ViewportPicker {
    fn accept_viewport(&mut self, id: ViewportNodeID, r: Rect<u32, u32>, info: &mut ViewportInfo, parent: Option<ViewportNodeID>, border_px: u32) {
        if r.contains_point(self.pos) {
            self.found = Some(id);
        }
        /*
        if let (Some(parent), Some(found)) = (parent, found) {
            if self.pos.x < r.x + border_px {
                self.on_border = Some(parent)
            }
        }
        */
    }
}
