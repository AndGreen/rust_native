use std::collections::HashMap;

use mf_core::View;
use mf_widgets::button::ButtonAction;
use native_schema::{LayoutFrame, ProtocolVersion, UiEvent, UiNodeId};

use crate::layout::compute_layout_frames;
use crate::mutations::{diff_node, emit_create_subtree};
use crate::tree::{
    canonicalize_view, collect_tap_handlers, flatten_children, is_fragment, CanonicalNode,
    NodeDescriptor,
};
use crate::types::{HostSize, RenderBatch};

pub struct VdomRuntime {
    current: Option<CanonicalNode>,
    current_layout: Vec<LayoutFrame>,
    next_id: UiNodeId,
    tap_handlers: HashMap<UiNodeId, ButtonAction>,
    force_full_resync: bool,
}

impl Default for VdomRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl VdomRuntime {
    pub fn new() -> Self {
        Self {
            current: None,
            current_layout: Vec::new(),
            next_id: 1,
            tap_handlers: HashMap::new(),
            force_full_resync: false,
        }
    }

    pub fn render(&mut self, view: &View, host_size: HostSize) -> RenderBatch {
        let previous = if self.force_full_resync {
            None
        } else {
            self.current.clone()
        };

        let Some(next) = self.build_root(previous.as_ref(), view) else {
            self.current = None;
            self.current_layout.clear();
            self.tap_handlers.clear();
            self.force_full_resync = false;
            return RenderBatch::default();
        };

        let mut mutations = Vec::new();
        match previous.as_ref() {
            None => emit_create_subtree(&next, &mut mutations),
            Some(previous_root) => diff_node(previous_root, &next, &mut mutations),
        }

        let next_layout = compute_layout_frames(&next, host_size);
        let layout_changed = self.force_full_resync || self.current_layout != next_layout;

        self.tap_handlers = collect_tap_handlers(&next);
        self.current = Some(next);
        self.current_layout = next_layout.clone();
        self.force_full_resync = false;

        RenderBatch {
            protocol_version: ProtocolVersion::V1,
            mutations,
            layout: if layout_changed {
                next_layout
            } else {
                Vec::new()
            },
        }
    }

    pub fn dispatch_event(&self, event: UiEvent) {
        if let UiEvent::Tap { id } = event {
            if let Some(handler) = self.tap_handlers.get(&id) {
                handler();
            }
        }
    }

    pub fn request_full_resync(&mut self) {
        self.force_full_resync = true;
    }

    fn build_root(
        &mut self,
        previous: Option<&CanonicalNode>,
        view: &View,
    ) -> Option<CanonicalNode> {
        if is_fragment(view) {
            let children = flatten_children(view.children());
            let first = children.first()?;
            return Some(self.build_node(previous, first));
        }

        Some(self.build_node(previous, view))
    }

    fn build_node(&mut self, previous: Option<&CanonicalNode>, view: &View) -> CanonicalNode {
        let descriptor = NodeDescriptor::from_view(view);
        let id = if previous
            .map(|node| node.descriptor == descriptor)
            .unwrap_or(false)
        {
            previous.expect("previous node").id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        };

        let previous_children = previous.map(|node| node.children.as_slice()).unwrap_or(&[]);
        let mut children = Vec::new();
        let flat_children = flatten_children(view.children());
        for (index, child_view) in flat_children.into_iter().enumerate() {
            let prior = previous_children
                .get(index)
                .filter(|child| child.descriptor == NodeDescriptor::from_view(child_view));
            children.push(self.build_node(prior, child_view));
        }

        canonicalize_view(id, view, children)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use mf_core::{IntoView, View, WithChildren};
    use mf_widgets::prelude::*;
    use native_schema::{ElementKind, EventKind, Mutation, UiEvent};

    use super::VdomRuntime;
    use crate::types::{HostSize, RenderBatch};

    const TEST_HOST: HostSize = HostSize::new(390.0, 844.0);

    fn render(runtime: &mut VdomRuntime, view: View) -> RenderBatch {
        runtime.render(&view, TEST_HOST)
    }

    #[test]
    fn initial_mount_emits_create_insert_and_layout() {
        let mut runtime = VdomRuntime::new();
        let batch = render(
            &mut runtime,
            VStack()
                .spacing(12.0)
                .padding(16.0)
                .with_children(vec![Text("Hello").into_view(), Button("Tap").into_view()]),
        );

        assert!(matches!(
            batch.mutations.first(),
            Some(Mutation::CreateNode {
                kind: ElementKind::Stack,
                ..
            })
        ));
        assert!(batch
            .mutations
            .iter()
            .any(|mutation| matches!(mutation, Mutation::CreateTextNode { .. })));
        assert!(batch
            .mutations
            .iter()
            .any(|mutation| matches!(mutation, Mutation::InsertChild { .. })));
        assert_eq!(batch.layout.len(), 3);
        assert_eq!(batch.layout[0].width, TEST_HOST.width);
        assert_eq!(batch.layout[0].height, TEST_HOST.height);
    }

    #[test]
    fn text_update_emits_only_set_text_and_no_layout_delta_when_frames_match() {
        let mut runtime = VdomRuntime::new();
        let first = Text("Count: 0").into_view();
        let second = Text("Count: 1").into_view();

        let _ = runtime.render(&first, TEST_HOST);
        let batch = runtime.render(&second, TEST_HOST);

        assert_eq!(batch.mutations.len(), 1);
        assert!(matches!(batch.mutations[0], Mutation::SetText { .. }));
        assert!(batch.layout.is_empty());
    }

    #[test]
    fn removed_props_force_replace() {
        let mut runtime = VdomRuntime::new();
        let first = VStack().with_children(vec![Text("Hello")
            .font(mf_widgets::Font::bold(24.0))
            .into_view()]);
        let second = VStack().with_children(vec![Text("Hello").into_view()]);

        let _ = runtime.render(&first, TEST_HOST);
        let batch = runtime.render(&second, TEST_HOST);

        assert!(matches!(batch.mutations[0], Mutation::ReplaceNode { .. }));
        assert!(!batch.layout.is_empty());
    }

    #[test]
    fn button_tap_dispatch_invokes_handler() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = Arc::clone(&calls);
        let view = Button("Tap")
            .on_click(move || {
                calls_for_handler.fetch_add(1, Ordering::Relaxed);
            })
            .into_view();

        let mut runtime = VdomRuntime::new();
        let batch = runtime.render(&view, TEST_HOST);
        let button_id = batch
            .mutations
            .iter()
            .find_map(|mutation| match mutation {
                Mutation::AttachEventListener {
                    id,
                    event: EventKind::Tap,
                } => Some(*id),
                _ => None,
            })
            .expect("button id");

        runtime.dispatch_event(UiEvent::Tap { id: button_id });
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn nested_layout_is_parent_first_and_covers_every_node() {
        let mut runtime = VdomRuntime::new();
        let batch = render(
            &mut runtime,
            VStack().spacing(16.0).padding(24.0).with_children(vec![
                Text("Albums").font(Font::bold(32.0)).into_view(),
                HStack().spacing(12.0).padding(8.0).with_children(vec![
                    Image("cover.jpg").size(60.0, 60.0).into_view(),
                    VStack()
                        .alignment(mf_widgets::Alignment::Leading)
                        .with_children(vec![
                            Text("Explorations").font(Font::semibold(18.0)).into_view(),
                            Text("Nova Collective").into_view(),
                        ]),
                    Button("Like").into_view(),
                ]),
            ]),
        );

        assert_eq!(batch.layout.len(), 8);
        assert_eq!(batch.layout[0].x, 0.0);
        assert_eq!(batch.layout[0].y, 0.0);
        assert!(batch.layout.windows(2).all(|pair| pair[0].id != pair[1].id));
    }

    #[test]
    fn safe_area_mount_emits_safe_area_node_and_offsets_child_layout() {
        let mut runtime = VdomRuntime::new();
        let host = HostSize::with_safe_area(
            390.0,
            844.0,
            native_schema::EdgeInsets::new(59.0, 0.0, 34.0, 0.0),
        );
        let view = SafeArea().with_children(vec![Text("Albums").into_view()]);

        let batch = runtime.render(&view, host);

        assert!(batch.mutations.iter().any(|mutation| matches!(
            mutation,
            Mutation::CreateNode {
                kind: ElementKind::SafeArea,
                ..
            }
        )));
        assert_eq!(batch.layout[1].y, 59.0);
    }

    #[test]
    fn safe_area_keeps_child_container_full_width() {
        let mut runtime = VdomRuntime::new();
        let host = HostSize::with_safe_area(
            390.0,
            844.0,
            native_schema::EdgeInsets::new(59.0, 0.0, 34.0, 0.0),
        );
        let view = SafeArea().with_children(vec![VStack()
            .spacing(12.0)
            .padding(16.0)
            .with_children(vec![
                Text("Count: 2").font(Font::bold(24.0)).into_view(),
                HStack()
                    .spacing(8.0)
                    .with_children(vec![Button("-").into_view(), Button("+").into_view()])
                    .into_view(),
            ])
            .into_view()]);

        let batch = runtime.render(&view, host);

        assert_eq!(batch.layout[1].x, 0.0);
        assert_eq!(batch.layout[1].y, 59.0);
        assert_eq!(batch.layout[1].width, 390.0);
    }
}
