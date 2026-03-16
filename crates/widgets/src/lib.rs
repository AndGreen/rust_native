pub mod button;
pub mod color;
pub mod container;
pub mod font;
pub mod image;
pub mod input;
pub mod layout;
pub mod list;
pub mod safe_area;
pub mod text;

mod builders;

pub use builders::*;
pub use button::ButtonView;
pub use color::Color;
pub use container::Container;
pub use font::{Font, FontWeight};
pub use image::ImageView;
pub use input::InputView;
pub use layout::{Alignment, HStack, VStack};
pub use list::ListView;
pub use native_schema::{EdgeInsets, JustifyContent};
pub use safe_area::SafeArea;
pub use text::TextView;

pub mod prelude {
    pub use crate::button::ButtonView;
    pub use crate::color::Color;
    pub use crate::container::Container as ContainerView;
    pub use crate::font::{Font, FontWeight};
    pub use crate::image::ImageView;
    pub use crate::input::InputView;
    pub use crate::layout::{Alignment, HStack as HStackLayout, VStack as VStackLayout};
    pub use crate::list::ListView;
    pub use crate::safe_area::SafeArea as SafeAreaView;
    pub use crate::text::TextView;
    pub use crate::EdgeInsets;
    pub use crate::JustifyContent;
    pub use crate::{Button, Container, HStack, Image, Input, List, SafeArea, Text, VStack};
    pub use mf_core::dsl::{IntoView, WithChildren};
}
