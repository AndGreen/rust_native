use std::collections::HashMap;

use backend_api::BackendError;
use native_schema::{ElementKind, EventKind, LayoutFrame, Mutation, PropKey, PropValue, UiEvent, UiNodeId};

use super::PlatformAdapter;

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
    pub(crate) fn new(kind: ElementKind, handle: H, text: Option<String>) -> Self {
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
    pub(crate) root_id: Option<UiNodeId>,
    pub(crate) nodes: HashMap<UiNodeId, NodeRecord<H>>,
    pub(crate) frames: HashMap<UiNodeId, LayoutFrame>,
    pub(crate) pending_layout: Vec<LayoutFrame>,
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
    pub fn apply_mutations<A>(
        &mut self,
        adapter: &mut A,
        mutations: &[Mutation],
    ) -> Result<(), BackendError>
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
            frame.validate().map_err(|err| {
                BackendError::BatchRejected(format!("invalid layout frame: {err:?}"))
            })?;
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

    pub(crate) fn node_mut(&mut self, id: UiNodeId) -> Result<&mut NodeRecord<H>, BackendError> {
        self.nodes
            .get_mut(&id)
            .ok_or_else(|| BackendError::BatchRejected(format!("unknown node id {id}")))
    }
}
