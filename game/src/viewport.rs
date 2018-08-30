use std::collections::HashMap;

use fate::math::{Rect, Rgba};

use rand::random;

use mouse_cursor::{MouseCursor, SystemCursor};
use system::*;

#[derive(Debug)]
pub struct ViewportDB {
    viewport_border_color: Rgba<f32>,
    viewport_border_px: u32,
    _highest_viewport_node_id: ViewportNodeID, // Do not keep; Replace by SlotMap!
    root_viewport_node_id: ViewportNodeID,
    focused_viewport_node_id: ViewportNodeID,
    hovered_viewport_node_id: Option<ViewportNodeID>,
    viewport_nodes: HashMap<ViewportNodeID, ViewportNode>,
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ViewportNodeID(u32);

#[derive(Debug, Clone, PartialEq)]
pub enum ViewportNode {
    Whole {
        parent: Option<ViewportNodeID>,
        info: ViewportInfo
    },
    Split {
        parent: Option<ViewportNodeID>,
        split: Split,
        children: (ViewportNodeID, ViewportNodeID),
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ViewportInfo {
    // TODO: Describes what a viewport displays    
    pub clear_color: Rgba<f32>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Split {
    pub origin: SplitOrigin,
    pub unit: SplitUnit,
    pub value: f32,
    pub direction: SplitDirection,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SplitOrigin {
    LeftOrBottom, Middle, RightOrTop,    
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SplitUnit {
    Ratio, Px,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal, Vertical,
}

pub trait ViewportVisitor {
    fn accept_viewport(&mut self, id: ViewportNodeID, r: Rect<u32, u32>, info: &mut ViewportInfo, parent: Option<ViewportNodeID>, border_px: u32);
}

#[derive(Debug)]
pub struct ViewportInputHandler;

#[derive(Debug)]
struct ViewportPicker {
    pos: Vec2<u32>,
    found: Option<ViewportNodeID>,
    on_border: Option<ViewportNodeID>,
}



impl ViewportInputHandler {
    pub fn new() -> Self {
        ViewportInputHandler 
    }
}

impl Default for ViewportNode {
    fn default() -> Self {
        ViewportNode::Whole {
            parent: None,
            info: Default::default(),
        }
    }
}

impl ViewportDB {
    pub fn new() -> Self {
        let mut viewport_nodes = HashMap::new();
        let root_viewport_node_id = ViewportNodeID(0);
        viewport_nodes.insert(root_viewport_node_id, ViewportNode::default());
        let _highest_viewport_node_id = root_viewport_node_id;
 
        Self {
            viewport_nodes,
            _highest_viewport_node_id,
            root_viewport_node_id,
            focused_viewport_node_id: root_viewport_node_id,
            hovered_viewport_node_id: Some(root_viewport_node_id),
            viewport_border_px: 1,
            viewport_border_color: Rgba::grey(0.96),
        }
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
        g.viewport_db_mut().set_hovered_viewport_node_id(visitor.found);
    }
    fn on_mouse_leave(&mut self, g: &mut G) {
        g.viewport_db_mut().set_hovered_viewport_node_id(None);
    }
    fn on_mouse_button(&mut self, g: &mut G, btn: MouseButton, state: ButtonState) {
        match btn {
            MouseButton::Left if state.is_down() => {
                if let Some(hovered) = g.viewport_db().hovered_viewport_node_id() {
                    g.viewport_db_mut().focus_viewport(hovered);
                }
            },
            _ => {},
        }
    }
    fn on_key(&mut self, g: &mut G, key: Key, state: KeyState) {
        match key.sym {
            Some(Keysym::V) if state.is_down() => g.viewport_db_mut().viewport_split_v(),
            Some(Keysym::H) if state.is_down() => g.viewport_db_mut().viewport_split_h(),
            Some(Keysym::M) if state.is_down() => g.viewport_db_mut().viewport_merge(),
            _ => {},
        }
    }
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


impl ViewportDB {
    pub fn viewport_border_color(&self) -> Rgba<f32> {
        self.viewport_border_color
    }
    pub fn viewport_border_px(&self) -> u32 {
        self.viewport_border_px
    }
    pub fn root_viewport_node_id(&self) -> ViewportNodeID {
        self.root_viewport_node_id
    }
    pub fn focused_viewport_node_id(&self) -> ViewportNodeID {
        self.focused_viewport_node_id
    }
    pub fn hovered_viewport_node_id(&self) -> Option<ViewportNodeID> {
        self.hovered_viewport_node_id
    }
    pub fn set_hovered_viewport_node_id(&mut self, id: Option<ViewportNodeID>) {
        debug!("Now hovering {:?}", id);
        self.hovered_viewport_node_id = id;
    }
    pub fn focus_viewport(&mut self, id: ViewportNodeID) {
        debug!("Now focusing {:?}", id);
        self.focused_viewport_node_id = id;
    }
    pub fn viewport_node(&self, id: ViewportNodeID) -> Option<&ViewportNode> {
        self.viewport_nodes.get(&id)
    }
    pub fn viewport_node_mut(&mut self, id: ViewportNodeID) -> Option<&mut ViewportNode> {
        self.viewport_nodes.get_mut(&id)
    }
    pub fn viewport_split_h(&mut self) {
        self.viewport_split(SplitDirection::Horizontal)
    }
    pub fn viewport_split_v(&mut self) {
        self.viewport_split(SplitDirection::Vertical)
    }
    pub fn viewport_split(&mut self, direction: SplitDirection) {
        let id = self.focused_viewport_node_id();

        let c0_id = ViewportNodeID(self._highest_viewport_node_id.0 + 1);
        let c1_id = ViewportNodeID(self._highest_viewport_node_id.0 + 2);

        let info = {
            let node = self.viewport_node_mut(id).unwrap();
            let (parent, info) = match *node {
                ViewportNode::Split { .. } => panic!("A non-leaf viewport node cannot be focused"),
                ViewportNode::Whole { ref info, parent, .. } => (parent, info.clone()),
            };
            *node = ViewportNode::Split {
                parent,
                children: (c0_id, c1_id),
                split: Split {
                    direction,
                    origin: SplitOrigin::Middle,
                    unit: SplitUnit::Ratio,
                    value: 0.,
                }
            };
            info
        };

        self._highest_viewport_node_id.0 += 2;
        let c0_info = info.clone();
        let mut c1_info = info;

        self.focused_viewport_node_id = c0_id;
        c1_info.clear_color = Rgba::<u8>::new_opaque(random(), random(), random()).map(|x| x as f32 / 255.);

        let c0_node = ViewportNode::Whole { info: c0_info, parent: Some(id) };
        let c1_node = ViewportNode::Whole { info: c1_info, parent: Some(id) };
        self.viewport_nodes.insert(c0_id, c0_node);
        self.viewport_nodes.insert(c1_id, c1_node);
    }
    /// Merges the focused viewport node into its neighbour.
    pub fn viewport_merge(&mut self) {
        let focus_id = self.focused_viewport_node_id();

        let (merge_id, info) = {
            let focus = self.viewport_node_mut(focus_id).unwrap();
            let (parent, info) = match *focus {
                ViewportNode::Split { .. } => panic!("A non-leaf viewport node cannot be focused"),
                ViewportNode::Whole { parent, ref info } => (parent, info.clone()),
            };
            (parent, info)
        };

        let merge_id = match merge_id {
            None => return,
            Some(x) => x,
        };

        let (c0_id, c1_id) = {
            let merge = self.viewport_node_mut(merge_id).unwrap();
            let (parent, c0_id, c1_id) = match *merge {
                ViewportNode::Whole { .. } => panic!("A parent node can't be whole"),
                ViewportNode::Split { parent, children, .. } => (parent, children.0, children.1),
            };
            *merge = ViewportNode::Whole { info, parent };
            (c0_id, c1_id)
        };

        self.viewport_nodes.remove(&c0_id).unwrap();
        self.viewport_nodes.remove(&c1_id).unwrap();
        self.focused_viewport_node_id = merge_id;
    }
    pub fn visit_viewports(&mut self, rect: Rect<u32, u32>, f: &mut ViewportVisitor) {
        let root_id = self.root_viewport_node_id();
        let border_px = self.viewport_border_px();
        self.visit_viewport(root_id, rect, f, border_px)
    }
    fn visit_viewport(&mut self, id: ViewportNodeID, rect: Rect<u32, u32>, f: &mut ViewportVisitor, border_px: u32) {
        let (c0, c1, r0, r1) = {
            let node = self.viewport_node_mut(id).unwrap();
            match *node {
                ViewportNode::Split { children: (c0, c1), split: Split { origin, unit, value, direction }, .. } => {
                    // FIXME: assuming value is relative to middle
                    let mut r0 = rect;
                    let mut r1 = rect;
                    match direction {
                        SplitDirection::Horizontal => {
                            r0.h /= 2;
                            r1.h = rect.h - r0.h;
                            r1.y = rect.y + r0.h;
                        },
                        SplitDirection::Vertical => {
                            r0.w /= 2;
                            r1.w = rect.w - r0.w;
                            r1.x = rect.x + r0.w;
                        },
                    }
                    (c0, c1, r0, r1)
                },
                ViewportNode::Whole { ref mut info, parent } => {
                    let border_px = if parent.is_some() {
                        border_px
                    } else {
                        0
                    };
                    return f.accept_viewport(id, rect, info, parent, border_px);
                },
            }
        };
        self.visit_viewport(c0, r0, f, border_px);
        self.visit_viewport(c1, r1, f, border_px);
    }
}