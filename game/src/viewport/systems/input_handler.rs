use mouse_cursor::{MouseCursor, SystemCursor};
use super::*;
use super::super::*;
use system::*;

#[derive(Debug)]
pub struct ViewportInputHandler;

impl ViewportInputHandler {
    pub fn new() -> Self {
        ViewportInputHandler 
    }
}

impl System for ViewportInputHandler {
    fn draw(&mut self, g: &mut G, _: &Draw) {
        if g.viewport_db().dragged().is_none() {
            if let Some(pos) = g.input.mouse_position() {
                let mut pos = pos.map(|x| x.round() as u32);
                pos.y = g.input.canvas_size().h.saturating_sub(pos.y);

                let mut visitor = ViewportHoverer { pos, found: None, on_border: None, };
                g.visit_viewports(&mut visitor);
                g.viewport_db_mut().hover(visitor.found);
                g.viewport_db_mut().hover_border(visitor.on_border);
            }
        }

        let cursor = match g.viewport_db().hovered_border() {
            None => SystemCursor::Arrow,
            Some(id) => match g.viewport_db().node(id) {
                Some(ViewportNode { value: ViewportNodeValue::Split { split, .. }, .. }) => match split.direction {
                    SplitDirection::Horizontal => SystemCursor::ResizeV,
                    SplitDirection::Vertical => SystemCursor::ResizeH,
                },
                _ => unreachable!(),
            },
        };
        g.mouse_cursor = MouseCursor::System(cursor);
    }
    fn on_mouse_motion(&mut self, g: &mut G, pos: Vec2<f64>) {
        let mut pos = pos.map(|x| x.round() as u32);
        pos.y = g.input.canvas_size().h.saturating_sub(pos.y);

        if let Some(id) = g.viewport_db().dragged() {
            let mut visitor = ViewportDragger { pos, id };
            g.visit_viewports(&mut visitor);
        }
    }
    fn on_mouse_leave(&mut self, g: &mut G) {
        g.viewport_db_mut().hover(None);
    }
    fn on_mouse_button(&mut self, g: &mut G, btn: MouseButton, state: ButtonState) {
        match btn {
            MouseButton::Left if state.is_down() => {
                if let Some(border) = g.viewport_db().hovered_border() {
                    g.viewport_db_mut().drag(Some(border));
                } else if let Some(hovered) = g.viewport_db().hovered() {
                    g.viewport_db_mut().focus(hovered);
                }
            },
            MouseButton::Left if state.is_up() => {
                g.viewport_db_mut().drag(None);
            },
            _ => {},
        }
    }
    fn on_key(&mut self, g: &mut G, key: Key, state: KeyState) {
        match key.sym {
            Some(Keysym::V) if state.is_down() => { g.viewport_db_mut().split_focused(SplitDirection::Vertical); },
            Some(Keysym::H) if state.is_down() => { g.viewport_db_mut().split_focused(SplitDirection::Horizontal); },
            Some(Keysym::M) if state.is_down() => { g.viewport_db_mut().merge() },
            _ => {},
        }
    }
}
