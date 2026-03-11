#![allow(non_snake_case)]
use crate::button::ButtonView;
use crate::image::ImageView;
use crate::input::InputView;
use crate::layout::{HStack, VStack};
use crate::list::ListView;
use crate::safe_area::SafeArea;
use crate::text::TextView;
use mf_core::View;

pub fn Text(content: impl Into<String>) -> TextView {
    TextView::new(content)
}

pub fn Button(label: impl Into<String>) -> ButtonView {
    ButtonView::new(label)
}

pub fn VStack() -> VStack {
    VStack::new()
}

pub fn HStack() -> HStack {
    HStack::new()
}

pub fn Image(source: impl Into<String>) -> ImageView {
    ImageView::new(source)
}

pub fn Input(value: impl Into<String>) -> InputView {
    InputView::new(value)
}

pub fn SafeArea() -> SafeArea {
    SafeArea::new()
}

pub fn List<I, F, Item>(items: I, builder: F) -> ListView
where
    I: IntoIterator<Item = Item>,
    F: Fn(Item) -> View,
{
    ListView::from_iterator(items, builder)
}
