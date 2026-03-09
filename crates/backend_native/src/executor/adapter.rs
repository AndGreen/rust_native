use std::collections::HashMap;

use backend_api::BackendError;
use native_schema::{ElementKind, EventKind, LayoutFrame, PropKey, PropValue, UiEvent, UiNodeId};

pub trait PlatformAdapter {
    type Handle: Copy + Eq;

    fn create_view(
        &mut self,
        kind: ElementKind,
        text: Option<&str>,
    ) -> Result<Self::Handle, BackendError>;
    fn attach_root(&mut self, node_id: UiNodeId, handle: Self::Handle) -> Result<(), BackendError>;
    fn detach_root(&mut self, node_id: UiNodeId, handle: Self::Handle) -> Result<(), BackendError>;
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
    fn apply_frame(&mut self, handle: Self::Handle, frame: LayoutFrame)
        -> Result<(), BackendError>;
    fn flush(&mut self) -> Result<(), BackendError>;
    fn drain_events(&mut self) -> Vec<UiEvent>;
}
