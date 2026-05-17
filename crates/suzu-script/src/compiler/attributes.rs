use suzu_core::Color;

use crate::parser::Attribute;

use super::CompileError;

pub(super) fn required(
    command: &str,
    attributes: &[Attribute],
    key: &str,
) -> Result<String, CompileError> {
    optional(attributes, key)
        .map(ToOwned::to_owned)
        .ok_or_else(|| CompileError::MissingAttribute {
            command: command.to_owned(),
            key: key.to_owned(),
            span: None,
        })
}

pub(super) fn optional<'a>(attributes: &'a [Attribute], key: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|attribute| attribute.key == key)
        .map(|attribute| attribute.value.as_str())
}

pub(super) fn optional_u32(attributes: &[Attribute], key: &str) -> Option<u32> {
    optional(attributes, key)?.parse().ok()
}

pub(super) fn optional_i32(attributes: &[Attribute], key: &str) -> Option<i32> {
    optional(attributes, key)?.parse().ok()
}

pub(super) fn optional_f32(attributes: &[Attribute], key: &str) -> Option<f32> {
    optional(attributes, key)?.parse().ok()
}

pub(super) fn optional_bool(attributes: &[Attribute], key: &str) -> Option<bool> {
    optional(attributes, key)?.parse().ok()
}

pub(super) fn optional_color(attributes: &[Attribute], key: &str) -> Option<Color> {
    let value = optional(attributes, key)?.trim_start_matches('#');
    if value.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&value[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&value[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&value[4..6], 16).ok()? as f32 / 255.0;
    Some(Color::rgba(r, g, b, 1.0))
}
