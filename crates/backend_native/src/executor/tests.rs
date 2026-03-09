use std::collections::HashMap;

use backend_api::BackendError;
use native_schema::{
    ElementKind, EventKind, LayoutFrame, Mutation, PropKey, PropValue, UiEvent, UiNodeId,
};

use super::{ExecutorState, PlatformAdapter};

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

    let remove_result = state.apply_mutations(&mut adapter, &[Mutation::RemoveNode { id: 1 }]);
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
