use mf_core::{IntoView, View, WithChildren};
use mf_widgets::prelude::*;
use native_schema::{LayoutFrame, Mutation, PropValue};
use vdom_runtime::{HostSize, RenderBatch, VdomRuntime};

const TEST_HOST: HostSize = HostSize::new(390.0, 844.0);

#[test]
fn counter_initial_mount_snapshot_matches_fixture() {
    let mut runtime = VdomRuntime::new();
    let batch = runtime.render(&counter_view(0), TEST_HOST);

    assert_snapshot("counter_initial", &format_batch("counter_initial", &batch));
}

#[test]
fn counter_update_snapshot_matches_fixture() {
    let mut runtime = VdomRuntime::new();
    let _ = runtime.render(&counter_view(0), TEST_HOST);
    let batch = runtime.render(&counter_view(1), TEST_HOST);

    assert_snapshot("counter_update", &format_batch("counter_update", &batch));
}

#[test]
fn album_list_initial_snapshot_matches_fixture() {
    let mut runtime = VdomRuntime::new();
    let batch = runtime.render(&album_list_view(&albums_base()), TEST_HOST);

    assert_snapshot(
        "album_list_initial",
        &format_batch("album_list_initial", &batch),
    );
}

#[test]
fn album_list_append_item_snapshot_matches_fixture() {
    let mut runtime = VdomRuntime::new();
    let _ = runtime.render(&album_list_view(&albums_base()), TEST_HOST);
    let batch = runtime.render(&album_list_view(&albums_with_append()), TEST_HOST);

    assert_snapshot(
        "album_list_append_item",
        &format_batch("album_list_append_item", &batch),
    );
}

fn counter_view(count: i32) -> View {
    SafeArea()
        .background(Color::hex_or_black("#FAF6F1"))
        .alignment(Alignment::Center)
        .justify_content(JustifyContent::Center)
        .with_children(vec![VStack()
            .spacing(12.0)
            .padding(16.0)
            .alignment(Alignment::Center)
            .with_children(vec![
                Text(format!("Count: {count}"))
                    .font(Font::bold(24.0))
                    .color(Color::primary())
                    .into_view(),
                HStack()
                    .spacing(8.0)
                    .with_children(vec![
                        Button("−")
                            .background(Color::hex_or_black("#D14A42"))
                            .foreground(Color::hex_or_black("#F6F0EB"))
                            .corner_radius(12.0)
                            .on_click(|| {})
                            .into_view(),
                        Button("+")
                            .background(Color::hex_or_black("#248C61"))
                            .foreground(Color::hex_or_black("#F6F0EB"))
                            .corner_radius(12.0)
                            .on_click(|| {})
                            .into_view(),
                    ])
                    .into_view(),
            ])
            .into_view()])
}

fn album_list_view(albums: &[AlbumFixture]) -> View {
    SafeArea().with_children(vec![VStack()
        .spacing(16.0)
        .padding(24.0)
        .with_children(vec![
            Text("Albums").font(Font::bold(32.0)).into_view(),
            List(albums.iter().cloned(), |album: AlbumFixture| {
                let title = album.title.to_string();
                let artist = album.artist.to_string();
                let cover = album.cover.to_string();
                HStack()
                    .spacing(12.0)
                    .padding(8.0)
                    .with_children(vec![
                        Image(cover).size(60.0, 60.0).corner_radius(8.0).into_view(),
                        VStack()
                            .alignment(Alignment::Leading)
                            .with_children(vec![
                                Text(title).font(Font::semibold(18.0)).into_view(),
                                Text(artist).foreground(Color::secondary()).into_view(),
                            ])
                            .into_view(),
                        Button("Like").on_click(|| {}).into_view(),
                    ])
                    .into_view()
            })
            .into_view(),
        ])
        .into_view()])
}

#[derive(Clone)]
struct AlbumFixture {
    title: &'static str,
    artist: &'static str,
    cover: &'static str,
}

fn albums_base() -> Vec<AlbumFixture> {
    vec![
        AlbumFixture {
            title: "Explorations",
            artist: "Nova Collective",
            cover: "explorations.jpg",
        },
        AlbumFixture {
            title: "Analog Dreams",
            artist: "Chromatic",
            cover: "analog_dreams.jpg",
        },
        AlbumFixture {
            title: "Signal Flow",
            artist: "Greyline",
            cover: "signal_flow.jpg",
        },
    ]
}

fn albums_with_append() -> Vec<AlbumFixture> {
    let mut albums = albums_base();
    albums.push(AlbumFixture {
        title: "Northern Lights",
        artist: "Static Bloom",
        cover: "northern_lights.jpg",
    });
    albums
}

fn assert_snapshot(name: &str, actual: &str) {
    let expected = match name {
        "counter_initial" => include_str!("fixtures/counter_initial.snap"),
        "counter_update" => include_str!("fixtures/counter_update.snap"),
        "album_list_initial" => include_str!("fixtures/album_list_initial.snap"),
        "album_list_append_item" => include_str!("fixtures/album_list_append_item.snap"),
        other => panic!("unknown snapshot fixture: {other}"),
    };

    assert_eq!(actual, expected, "snapshot mismatch for {name}");
}

fn format_batch(name: &str, batch: &RenderBatch) -> String {
    let mut lines = vec![
        format!("snapshot: {name}"),
        format!("protocol: {:?}", batch.protocol_version),
        "mutations:".to_string(),
    ];
    for mutation in &batch.mutations {
        lines.push(format!("  {}", format_mutation(mutation)));
    }
    lines.push("layout:".to_string());
    for frame in &batch.layout {
        lines.push(format!("  {}", format_layout(frame)));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn format_mutation(mutation: &Mutation) -> String {
    match mutation {
        Mutation::CreateNode { id, kind } => format!("CreateNode id={id} kind={kind:?}"),
        Mutation::CreateTextNode { id, text } => {
            format!("CreateTextNode id={id} text={text:?}")
        }
        Mutation::SetText { id, text } => format!("SetText id={id} text={text:?}"),
        Mutation::SetProp { id, key, value } => {
            format!(
                "SetProp id={id} key={key:?} value={}",
                format_prop_value(value)
            )
        }
        Mutation::UnsetProp { id, key } => format!("UnsetProp id={id} key={key:?}"),
        Mutation::InsertChild {
            parent,
            child,
            index,
        } => format!("InsertChild parent={parent} child={child} index={index}"),
        Mutation::MoveNode {
            id,
            new_parent,
            index,
        } => format!("MoveNode id={id} new_parent={new_parent} index={index}"),
        Mutation::ReplaceNode { old, new_id, kind } => {
            format!("ReplaceNode old={old} new_id={new_id} kind={kind:?}")
        }
        Mutation::RemoveNode { id } => format!("RemoveNode id={id}"),
        Mutation::AttachEventListener { id, event } => {
            format!("AttachEventListener id={id} event={event:?}")
        }
    }
}

fn format_prop_value(value: &PropValue) -> String {
    match value {
        PropValue::String(value) => format!("{value:?}"),
        PropValue::Bool(value) => value.to_string(),
        PropValue::Float(value) => format_float(*value),
        PropValue::Color(color) => format!(
            "Color(r={}, g={}, b={}, a={})",
            format_float(color.r),
            format_float(color.g),
            format_float(color.b),
            format_float(color.a)
        ),
        PropValue::Axis(axis) => format!("Axis({axis:?})"),
        PropValue::Alignment(alignment) => format!("Alignment({alignment:?})"),
        PropValue::JustifyContent(justify_content) => {
            format!("JustifyContent({justify_content:?})")
        }
        PropValue::SafeAreaEdges(edges) => format!("SafeAreaEdges({edges:?})"),
        PropValue::FontWeight(weight) => format!("FontWeight({weight:?})"),
        PropValue::Insets(insets) => format!(
            "Insets(top={}, right={}, bottom={}, left={})",
            format_float(insets.top),
            format_float(insets.right),
            format_float(insets.bottom),
            format_float(insets.left)
        ),
        PropValue::Dimension(dimension) => format!("Dimension({dimension:?})"),
        PropValue::CornerRadii(radii) => format!(
            "CornerRadii(top_left={}, top_right={}, bottom_right={}, bottom_left={})",
            format_float(radii.top_left),
            format_float(radii.top_right),
            format_float(radii.bottom_right),
            format_float(radii.bottom_left)
        ),
        PropValue::LineStyle(style) => format!(
            "LineStyle(width={}, color=({}, {}, {}, {}))",
            format_float(style.width),
            format_float(style.color.r),
            format_float(style.color.g),
            format_float(style.color.b),
            format_float(style.color.a)
        ),
        PropValue::Shadow(shadow) => format!(
            "Shadow(color=({}, {}, {}, {}), radius={}, offset=({}, {}))",
            format_float(shadow.color.r),
            format_float(shadow.color.g),
            format_float(shadow.color.b),
            format_float(shadow.color.a),
            format_float(shadow.radius),
            format_float(shadow.offset.x),
            format_float(shadow.offset.y)
        ),
        PropValue::Point(point) => {
            format!(
                "Point(x={}, y={})",
                format_float(point.x),
                format_float(point.y)
            )
        }
    }
}

fn format_layout(frame: &LayoutFrame) -> String {
    format!(
        "LayoutFrame id={} x={} y={} width={} height={}",
        frame.id,
        format_float(frame.x),
        format_float(frame.y),
        format_float(frame.width),
        format_float(frame.height)
    )
}

fn format_float(value: f32) -> String {
    format!("{value:.1}")
}
