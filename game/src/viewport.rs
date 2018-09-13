use std::collections::HashMap;
use std::cell::{Cell, RefCell};

use fate::math::{Rect, Rgba};

use rand::random;

use cubemap::{CubemapSelector, CubemapArrayID};
use mouse_cursor::{MouseCursor, SystemCursor};
use system::*;
use eid::EID;

#[derive(Debug)]
pub struct ViewportDB {
    border_color: Rgba<f32>,
    border_px: u32,
    highest_id: ViewportNodeID, // Do not keep; Replace by SlotMap!
    root: ViewportNodeID,
    focused: ViewportNodeID,
    hovered: Option<ViewportNodeID>,
    hovered_border: Option<ViewportNodeID>,
    dragged: Option<ViewportNodeID>,
    nodes: HashMap<ViewportNodeID, ViewportNode>,
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ViewportNodeID(u32);

#[derive(Debug, Clone, PartialEq)]
pub enum ViewportNode {
    Whole {
        parent: Option<ViewportNodeID>,
        info: RefCell<ViewportInfo>,
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
    pub skybox_cubemap_selector: CubemapSelector,
    pub camera: EID, // TODO: Multiple (stacked) cameras (but draw skybox once with one of them)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Split {
    pub origin: SplitOrigin,
    pub unit: SplitUnit,
    pub value: Cell<f32>,
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
    fn accept_leaf_viewport(&mut self, AcceptLeafViewport) {}
    fn accept_split_viewport(&mut self, AcceptSplitViewport) {}
}

#[derive(Debug)]
pub struct AcceptLeafViewport<'a> {
    pub id: ViewportNodeID,
    pub rect: Rect<u32, u32>, 
    pub info: &'a mut ViewportInfo,
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

#[derive(Debug)]
pub struct ViewportInputHandler;

#[derive(Debug)]
struct ViewportHoverer {
    pos: Vec2<u32>,
    found: Option<ViewportNodeID>,
    on_border: Option<ViewportNodeID>,
}

#[derive(Debug)]
pub struct ViewportDragger {
    pos: Vec2<u32>,
    id: ViewportNodeID,
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

impl ViewportNode {
    pub fn with_info(info: ViewportInfo) -> Self {
        ViewportNode::Whole {
            parent: None,
            info: info.into(),
        }
    }
}

impl ViewportDB {
    pub fn new(info: ViewportInfo) -> Self {
        let mut nodes = HashMap::new();
        let root = ViewportNodeID(0);
        nodes.insert(root, ViewportNode::with_info(info));
        let highest_id = root;
 
        Self {
            nodes,
            highest_id,
            root,
            focused: root,
            hovered: None,
            hovered_border: None,
            dragged: None,
            border_px: 1,
            border_color: Rgba::grey(0.96),
        }
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
                Some(ViewportNode::Split { split, .. }) => match split.direction {
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
            Some(Keysym::V) if state.is_down() => g.viewport_db_mut().split_v(),
            Some(Keysym::H) if state.is_down() => g.viewport_db_mut().split_h(),
            Some(Keysym::M) if state.is_down() => g.viewport_db_mut().merge(),
            _ => {},
        }
    }
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

impl ViewportDB {
    pub fn border_color(&self) -> Rgba<f32> {
        self.border_color
    }
    pub fn border_px(&self) -> u32 {
        self.border_px
    }
    pub fn root(&self) -> ViewportNodeID {
        self.root
    }
    pub fn hovered(&self) -> Option<ViewportNodeID> {
        self.hovered
    }
    pub fn hover(&mut self, id: Option<ViewportNodeID>) {
        if self.hovered != id {
            debug!("Now hovering {:?}", id);
            self.hovered = id;
        }
    }
    pub fn focused(&self) -> ViewportNodeID {
        self.focused
    }
    pub fn focus(&mut self, id: ViewportNodeID) {
        if self.focused != id {
            debug!("Now focusing {:?}", id);
            self.focused = id;
        }
    }
    pub fn drag(&mut self, id: Option<ViewportNodeID>) {
        self.dragged = id;
    }
    pub fn dragged(&self) -> Option<ViewportNodeID> {
        self.dragged
    }
    pub fn hover_border(&mut self, id: Option<ViewportNodeID>) {
        self.hovered_border = id;
    }
    pub fn hovered_border(&self) -> Option<ViewportNodeID> {
        self.hovered_border
    }
    pub fn node(&self, id: ViewportNodeID) -> Option<&ViewportNode> {
        self.nodes.get(&id)
    }
    pub fn node_mut(&mut self, id: ViewportNodeID) -> Option<&mut ViewportNode> {
        self.nodes.get_mut(&id)
    }
    pub fn split_h(&mut self) {
        self.split(SplitDirection::Horizontal)
    }
    pub fn split_v(&mut self) {
        self.split(SplitDirection::Vertical)
    }
    pub fn split(&mut self, direction: SplitDirection) {
        let id = self.focused();

        let c0_id = ViewportNodeID(self.highest_id.0 + 1);
        let c1_id = ViewportNodeID(self.highest_id.0 + 2);

        let info = {
            let node = self.node_mut(id).unwrap();
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
                    value: (0.).into(),
                }
            };
            info
        };

        self.highest_id.0 += 2;
        let c0_info = info.clone();
        let c1_info = ViewportInfo {
            clear_color: Rgba::<u8>::new_opaque(random(), random(), random()).map(|x| x as f32 / 255.),
            skybox_cubemap_selector: CubemapSelector {
                array_id: CubemapArrayID((random::<f32>() * 2_f32) as _),
                cubemap: (random::<f32>() * 2_f32) as _,
            },
            camera: c0_info.borrow().camera.clone(),
        }.into();
        debug!("Created {:#?}", c1_info);

        self.focus(c0_id);

        let c0_node = ViewportNode::Whole { info: c0_info, parent: Some(id) };
        let c1_node = ViewportNode::Whole { info: c1_info, parent: Some(id) };
        self.nodes.insert(c0_id, c0_node);
        self.nodes.insert(c1_id, c1_node);
    }
    /// Merges the focused viewport node into its neighbour.
    pub fn merge(&mut self) {
        let focus_id = self.focused();

        let (merge_id, info) = {
            let focus = self.node_mut(focus_id).unwrap();
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
            let merge = self.node_mut(merge_id).unwrap();
            let (parent, c0_id, c1_id) = match *merge {
                ViewportNode::Whole { .. } => panic!("A parent node can't be whole"),
                ViewportNode::Split { parent, children, .. } => (parent, children.0, children.1),
            };
            *merge = ViewportNode::Whole { info, parent };
            (c0_id, c1_id)
        };

        self.nodes.remove(&c0_id).unwrap();
        self.nodes.remove(&c1_id).unwrap();
        self.focus(merge_id);
    }
    pub fn visit(&self, rect: Rect<u32, u32>, f: &mut ViewportVisitor) {
        let root_id = self.root();
        let border_px = self.border_px();
        self.visit_viewport(root_id, rect, f, border_px)
    }
    fn visit_viewport(&self, id: ViewportNodeID, rect: Rect<u32, u32>, f: &mut ViewportVisitor, border_px: u32) {
        let (c0, c1, r0, r1) = {
            let node = self.node(id).unwrap();
            match *node {
                ViewportNode::Split { children: (c0, c1), split: Split { origin, unit, ref value, direction }, parent } => {
                    let v = value.get();
                    let mut distance_from_left_or_bottom_px = match (origin, unit, direction) {
                        (SplitOrigin::LeftOrBottom, SplitUnit::Px, _) => v as u32,
                        (SplitOrigin::LeftOrBottom, SplitUnit::Ratio, SplitDirection::Horizontal) => (v * rect.h as f32).round() as u32,
                        (SplitOrigin::LeftOrBottom, SplitUnit::Ratio, SplitDirection::Vertical)   => (v * rect.w as f32).round() as u32,
                        (SplitOrigin::Middle, SplitUnit::Px, SplitDirection::Horizontal) => (rect.h / 2 + v as u32),
                        (SplitOrigin::Middle, SplitUnit::Px, SplitDirection::Vertical)   => (rect.w / 2 + v as u32),
                        (SplitOrigin::Middle, SplitUnit::Ratio, SplitDirection::Horizontal) => (((v + 1.) / 2.) * rect.h as f32).round() as u32,
                        (SplitOrigin::Middle, SplitUnit::Ratio, SplitDirection::Vertical)   => (((v + 1.) / 2.) * rect.w as f32).round() as u32,
                        (SplitOrigin::RightOrTop, SplitUnit::Px, SplitDirection::Horizontal) => (rect.h - v as u32),
                        (SplitOrigin::RightOrTop, SplitUnit::Px, SplitDirection::Vertical)   => (rect.w - v as u32),
                        (SplitOrigin::RightOrTop, SplitUnit::Ratio, SplitDirection::Horizontal) => ((1. - v) * rect.h as f32).round() as u32,
                        (SplitOrigin::RightOrTop, SplitUnit::Ratio, SplitDirection::Vertical)   => ((1. - v) * rect.w as f32).round() as u32,
                    };

                    f.accept_split_viewport(AcceptSplitViewport{ id, rect, split_direction: direction, distance_from_left_or_bottom_px: &mut distance_from_left_or_bottom_px, parent, border_px });

                    let d = distance_from_left_or_bottom_px as f32;
                    value.set(match (origin, unit, direction) {
                        (SplitOrigin::LeftOrBottom, SplitUnit::Px, _) => d,
                        (SplitOrigin::LeftOrBottom, SplitUnit::Ratio, SplitDirection::Horizontal) => d / rect.h as f32,
                        (SplitOrigin::LeftOrBottom, SplitUnit::Ratio, SplitDirection::Vertical)   => d / rect.w as f32,
                        (SplitOrigin::Middle, SplitUnit::Px, SplitDirection::Horizontal) => (d - (rect.h / 2) as f32),
                        (SplitOrigin::Middle, SplitUnit::Px, SplitDirection::Vertical)   => (d - (rect.w / 2) as f32),
                        (SplitOrigin::Middle, SplitUnit::Ratio, SplitDirection::Horizontal) => 2. * d / rect.h as f32 - 1.,
                        (SplitOrigin::Middle, SplitUnit::Ratio, SplitDirection::Vertical)   => 2. * d / rect.w as f32 - 1.,
                        (SplitOrigin::RightOrTop, SplitUnit::Px, SplitDirection::Horizontal) => rect.h as f32 - d,
                        (SplitOrigin::RightOrTop, SplitUnit::Px, SplitDirection::Vertical)   => rect.w as f32 - d,
                        (SplitOrigin::RightOrTop, SplitUnit::Ratio, SplitDirection::Horizontal) => 1. - (d / rect.h as f32),
                        (SplitOrigin::RightOrTop, SplitUnit::Ratio, SplitDirection::Vertical)   => 1. - (d / rect.w as f32),
                    });

                    let mut r0 = rect;
                    let mut r1 = rect;
                    match direction {
                        SplitDirection::Horizontal => {
                            r0.h = distance_from_left_or_bottom_px;
                            r1.h = rect.h - r0.h;
                            r1.y = rect.y + r0.h;
                        },
                        SplitDirection::Vertical => {
                            r0.w = distance_from_left_or_bottom_px;
                            r1.w = rect.w - r0.w;
                            r1.x = rect.x + r0.w;
                        },
                    };

                    (c0, c1, r0, r1)
                },
                ViewportNode::Whole { ref info, parent } => {
                    let border_px = if parent.is_some() {
                        border_px
                    } else {
                        0
                    };
                    return f.accept_leaf_viewport(AcceptLeafViewport{ id, rect, info: &mut info.borrow_mut(), parent, border_px });
                },
            }
        };
        self.visit_viewport(c0, r0, f, border_px);
        self.visit_viewport(c1, r1, f, border_px);
    }
}
