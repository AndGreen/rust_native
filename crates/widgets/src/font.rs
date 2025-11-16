#[derive(Debug, Clone)]
pub struct Font {
    pub size: f32,
    pub weight: FontWeight,
}

impl Font {
    pub fn new(size: f32, weight: FontWeight) -> Self {
        Self { size, weight }
    }

    pub fn regular(size: f32) -> Self {
        Self::new(size, FontWeight::Regular)
    }

    pub fn bold(size: f32) -> Self {
        Self::new(size, FontWeight::Bold)
    }

    pub fn semibold(size: f32) -> Self {
        Self::new(size, FontWeight::SemiBold)
    }
}

#[derive(Debug, Clone)]
pub enum FontWeight {
    Regular,
    SemiBold,
    Bold,
}
