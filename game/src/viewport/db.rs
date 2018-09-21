use std::cell::Cell;
use fate::math::{Rgba, Rect};
use fate::dmap::{Key, DMap};
use cubemap::{CubemapSelector, CubemapArrayID};
use rand::random;
use super::*;

pub type ViewportNodeID = Key;

#[derive(Debug)]
pub struct ViewportDB {
    border_color: Rgba<f32>,
    border_px: u32,
    root: ViewportNodeID,
    focused: ViewportNodeID,
    hovered: Option<ViewportNodeID>,
    hovered_border: Option<ViewportNodeID>,
    dragged: Option<ViewportNodeID>,
    nodes: DMap<ViewportNode>,
}


impl ViewportDB {
    pub fn new(leaf: LeafViewport) -> Self {
        let mut nodes = DMap::with_capacity_and_free_list_capacity(16, 16);
        let root = nodes.insert(ViewportNode::new_root(leaf));
 
        Self {
            nodes,
            root,
            focused: root,
            hovered: None,
            hovered_border: None,
            dragged: None,
            border_px: 1,
            border_color: Rgba::grey(0.96),
        }
    }
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
    pub fn root_node(&self) -> &ViewportNode {
        self.node(self.root()).unwrap()
    }
    pub fn hovered_node(&self) -> Option<&ViewportNode> {
        self.node(self.hovered()?)
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
    pub fn focused_node(&self) -> &ViewportNode {
        self.node(self.focused()).unwrap()
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
        self.nodes.get(id)
    }
    pub fn node_mut(&mut self, id: ViewportNodeID) -> Option<&mut ViewportNode> {
        self.nodes.get_mut(id)
    }
    // Returns the new leaf node.
    pub fn split_focused(&mut self, direction: SplitDirection) -> ViewportNodeID {
        let id = self.focused();
        self.split(id, direction)
    }
    pub fn split(&mut self, id: ViewportNodeID, direction: SplitDirection) -> ViewportNodeID {
        let info = {
            let node = self.node_mut(id).unwrap();
            let info = match node.value {
                ViewportNodeValue::Split { .. } => panic!("A non-leaf viewport node cannot be focused"),
                ViewportNodeValue::Leaf(ref info) => info.clone(),
            };
            info
        };
        let c0_info = info.clone();
        let c1_info = info;
        let c0_node = ViewportNode { parent: Some(id), value: ViewportNodeValue::Leaf(c0_info), };
        let c1_node = ViewportNode { parent: Some(id), value: ViewportNodeValue::Leaf(c1_info), };
        let c0_id = self.nodes.insert(c0_node);
        let c1_id = self.nodes.insert(c1_node);

        {
            let node = self.node_mut(id).unwrap();
            node.value = ViewportNodeValue::Split {
                children: (c0_id, c1_id),
                split: Split {
                    direction,
                    origin: SplitOrigin::Middle,
                    unit: SplitUnit::Ratio,
                    value: (0.).into(),
                }
            };
        }

        self.focus(c0_id);
        c1_id
    }
    /// Merges the focused viewport node into its neighbour.
    pub fn merge(&mut self) {
        let focus_id = self.focused();

        let (merge_id, info) = {
            let focus = self.node_mut(focus_id).unwrap();
            let info = match focus.value {
                ViewportNodeValue::Split { .. } => panic!("A non-leaf viewport node cannot be focused"),
                ViewportNodeValue::Leaf(ref info) => info.clone(),
            };
            (focus.parent, info)
        };

        let merge_id = match merge_id {
            None => return,
            Some(x) => x,
        };

        let (c0_id, c1_id) = {
            let merge = self.node_mut(merge_id).unwrap();
            let (c0_id, c1_id) = match merge.value {
                ViewportNodeValue::Leaf(_) => panic!("A parent node can't be whole"),
                ViewportNodeValue::Split { children, .. } => (children.0, children.1),
            };
            merge.value = ViewportNodeValue::Leaf(info);
            (c0_id, c1_id)
        };

        self.nodes.remove(c0_id).unwrap();
        self.nodes.remove(c1_id).unwrap();
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
            match node.value {
                ViewportNodeValue::Split { children: (c0, c1), split: Split { origin, unit, ref value, direction } } => {
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

                    f.accept_split_viewport(AcceptSplitViewport{ id, rect, split_direction: direction, distance_from_left_or_bottom_px: &mut distance_from_left_or_bottom_px, parent: node.parent, border_px });

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
                ViewportNodeValue::Leaf(ref info) => {
                    let border_px = if node.parent.is_some() {
                        border_px
                    } else {
                        0
                    };
                    return f.accept_leaf_viewport(AcceptLeafViewport{ id, rect, info: &mut info.borrow_mut(), parent: node.parent, border_px });
                },
            }
        };
        self.visit_viewport(c0, r0, f, border_px);
        self.visit_viewport(c1, r1, f, border_px);
    }
}
