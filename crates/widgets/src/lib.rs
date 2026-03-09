pub mod button;
pub mod color;
pub mod font;
pub mod image;
pub mod layout;
pub mod list;
pub mod safe_area;
pub mod text;

mod builders;

pub use builders::*;
pub use button::ButtonView;
pub use color::Color;
pub use font::{Font, FontWeight};
pub use image::ImageView;
pub use layout::{Alignment, HStack, VStack};
pub use list::ListView;
pub use safe_area::SafeArea;
pub use text::TextView;

pub mod prelude {
    pub use crate::button::ButtonView;
    pub use crate::color::Color;
    pub use crate::font::{Font, FontWeight};
    pub use crate::image::ImageView;
    pub use crate::layout::{Alignment, HStack as HStackLayout, VStack as VStackLayout};
    pub use crate::list::ListView;
    pub use crate::safe_area::SafeArea as SafeAreaView;
    pub use crate::text::TextView;
    pub use crate::{Button, HStack, Image, List, SafeArea, Text, VStack};
    pub use mf_core::dsl::{IntoView, WithChildren};
}
