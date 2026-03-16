use std::error::Error;
use std::fmt;
use std::sync::Once;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorParseError {
    MissingHash,
    InvalidLength,
    InvalidHex,
}

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingHash => f.write_str("hex color must start with '#'"),
            Self::InvalidLength => f.write_str("hex color must be #RRGGBB or #RRGGBBAA"),
            Self::InvalidHex => f.write_str("hex color contains invalid hex digits"),
        }
    }
}

impl Error for ColorParseError {}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn hex(value: &str) -> Result<Self, ColorParseError> {
        let hex = value
            .strip_prefix('#')
            .ok_or(ColorParseError::MissingHash)?;

        let (rgb, alpha) = match hex.len() {
            6 => (hex, 255),
            8 => (&hex[..6], parse_hex_byte(&hex[6..8])?),
            _ => return Err(ColorParseError::InvalidLength),
        };

        Ok(Self {
            r: parse_hex_byte(&rgb[0..2])? as f32 / 255.0,
            g: parse_hex_byte(&rgb[2..4])? as f32 / 255.0,
            b: parse_hex_byte(&rgb[4..6])? as f32 / 255.0,
            a: alpha as f32 / 255.0,
        })
    }

    pub fn hex_or_black(value: &str) -> Self {
        match Self::hex(value) {
            Ok(color) => color,
            Err(err) => {
                if cfg!(any(debug_assertions, test)) {
                    // TODO move to utils crate
                    static INVALID_HEX_WARNING: Once = Once::new();
                    INVALID_HEX_WARNING.call_once(|| {
                        eprintln!(
                            "[mf_widgets/color] invalid hex color {:?}: {}; falling back to black",
                            value, err
                        );
                    });
                }
                Self::new(0.0, 0.0, 0.0)
            }
        }
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

impl From<Color> for native_schema::ColorValue {
    fn from(value: Color) -> Self {
        native_schema::ColorValue::new(value.r, value.g, value.b, value.a)
    }
}

fn parse_hex_byte(value: &str) -> Result<u8, ColorParseError> {
    u8::from_str_radix(value, 16).map_err(|_| ColorParseError::InvalidHex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rgb_hex_with_implicit_opaque_alpha() {
        let color = Color::hex("#D14A42").expect("valid hex color");

        assert_eq!(color, Color::new(209.0 / 255.0, 74.0 / 255.0, 66.0 / 255.0));
    }

    #[test]
    fn parses_rgba_hex_with_explicit_alpha() {
        let color = Color::hex("#248C61CC").expect("valid hex color");

        assert_eq!(
            color,
            Color {
                r: 36.0 / 255.0,
                g: 140.0 / 255.0,
                b: 97.0 / 255.0,
                a: 204.0 / 255.0,
            }
        );
    }

    #[test]
    fn parses_lowercase_and_uppercase_hex_digits() {
        let uppercase = Color::hex("#F6F0EB").expect("valid uppercase hex color");
        let lowercase = Color::hex("#f6f0eb").expect("valid lowercase hex color");

        assert_eq!(uppercase, lowercase);
    }

    #[test]
    fn hex_or_black_returns_same_color_for_valid_hex() {
        assert_eq!(
            Color::hex_or_black("#248C61CC"),
            Color::hex("#248C61CC").expect("valid hex color")
        );
    }

    #[test]
    fn hex_or_black_falls_back_to_black_for_invalid_hex() {
        assert_eq!(Color::hex_or_black("#oops"), Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn hex_or_black_keeps_returning_black_on_repeated_invalid_values() {
        assert_eq!(Color::hex_or_black("#oops"), Color::new(0.0, 0.0, 0.0));
        assert_eq!(Color::hex_or_black("still bad"), Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn rejects_missing_hash_prefix() {
        let err = Color::hex("D14A42").expect_err("missing hash must fail");

        assert_eq!(err, ColorParseError::MissingHash);
    }

    #[test]
    fn rejects_invalid_hex_length() {
        let err = Color::hex("#ABC").expect_err("short hex must fail");

        assert_eq!(err, ColorParseError::InvalidLength);
    }

    #[test]
    fn rejects_invalid_hex_digits() {
        let err = Color::hex("#GGGGGG").expect_err("non-hex digits must fail");

        assert_eq!(err, ColorParseError::InvalidHex);
    }
}
