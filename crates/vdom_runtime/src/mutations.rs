use native_schema::{ElementKind, EventKind, Mutation, UiNodeId};

use crate::tree::{prop_value, CanonicalNode, NodeDescriptor};

pub(crate) fn emit_create_subtree(node: &CanonicalNode, mutations: &mut Vec<Mutation>) {
    match node.descriptor {
        NodeDescriptor::Text => mutations.push(Mutation::CreateTextNode {
            id: node.id,
            text: node.text.clone().unwrap_or_default(),
        }),
        NodeDescriptor::Element(kind) => mutations.push(Mutation::CreateNode { id: node.id, kind }),
    }

    if let Some(text) = &node.text {
        if matches!(node.descriptor, NodeDescriptor::Element(_)) {
            mutations.push(Mutation::SetText {
                id: node.id,
                text: text.clone(),
            });
        }
    }

    for (key, value) in &node.props {
        mutations.push(Mutation::SetProp {
            id: node.id,
            key: *key,
            value: value.clone(),
        });
    }

    if node.tap_handler.is_some() {
        mutations.push(Mutation::AttachEventListener {
            id: node.id,
            event: EventKind::Tap,
        });
    }
    if node.input_handler.is_some() {
        mutations.push(Mutation::AttachEventListener {
            id: node.id,
            event: EventKind::TextInput,
        });
    }
    if node.focus_change_handler.is_some() {
        mutations.push(Mutation::AttachEventListener {
            id: node.id,
            event: EventKind::FocusChanged,
        });
    }

    for (index, child) in node.children.iter().enumerate() {
        emit_create_subtree(child, mutations);
        mutations.push(Mutation::InsertChild {
            parent: node.id,
            child: child.id,
            index: index as u32,
        });
    }
}

pub(crate) fn diff_node(
    previous: &CanonicalNode,
    next: &CanonicalNode,
    mutations: &mut Vec<Mutation>,
) {
    if previous.descriptor != next.descriptor {
        mutations.push(replace_mutation(previous.id, next));
        emit_replace_payload(next, mutations);
        return;
    }

    if listener_signature(previous) != listener_signature(next) || props_removed(previous, next) {
        mutations.push(replace_mutation(previous.id, next));
        emit_replace_payload(next, mutations);
        return;
    }

    if previous.text != next.text {
        if let Some(text) = &next.text {
            mutations.push(Mutation::SetText {
                id: next.id,
                text: text.clone(),
            });
        }
    }

    for (key, value) in &next.props {
        if prop_value(previous, *key) != Some(value) {
            mutations.push(Mutation::SetProp {
                id: next.id,
                key: *key,
                value: value.clone(),
            });
        }
    }

    let shared_len = previous.children.len().min(next.children.len());
    for index in 0..shared_len {
        diff_node(&previous.children[index], &next.children[index], mutations);
    }

    for child in previous.children.iter().skip(shared_len) {
        mutations.push(Mutation::RemoveNode { id: child.id });
    }

    for (index, child) in next.children.iter().enumerate().skip(shared_len) {
        emit_create_subtree(child, mutations);
        mutations.push(Mutation::InsertChild {
            parent: next.id,
            child: child.id,
            index: index as u32,
        });
    }
}

fn replace_mutation(old: UiNodeId, next: &CanonicalNode) -> Mutation {
    match next.descriptor {
        NodeDescriptor::Text => Mutation::ReplaceNode {
            old,
            new_id: next.id,
            kind: ElementKind::Text,
        },
        NodeDescriptor::Element(kind) => Mutation::ReplaceNode {
            old,
            new_id: next.id,
            kind,
        },
    }
}

fn emit_replace_payload(node: &CanonicalNode, mutations: &mut Vec<Mutation>) {
    if let Some(text) = &node.text {
        mutations.push(Mutation::SetText {
            id: node.id,
            text: text.clone(),
        });
    }

    for (key, value) in &node.props {
        mutations.push(Mutation::SetProp {
            id: node.id,
            key: *key,
            value: value.clone(),
        });
    }

    if node.tap_handler.is_some() {
        mutations.push(Mutation::AttachEventListener {
            id: node.id,
            event: EventKind::Tap,
        });
    }
    if node.input_handler.is_some() {
        mutations.push(Mutation::AttachEventListener {
            id: node.id,
            event: EventKind::TextInput,
        });
    }
    if node.focus_change_handler.is_some() {
        mutations.push(Mutation::AttachEventListener {
            id: node.id,
            event: EventKind::FocusChanged,
        });
    }

    for (index, child) in node.children.iter().enumerate() {
        emit_create_subtree(child, mutations);
        mutations.push(Mutation::InsertChild {
            parent: node.id,
            child: child.id,
            index: index as u32,
        });
    }
}

fn props_removed(previous: &CanonicalNode, next: &CanonicalNode) -> bool {
    previous
        .props
        .keys()
        .any(|key| prop_value(next, *key).is_none())
}

fn listener_signature(node: &CanonicalNode) -> (bool, bool, bool) {
    (
        node.tap_handler.is_some(),
        node.input_handler.is_some(),
        node.focus_change_handler.is_some(),
    )
}
