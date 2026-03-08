#![allow(unsafe_op_in_unsafe_fn)]

use std::collections::HashMap;

use backend_api::{Backend, BackendError};
use native_schema::{
    ElementKind, EventKind, LayoutFrame, Mutation, PropKey, PropValue, UiEvent, UiNodeId,
};

/// Temporary iOS backend that accepts canonical batches and maintains a shadow tree.
/// Incremental UIKit execution is deferred to the dedicated P0-06 step.
#[derive(Default)]
pub struct NativeBackend {
    state: ShadowState,
}

impl Backend for NativeBackend {
    fn apply_mutations(&mut self, mutations: &[Mutation]) -> Result<(), BackendError> {
        for mutation in mutations {
            self.state.apply(mutation)?;
        }
        Ok(())
    }

    fn apply_layout(&mut self, frames: &[LayoutFrame]) -> Result<(), BackendError> {
        for frame in frames {
            self.state.frames.insert(frame.id, *frame);
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), BackendError> {
        Ok(())
    }

    fn drain_events(&mut self) -> Vec<UiEvent> {
        Vec::new()
    }
}

#[derive(Default)]
struct ShadowState {
    root: Option<UiNodeId>,
    nodes: HashMap<UiNodeId, ShadowNode>,
    frames: HashMap<UiNodeId, LayoutFrame>,
}

impl ShadowState {
    fn apply(&mut self, mutation: &Mutation) -> Result<(), BackendError> {
        match mutation {
            Mutation::CreateNode { id, kind } => {
                self.nodes.insert(*id, ShadowNode::new(*kind));
                if self.root.is_none() {
                    self.root = Some(*id);
                }
            }
            Mutation::CreateTextNode { id, text } => {
                let mut node = ShadowNode::new(ElementKind::Text);
                node.text = Some(text.clone());
                self.nodes.insert(*id, node);
                if self.root.is_none() {
                    self.root = Some(*id);
                }
            }
            Mutation::SetText { id, text } => self.node_mut(*id)?.text = Some(text.clone()),
            Mutation::SetProp { id, key, value } => {
                self.node_mut(*id)?.props.insert(*key, value.clone());
            }
            Mutation::InsertChild {
                parent,
                child,
                index,
            } => {
                self.node_mut(*child)?.parent = Some(*parent);
                let children = &mut self.node_mut(*parent)?.children;
                let index = (*index as usize).min(children.len());
                if !children.contains(child) {
                    children.insert(index, *child);
                }
            }
            Mutation::MoveNode {
                id,
                new_parent,
                index,
            } => {
                self.detach(*id);
                self.node_mut(*id)?.parent = Some(*new_parent);
                let siblings = &mut self.node_mut(*new_parent)?.children;
                let index = (*index as usize).min(siblings.len());
                siblings.insert(index, *id);
            }
            Mutation::ReplaceNode { old, new_id, kind } => {
                let parent = self.nodes.get(old).and_then(|node| node.parent);
                let position = parent.and_then(|parent_id| {
                    self.nodes
                        .get(&parent_id)
                        .and_then(|node| node.children.iter().position(|child| child == old))
                });
                self.remove_subtree(*old);
                self.nodes.insert(*new_id, ShadowNode::new(*kind));
                if let Some(parent_id) = parent {
                    let children = &mut self.node_mut(parent_id)?.children;
                    let index = position.unwrap_or(children.len());
                    children.insert(index, *new_id);
                    self.node_mut(*new_id)?.parent = Some(parent_id);
                } else {
                    self.root = Some(*new_id);
                }
            }
            Mutation::RemoveNode { id } => {
                self.detach(*id);
                self.remove_subtree(*id);
            }
            Mutation::AttachEventListener { id, event } => {
                self.node_mut(*id)?.events.push(*event);
            }
        }
        Ok(())
    }

    fn node_mut(&mut self, id: UiNodeId) -> Result<&mut ShadowNode, BackendError> {
        self.nodes
            .get_mut(&id)
            .ok_or_else(|| BackendError::BatchRejected(format!("unknown node id {id}")))
    }

    fn detach(&mut self, id: UiNodeId) {
        let parent = self.nodes.get(&id).and_then(|node| node.parent);
        if let Some(parent_id) = parent {
            if let Some(parent_node) = self.nodes.get_mut(&parent_id) {
                parent_node.children.retain(|child| *child != id);
            }
        }
    }

    fn remove_subtree(&mut self, id: UiNodeId) {
        let children = self
            .nodes
            .get(&id)
            .map(|node| node.children.clone())
            .unwrap_or_default();
        for child in children {
            self.remove_subtree(child);
        }
        self.nodes.remove(&id);
        self.frames.remove(&id);
        if self.root == Some(id) {
            self.root = None;
        }
    }
}

struct ShadowNode {
    kind: ElementKind,
    parent: Option<UiNodeId>,
    children: Vec<UiNodeId>,
    props: HashMap<PropKey, PropValue>,
    text: Option<String>,
    events: Vec<EventKind>,
}

impl ShadowNode {
    fn new(kind: ElementKind) -> Self {
        Self {
            kind,
            parent: None,
            children: Vec::new(),
            props: HashMap::new(),
            text: None,
            events: Vec::new(),
        }
    }
}
