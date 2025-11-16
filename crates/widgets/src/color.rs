#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.a = alpha;
        self
    }

    pub fn primary() -> Self {
        Self::new(0.1, 0.1, 0.12)
    }

    pub fn secondary() -> Self {
        Self::new(0.4, 0.4, 0.48)
    }
}
