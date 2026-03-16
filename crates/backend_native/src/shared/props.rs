use std::collections::HashMap;

use native_schema::{
    ColorValue, CornerRadii, FontWeight, LineStyle, PointValue, PropKey, PropValue, ShadowStyle,
};

pub(crate) fn color(
    props: &HashMap<PropKey, PropValue>,
    key: PropKey,
) -> Option<Result<ColorValue, &'static str>> {
    props.get(&key).map(|value| match value {
        PropValue::Color(color) => Ok(*color),
        _ => Err("invalid color prop"),
    })
}

pub(crate) fn font(
    props: &HashMap<PropKey, PropValue>,
    default_size: f32,
) -> Result<(f32, FontWeight), &'static str> {
    let size = match props.get(&PropKey::FontSize) {
        Some(PropValue::Float(size)) => *size,
        Some(_) => return Err("invalid FontSize prop"),
        None => default_size,
    };
    let weight = match props.get(&PropKey::FontWeight) {
        Some(PropValue::FontWeight(weight)) => *weight,
        Some(_) => return Err("invalid FontWeight prop"),
        None => FontWeight::Regular,
    };

    Ok((size, weight))
}

pub(crate) fn float(
    props: &HashMap<PropKey, PropValue>,
    key: PropKey,
) -> Option<Result<f32, &'static str>> {
    props.get(&key).map(|value| match value {
        PropValue::Float(value) => Ok(*value),
        _ => Err("invalid float prop"),
    })
}

pub(crate) fn bool_value(
    props: &HashMap<PropKey, PropValue>,
    key: PropKey,
) -> Option<Result<bool, &'static str>> {
    props.get(&key).map(|value| match value {
        PropValue::Bool(value) => Ok(*value),
        _ => Err("invalid bool prop"),
    })
}

pub(crate) fn string(
    props: &HashMap<PropKey, PropValue>,
    key: PropKey,
) -> Option<Result<&str, &'static str>> {
    props.get(&key).map(|value| match value {
        PropValue::String(value) => Ok(value.as_str()),
        _ => Err("invalid string prop"),
    })
}

#[allow(dead_code)]
pub(crate) fn line_style(
    props: &HashMap<PropKey, PropValue>,
    key: PropKey,
) -> Option<Result<LineStyle, &'static str>> {
    props.get(&key).map(|value| match value {
        PropValue::LineStyle(value) => Ok(*value),
        _ => Err("invalid line style prop"),
    })
}

#[allow(dead_code)]
pub(crate) fn shadow(
    props: &HashMap<PropKey, PropValue>,
    key: PropKey,
) -> Option<Result<ShadowStyle, &'static str>> {
    props.get(&key).map(|value| match value {
        PropValue::Shadow(value) => Ok(*value),
        _ => Err("invalid shadow prop"),
    })
}

#[allow(dead_code)]
pub(crate) fn point(
    props: &HashMap<PropKey, PropValue>,
    key: PropKey,
) -> Option<Result<PointValue, &'static str>> {
    props.get(&key).map(|value| match value {
        PropValue::Point(value) => Ok(*value),
        _ => Err("invalid point prop"),
    })
}

#[allow(dead_code)]
pub(crate) fn corner_radii(
    props: &HashMap<PropKey, PropValue>,
    key: PropKey,
) -> Option<Result<CornerRadii, &'static str>> {
    props.get(&key).map(|value| match value {
        PropValue::CornerRadii(value) => Ok(*value),
        _ => Err("invalid corner radii prop"),
    })
}
