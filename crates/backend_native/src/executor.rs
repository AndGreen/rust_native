use std::collections::HashMap;

use backend_api::BackendError;
use native_schema::{
    ElementKind, EventKind, LayoutFrame, Mutation, PropKey, PropValue, UiEvent, UiNodeId,
};

pub trait PlatformAdapter {
    type Handle: Copy + Eq;

    fn create_view(
        &mut self,
        kind: ElementKind,
        text: Option<&str>,
    ) -> Result<Self::Handle, BackendError>;
    fn attach_root(
        &mut self,
        node_id: UiNodeId,
        handle: Self::Handle,
    ) -> Result<(), BackendError>;
    fn detach_root(
        &mut self,
        node_id: UiNodeId,
        handle: Self::Handle,
    ) -> Result<(), BackendError>;
    fn insert_child(
        &mut self,
        parent: Self::Handle,
        child_id: UiNodeId,
        child: Self::Handle,
        index: usize,
    ) -> Result<(), BackendError>;
    fn remove_child(
        &mut self,
        parent: Self::Handle,
        child: Self::Handle,
    ) -> Result<(), BackendError>;
    fn remove_view(
        &mut self,
        node_id: UiNodeId,
        handle: Self::Handle,
        listeners: &[EventKind],
    ) -> Result<(), BackendError>;
    fn set_text(
        &mut self,
        kind: ElementKind,
        handle: Self::Handle,
        text: &str,
    ) -> Result<(), BackendError>;
    fn set_prop(
        &mut self,
        kind: ElementKind,
        handle: Self::Handle,
        props: &HashMap<PropKey, PropValue>,
        key: PropKey,
    ) -> Result<(), BackendError>;
    fn attach_listener(
        &mut self,
        kind: ElementKind,
        handle: Self::Handle,
        node_id: UiNodeId,
        event: EventKind,
    ) -> Result<(), BackendError>;
    fn apply_frame(
        &mut self,
        handle: Self::Handle,
        frame: LayoutFrame,
    ) -> Result<(), BackendError>;
    fn flush(&mut self) -> Result<(), BackendError>;
    fn drain_events(&mut self) -> Vec<UiEvent>;
}

#[derive(Debug, Clone)]
pub struct NodeRecord<H>
where
    H: Copy + Eq,
{
    pub kind: ElementKind,
    pub handle: H,
    pub parent: Option<UiNodeId>,
    pub children: Vec<UiNodeId>,
    pub props: HashMap<PropKey, PropValue>,
    pub text: Option<String>,
    pub listeners: Vec<EventKind>,
}

impl<H> NodeRecord<H>
where
    H: Copy + Eq,
{
    fn new(kind: ElementKind, handle: H, text: Option<String>) -> Self {
        Self {
            kind,
            handle,
            parent: None,
            children: Vec::new(),
            props: HashMap::new(),
            text,
            listeners: Vec::new(),
        }
    }
}

pub struct ExecutorState<H>
where
    H: Copy + Eq,
{
    root_id: Option<UiNodeId>,
    nodes: HashMap<UiNodeId, NodeRecord<H>>,
    frames: HashMap<UiNodeId, LayoutFrame>,
    pending_layout: Vec<LayoutFrame>,
}

impl<H> Default for ExecutorState<H>
where
    H: Copy + Eq,
{
    fn default() -> Self {
        Self {
            root_id: None,
            nodes: HashMap::new(),
            frames: HashMap::new(),
            pending_layout: Vec::new(),
        }
    }
}

impl<H> ExecutorState<H>
where
    H: Copy + Eq,
{
    pub fn apply_mutations<A>(&mut self, adapter: &mut A, mutations: &[Mutation]) -> Result<(), BackendError>
    where
        A: PlatformAdapter<Handle = H>,
    {
        for mutation in mutations {
            self.apply_mutation(adapter, mutation)?;
        }
        Ok(())
    }

    pub fn apply_layout(&mut self, frames: &[LayoutFrame]) -> Result<(), BackendError> {
        for frame in frames {
            frame
                .validate()
                .map_err(|err| BackendError::BatchRejected(format!("invalid layout frame: {err:?}")))?;
            if !self.nodes.contains_key(&frame.id) {
                return Err(BackendError::BatchRejected(format!(
                    "layout references unknown node id {}",
                    frame.id
                )));
            }
            self.pending_layout.push(*frame);
        }
        Ok(())
    }

    pub fn flush<A>(&mut self, adapter: &mut A) -> Result<(), BackendError>
    where
        A: PlatformAdapter<Handle = H>,
    {
        for frame in self.pending_layout.drain(..) {
            let handle = self
                .nodes
                .get(&frame.id)
                .map(|node| node.handle)
                .ok_or_else(|| {
                    BackendError::BatchRejected(format!(
                        "layout references unknown node id {}",
                        frame.id
                    ))
                })?;
            adapter.apply_frame(handle, frame)?;
            self.frames.insert(frame.id, frame);
        }
        adapter.flush()
    }

    pub fn drain_events<A>(&mut self, adapter: &mut A) -> Vec<UiEvent>
    where
        A: PlatformAdapter<Handle = H>,
    {
        adapter.drain_events()
    }

    #[cfg(test)]
    pub fn node(&self, id: UiNodeId) -> Option<&NodeRecord<H>> {
        self.nodes.get(&id)
    }

    fn apply_mutation<A>(&mut self, adapter: &mut A, mutation: &Mutation) -> Result<(), BackendError>
    where
        A: PlatformAdapter<Handle = H>,
    {
        match mutation {
            Mutation::CreateNode { id, kind } => self.create_node(adapter, *id, *kind, None),
            Mutation::CreateTextNode { id, text } => {
                self.create_node(adapter, *id, ElementKind::Text, Some(text.clone()))
            }
            Mutation::SetText { id, text } => self.set_text(adapter, *id, text),
            Mutation::SetProp { id, key, value } => self.set_prop(adapter, *id, *key, value),
            Mutation::InsertChild {
                parent,
                child,
                index,
            } => self.insert_child(adapter, *parent, *child, *index as usize),
            Mutation::MoveNode {
                id,
                new_parent,
                index,
            } => self.move_node(adapter, *id, *new_parent, *index as usize),
            Mutation::ReplaceNode { old, new_id, kind } => {
                self.replace_node(adapter, *old, *new_id, *kind)
            }
            Mutation::RemoveNode { id } => self.remove_node(adapter, *id),
            Mutation::AttachEventListener { id, event } => {
                self.attach_listener(adapter, *id, *event)
            }
        }
    }

    fn create_node<A>(
        &mut self,
        adapter: &mut A,
        id: UiNodeId,
        kind: ElementKind,
        text: Option<String>,
    ) -> Result<(), BackendError>
    where
        A: PlatformAdapter<Handle = H>,
    {
        if self.nodes.contains_key(&id) {
            return Err(BackendError::BatchRejected(format!("duplicate node id {id}")));
        }

        let handle = adapter.create_view(kind, text.as_deref())?;
        self.nodes.insert(id, NodeRecord::new(kind, handle, text));

        if self.root_id.is_none() {
            self.root_id = Some(id);
            adapter.attach_root(id, handle)?;
        }

        Ok(())
    }

    fn set_text<A>(
        &mut self,
        adapter: &mut A,
        id: UiNodeId,
        text: &str,
    ) -> Result<(), BackendError>
    where
        A: PlatformAdapter<Handle = H>,
    {
        let node = self.node_mut(id)?;
        if !supports_text(node.kind) {
            return Err(BackendError::BatchRejected(format!(
                "set_text is unsupported for {:?}",
                node.kind
            )));
        }
        node.text = Some(text.to_string());
        adapter.set_text(node.kind, node.handle, text)
    }

    fn set_prop<A>(
        &mut self,
        adapter: &mut A,
        id: UiNodeId,
        key: PropKey,
        value: &PropValue,
    ) -> Result<(), BackendError>
    where
        A: PlatformAdapter<Handle = H>,
    {
        let node = self.node_mut(id)?;
        node.props.insert(key, value.clone());
        adapter.set_prop(node.kind, node.handle, &node.props, key)
    }

    fn insert_child<A>(
        &mut self,
        adapter: &mut A,
        parent: UiNodeId,
        child: UiNodeId,
        index: usize,
    ) -> Result<(), BackendError>
    where
        A: PlatformAdapter<Handle = H>,
    {
        if !self.nodes.contains_key(&parent) {
            return Err(BackendError::BatchRejected(format!("unknown parent node id {parent}")));
        }
        if !self.nodes.contains_key(&child) {
            return Err(BackendError::BatchRejected(format!("unknown child node id {child}")));
        }
        if child == parent {
            return Err(BackendError::BatchRejected("node cannot be its own parent".to_string()));
        }
        if self.root_id == Some(child) {
            return Err(BackendError::BatchRejected(
                "root node cannot become a child".to_string(),
            ));
        }

        let parent_kind = self.nodes[&parent].kind;
        if !accepts_children(parent_kind) {
            return Err(BackendError::BatchRejected(format!(
                "{parent_kind:?} cannot accept children"
            )));
        }
        if self.nodes[&child].parent.is_some() {
            return Err(BackendError::BatchRejected(format!(
                "node {child} already has a parent"
            )));
        }
        if self.would_cycle(parent, child) {
            return Err(BackendError::BatchRejected(format!(
                "inserting node {child} under {parent} would create a cycle"
            )));
        }

        let parent_handle = self.nodes[&parent].handle;
        let child_handle = self.nodes[&child].handle;
        let insert_at = index.min(self.nodes[&parent].children.len());

        adapter.insert_child(parent_handle, child, child_handle, insert_at)?;
        self.node_mut(child)?.parent = Some(parent);
        self.node_mut(parent)?.children.insert(insert_at, child);
        Ok(())
    }

    fn move_node<A>(
        &mut self,
        adapter: &mut A,
        id: UiNodeId,
        new_parent: UiNodeId,
        index: usize,
    ) -> Result<(), BackendError>
    where
        A: PlatformAdapter<Handle = H>,
    {
        if self.root_id == Some(id) {
            return Err(BackendError::BatchRejected(
                "root node cannot be moved".to_string(),
            ));
        }

        let old_parent = self
            .nodes
            .get(&id)
            .ok_or_else(|| BackendError::BatchRejected(format!("unknown node id {id}")))?
            .parent
            .ok_or_else(|| BackendError::BatchRejected(format!("node {id} has no parent")))?;

        if !self.nodes.contains_key(&new_parent) {
            return Err(BackendError::BatchRejected(format!(
                "unknown parent node id {new_parent}"
            )));
        }
        if !accepts_children(self.nodes[&new_parent].kind) {
            return Err(BackendError::BatchRejected(format!(
                "{:?} cannot accept children",
                self.nodes[&new_parent].kind
            )));
        }
        if self.would_cycle(new_parent, id) {
            return Err(BackendError::BatchRejected(format!(
                "moving node {id} under {new_parent} would create a cycle"
            )));
        }

        let handle = self.nodes[&id].handle;
        let old_parent_handle = self.nodes[&old_parent].handle;
        let new_parent_handle = self.nodes[&new_parent].handle;

        adapter.remove_child(old_parent_handle, handle)?;
        if let Some(position) = self.nodes[&old_parent]
            .children
            .iter()
            .position(|child| *child == id)
        {
            self.node_mut(old_parent)?.children.remove(position);
        }

        let insert_at = index.min(self.nodes[&new_parent].children.len());
        adapter.insert_child(new_parent_handle, id, handle, insert_at)?;
        self.node_mut(id)?.parent = Some(new_parent);
        self.node_mut(new_parent)?.children.insert(insert_at, id);
        Ok(())
    }

    fn replace_node<A>(
        &mut self,
        adapter: &mut A,
        old: UiNodeId,
        new_id: UiNodeId,
        kind: ElementKind,
    ) -> Result<(), BackendError>
    where
        A: PlatformAdapter<Handle = H>,
    {
        if self.nodes.contains_key(&new_id) {
            return Err(BackendError::BatchRejected(format!("duplicate node id {new_id}")));
        }
        let old_record = self
            .nodes
            .get(&old)
            .cloned()
            .ok_or_else(|| BackendError::BatchRejected(format!("unknown node id {old}")))?;
        let replace_index = old_record.parent.and_then(|parent_id| {
            self.nodes
                .get(&parent_id)
                .and_then(|parent| parent.children.iter().position(|child| *child == old))
        });

        let new_handle = adapter.create_view(kind, None)?;
        self.remove_subtree(adapter, old)?;
        self.nodes
            .insert(new_id, NodeRecord::new(kind, new_handle, None));

        match old_record.parent {
            Some(parent_id) => {
                let index = replace_index.unwrap_or(self.nodes[&parent_id].children.len());
                let insert_at = index.min(self.nodes[&parent_id].children.len());
                let parent_handle = self.nodes[&parent_id].handle;
                adapter.insert_child(parent_handle, new_id, new_handle, insert_at)?;
                self.node_mut(new_id)?.parent = Some(parent_id);
                self.node_mut(parent_id)?.children.insert(insert_at, new_id);
            }
            None => {
                self.root_id = Some(new_id);
                adapter.attach_root(new_id, new_handle)?;
            }
        }

        Ok(())
    }

    fn remove_node<A>(&mut self, adapter: &mut A, id: UiNodeId) -> Result<(), BackendError>
    where
        A: PlatformAdapter<Handle = H>,
    {
        if self.root_id == Some(id) {
            return Err(BackendError::BatchRejected(
                "root node cannot be removed without replacement".to_string(),
            ));
        }
        self.remove_subtree(adapter, id)
    }

    fn attach_listener<A>(
        &mut self,
        adapter: &mut A,
        id: UiNodeId,
        event: EventKind,
    ) -> Result<(), BackendError>
    where
        A: PlatformAdapter<Handle = H>,
    {
        let node = self.node_mut(id)?;
        if !node.listeners.contains(&event) {
            node.listeners.push(event);
        }
        adapter.attach_listener(node.kind, node.handle, id, event)
    }

    fn remove_subtree<A>(&mut self, adapter: &mut A, id: UiNodeId) -> Result<(), BackendError>
    where
        A: PlatformAdapter<Handle = H>,
    {
        let node = self
            .nodes
            .get(&id)
            .cloned()
            .ok_or_else(|| BackendError::BatchRejected(format!("unknown node id {id}")))?;

        for child in node.children.clone() {
            self.remove_subtree(adapter, child)?;
        }

        if let Some(parent_id) = node.parent {
            let parent_handle = self.nodes[&parent_id].handle;
            adapter.remove_child(parent_handle, node.handle)?;
            if let Some(position) = self.nodes[&parent_id]
                .children
                .iter()
                .position(|child| *child == id)
            {
                self.node_mut(parent_id)?.children.remove(position);
            }
        } else if self.root_id == Some(id) {
            adapter.detach_root(id, node.handle)?;
            self.root_id = None;
        }

        adapter.remove_view(id, node.handle, &node.listeners)?;
        self.nodes.remove(&id);
        self.frames.remove(&id);
        self.pending_layout.retain(|frame| frame.id != id);
        Ok(())
    }

    fn would_cycle(&self, parent: UiNodeId, child: UiNodeId) -> bool {
        let mut cursor = Some(parent);
        while let Some(current) = cursor {
            if current == child {
                return true;
            }
            cursor = self.nodes.get(&current).and_then(|node| node.parent);
        }
        false
    }

    fn node_mut(&mut self, id: UiNodeId) -> Result<&mut NodeRecord<H>, BackendError> {
        self.nodes
            .get_mut(&id)
            .ok_or_else(|| BackendError::BatchRejected(format!("unknown node id {id}")))
    }
}

fn accepts_children(kind: ElementKind) -> bool {
    matches!(kind, ElementKind::Stack | ElementKind::List)
}

fn supports_text(kind: ElementKind) -> bool {
    matches!(kind, ElementKind::Text | ElementKind::Button | ElementKind::Input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockAdapter {
        next_handle: usize,
        events: Vec<UiEvent>,
    }

    impl PlatformAdapter for MockAdapter {
        type Handle = usize;

        fn create_view(
            &mut self,
            _kind: ElementKind,
            _text: Option<&str>,
        ) -> Result<Self::Handle, BackendError> {
            let handle = self.next_handle;
            self.next_handle += 1;
            Ok(handle)
        }

        fn attach_root(
            &mut self,
            _node_id: UiNodeId,
            _handle: Self::Handle,
        ) -> Result<(), BackendError> {
            Ok(())
        }

        fn detach_root(
            &mut self,
            _node_id: UiNodeId,
            _handle: Self::Handle,
        ) -> Result<(), BackendError> {
            Ok(())
        }

        fn insert_child(
            &mut self,
            _parent: Self::Handle,
            _child_id: UiNodeId,
            _child: Self::Handle,
            _index: usize,
        ) -> Result<(), BackendError> {
            Ok(())
        }

        fn remove_child(
            &mut self,
            _parent: Self::Handle,
            _child: Self::Handle,
        ) -> Result<(), BackendError> {
            Ok(())
        }

        fn remove_view(
            &mut self,
            _node_id: UiNodeId,
            _handle: Self::Handle,
            _listeners: &[EventKind],
        ) -> Result<(), BackendError> {
            Ok(())
        }

        fn set_text(
            &mut self,
            _kind: ElementKind,
            _handle: Self::Handle,
            _text: &str,
        ) -> Result<(), BackendError> {
            Ok(())
        }

        fn set_prop(
            &mut self,
            _kind: ElementKind,
            _handle: Self::Handle,
            _props: &HashMap<PropKey, PropValue>,
            _key: PropKey,
        ) -> Result<(), BackendError> {
            Ok(())
        }

        fn attach_listener(
            &mut self,
            _kind: ElementKind,
            _handle: Self::Handle,
            node_id: UiNodeId,
            event: EventKind,
        ) -> Result<(), BackendError> {
            if event == EventKind::Tap {
                self.events.push(UiEvent::Tap { id: node_id });
            }
            Ok(())
        }

        fn apply_frame(
            &mut self,
            _handle: Self::Handle,
            _frame: LayoutFrame,
        ) -> Result<(), BackendError> {
            Ok(())
        }

        fn flush(&mut self) -> Result<(), BackendError> {
            Ok(())
        }

        fn drain_events(&mut self) -> Vec<UiEvent> {
            std::mem::take(&mut self.events)
        }
    }

    #[test]
    fn insert_move_and_replace_preserve_tree_invariants() {
        let mut state = ExecutorState::default();
        let mut adapter = MockAdapter::default();

        state
            .apply_mutations(
                &mut adapter,
                &[
                    Mutation::CreateNode {
                        id: 1,
                        kind: ElementKind::Stack,
                    },
                    Mutation::CreateNode {
                        id: 2,
                        kind: ElementKind::Stack,
                    },
                    Mutation::CreateTextNode {
                        id: 3,
                        text: "hello".to_string(),
                    },
                    Mutation::InsertChild {
                        parent: 1,
                        child: 2,
                        index: 0,
                    },
                    Mutation::InsertChild {
                        parent: 2,
                        child: 3,
                        index: 0,
                    },
                    Mutation::MoveNode {
                        id: 3,
                        new_parent: 1,
                        index: 1,
                    },
                    Mutation::ReplaceNode {
                        old: 2,
                        new_id: 4,
                        kind: ElementKind::Image,
                    },
                ],
            )
            .unwrap();

        assert_eq!(state.node(1).unwrap().children, vec![4, 3]);
        assert_eq!(state.node(3).unwrap().parent, Some(1));
        assert_eq!(state.node(4).unwrap().parent, Some(1));
        assert!(state.node(2).is_none());
    }

    #[test]
    fn invalid_parent_and_root_removal_are_rejected() {
        let mut state = ExecutorState::default();
        let mut adapter = MockAdapter::default();

        state
            .apply_mutations(
                &mut adapter,
                &[
                    Mutation::CreateNode {
                        id: 1,
                        kind: ElementKind::Stack,
                    },
                    Mutation::CreateTextNode {
                        id: 2,
                        text: "hello".to_string(),
                    },
                ],
            )
            .unwrap();

        let insert_result = state.apply_mutations(
            &mut adapter,
            &[Mutation::InsertChild {
                parent: 2,
                child: 1,
                index: 0,
            }],
        );
        assert!(matches!(
            insert_result,
            Err(BackendError::BatchRejected(_))
        ));

        let remove_result = state.apply_mutations(
            &mut adapter,
            &[Mutation::RemoveNode { id: 1 }],
        );
        assert!(matches!(
            remove_result,
            Err(BackendError::BatchRejected(_))
        ));
    }

    #[test]
    fn set_text_rejects_non_text_nodes() {
        let mut state = ExecutorState::default();
        let mut adapter = MockAdapter::default();

        state
            .apply_mutations(
                &mut adapter,
                &[Mutation::CreateNode {
                    id: 1,
                    kind: ElementKind::Image,
                }],
            )
            .unwrap();

        let result = state.apply_mutations(
            &mut adapter,
            &[Mutation::SetText {
                id: 1,
                text: "oops".to_string(),
            }],
        );

        assert!(matches!(result, Err(BackendError::BatchRejected(_))));
    }

    #[test]
    fn layout_requires_known_nodes_and_events_drain_through_adapter() {
        let mut state = ExecutorState::default();
        let mut adapter = MockAdapter::default();

        state
            .apply_mutations(
                &mut adapter,
                &[
                    Mutation::CreateNode {
                        id: 1,
                        kind: ElementKind::Stack,
                    },
                    Mutation::CreateNode {
                        id: 2,
                        kind: ElementKind::Button,
                    },
                    Mutation::InsertChild {
                        parent: 1,
                        child: 2,
                        index: 0,
                    },
                    Mutation::AttachEventListener {
                        id: 2,
                        event: EventKind::Tap,
                    },
                ],
            )
            .unwrap();

        state
            .apply_layout(&[LayoutFrame {
                id: 2,
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 40.0,
            }])
            .unwrap();
        state.flush(&mut adapter).unwrap();

        assert_eq!(state.drain_events(&mut adapter), vec![UiEvent::Tap { id: 2 }]);

        let layout_result = state.apply_layout(&[LayoutFrame {
            id: 99,
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        }]);
        assert!(matches!(
            layout_result,
            Err(BackendError::BatchRejected(_))
        ));
    }
}
