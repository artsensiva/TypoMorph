//! Bounded physical-key mapping using explicitly selected Linux XKB layouts.
//!
//! This module does not collect input, identify eligible fields, classify words,
//! or change application text. Callers must establish those safety boundaries.
//! Compose sequences and dead keys deliberately have no conversion here.

use std::{collections::BTreeMap, fmt, path::Path};
use xkbcommon::xkb;

pub const SUPPORTED_LAYOUTS: [&str; 8] = ["us", "gb", "ru", "ua", "de", "fr", "es", "latam"];
pub const MAX_STROKES: usize = 32;
const MAX_TEXT_BYTES: usize = 128;

/// A physical key with a complete modifier snapshot, not a text-derived guess.
#[derive(Clone, Copy, Default)]
pub struct Stroke {
    pub evdev_code: u16,
    pub shift: bool,
    pub caps_lock: bool,
    pub alt_gr: bool,
    /// Control, ordinary Alt, Super, or any modifier not represented above.
    pub unsupported_modifier: bool,
}

impl fmt::Debug for Stroke {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Stroke([redacted])")
    }
}

/// An in-memory proposal only. Neither string should be logged or persisted.
pub struct Conversion {
    pub original: String,
    pub replacement: String,
}

impl fmt::Debug for Conversion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Conversion([redacted])")
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum MappingError {
    #[error("invalid layout selection")]
    InvalidSelection,
    #[error("system keymap unavailable")]
    Unavailable,
    #[error("layout is not selected")]
    NotSelected,
    #[error("invalid conversion range")]
    InvalidRange,
    #[error("unsupported key or modifier")]
    UnsupportedInput,
}

/// Compiles only the approved, explicitly selected default layout families.
/// System definitions must also match the active installed layout before use.
/// User include paths and XKB_DEFAULT_* environment overrides are ignored.
pub struct Keymaps {
    maps: BTreeMap<String, xkb::Keymap>,
}

impl Keymaps {
    pub fn new(selected: &[&str]) -> Result<Self, MappingError> {
        if selected.is_empty()
            || selected.len() > SUPPORTED_LAYOUTS.len()
            || selected
                .iter()
                .enumerate()
                .any(|(i, id)| !SUPPORTED_LAYOUTS.contains(id) || selected[..i].contains(id))
        {
            return Err(MappingError::InvalidSelection);
        }
        let mut context =
            xkb::Context::new(xkb::CONTEXT_NO_DEFAULT_INCLUDES | xkb::CONTEXT_NO_ENVIRONMENT_NAMES);
        if !context.include_path_append(Path::new("/usr/share/X11/xkb")) {
            return Err(MappingError::Unavailable);
        }
        let mut maps = BTreeMap::new();
        for id in selected {
            let map = xkb::Keymap::new_from_names(
                &context,
                "evdev",
                "pc105",
                id,
                "",
                Some(String::new()),
                xkb::KEYMAP_COMPILE_NO_FLAGS,
            )
            .ok_or(MappingError::Unavailable)?;
            maps.insert((*id).to_owned(), map);
        }
        Ok(Self { maps })
    }

    /// Maps a bounded sequence without retaining it. Any unsupported stroke
    /// refuses the entire proposal. Whitespace is a boundary, not part of a word.
    pub fn convert(
        &self,
        source: &str,
        target: &str,
        strokes: &[Stroke],
    ) -> Result<Conversion, MappingError> {
        if strokes.is_empty() || strokes.len() > MAX_STROKES || source == target {
            return Err(MappingError::InvalidRange);
        }
        let source = self.maps.get(source).ok_or(MappingError::NotSelected)?;
        let target = self.maps.get(target).ok_or(MappingError::NotSelected)?;
        let mut original = String::with_capacity(MAX_TEXT_BYTES);
        let mut replacement = String::with_capacity(MAX_TEXT_BYTES);
        let mut source_state = xkb::State::new(source);
        let mut target_state = xkb::State::new(target);
        for stroke in strokes {
            original.push(map_stroke(source, &mut source_state, stroke)?);
            replacement.push(map_stroke(target, &mut target_state, stroke)?);
        }
        // Each stroke produces at most one Unicode scalar (four UTF-8 bytes).
        debug_assert!(original.len() <= MAX_TEXT_BYTES && replacement.len() <= MAX_TEXT_BYTES);
        Ok(Conversion {
            original,
            replacement,
        })
    }
}

fn modifier(map: &xkb::Keymap, enabled: bool, name: &str) -> Result<u32, MappingError> {
    if !enabled {
        return Ok(0);
    }
    1u32.checked_shl(map.mod_get_index(name))
        .ok_or(MappingError::UnsupportedInput)
}

fn map_stroke(
    map: &xkb::Keymap,
    state: &mut xkb::State,
    stroke: &Stroke,
) -> Result<char, MappingError> {
    if stroke.unsupported_modifier || stroke.evdev_code == 0 || stroke.evdev_code > 255 {
        return Err(MappingError::UnsupportedInput);
    }
    let depressed = modifier(map, stroke.shift, xkb::MOD_NAME_SHIFT)?
        | modifier(map, stroke.alt_gr, xkb::MOD_NAME_ISO_LEVEL3_SHIFT)?;
    let locked = modifier(map, stroke.caps_lock, xkb::MOD_NAME_CAPS)?;
    state.update_mask(depressed, 0, locked, 0, 0, 0);
    let key = xkb::Keycode::new(u32::from(stroke.evdev_code) + 8);
    if state.key_get_syms(key).len() != 1 {
        return Err(MappingError::UnsupportedInput);
    }
    let ch = char::from_u32(state.key_get_utf32(key))
        .filter(|c| !c.is_control() && !c.is_whitespace())
        .ok_or(MappingError::UnsupportedInput)?;
    Ok(ch)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strokes(codes: &[u16]) -> Vec<Stroke> {
        codes
            .iter()
            .map(|&evdev_code| Stroke {
                evdev_code,
                ..Stroke::default()
            })
            .collect()
    }

    #[test]
    fn all_eight_selected_layouts_use_native_definitions() {
        let maps = Keymaps::new(&SUPPORTED_LAYOUTS).unwrap();
        let expected = [
            ("us", "q"),
            ("gb", "q"),
            ("ru", "й"),
            ("ua", "й"),
            ("de", "q"),
            ("fr", "a"),
            ("es", "q"),
            ("latam", "q"),
        ];
        for (source, original) in expected {
            for (target, replacement) in expected {
                if source == target {
                    continue;
                }
                let result = maps.convert(source, target, &strokes(&[16])).unwrap();
                assert_eq!(result.original, original);
                assert_eq!(result.replacement, replacement);
            }
        }
        let result = maps
            .convert("us", "ru", &strokes(&[34, 35, 48, 32, 20, 49]))
            .unwrap();
        assert_eq!(result.original, "ghbdtn");
        assert_eq!(result.replacement, "привет");
    }

    #[test]
    fn regional_letters_and_punctuation() {
        let maps = Keymaps::new(&SUPPORTED_LAYOUTS).unwrap();
        for (id, code, expected) in [
            ("ua", 40, "є"),
            ("de", 21, "z"),
            ("fr", 2, "&"),
            ("es", 39, "ñ"),
            ("latam", 39, "ñ"),
        ] {
            assert_eq!(
                maps.convert("us", id, &strokes(&[code]))
                    .unwrap()
                    .replacement,
                expected
            );
        }
        let shift_two = Stroke {
            evdev_code: 3,
            shift: true,
            ..Stroke::default()
        };
        let result = maps.convert("us", "gb", &[shift_two]).unwrap();
        assert_eq!(result.original, "@");
        assert_eq!(result.replacement, "\"");
        let alt_gr_q = Stroke {
            evdev_code: 16,
            alt_gr: true,
            ..Stroke::default()
        };
        assert_eq!(
            maps.convert("de", "latam", &[alt_gr_q]).unwrap().original,
            "@"
        );
    }

    #[test]
    fn modifiers_are_snapshots_and_do_not_leak_between_keys() {
        let maps = Keymaps::new(&["us", "ru"]).unwrap();
        for (shift, caps_lock, expected) in [
            (false, false, "п"),
            (true, false, "П"),
            (false, true, "П"),
            (true, true, "п"),
        ] {
            let keys = [
                Stroke {
                    evdev_code: 34,
                    shift,
                    caps_lock,
                    ..Stroke::default()
                },
                Stroke {
                    evdev_code: 34,
                    ..Stroke::default()
                },
            ];
            assert_eq!(
                maps.convert("us", "ru", &keys).unwrap().replacement,
                format!("{expected}п")
            );
        }
    }

    #[test]
    fn selection_and_input_refusals_are_content_free() {
        for selected in [
            vec![],
            vec!["us", "us"],
            vec!["us", "us(intl)"],
            vec!["unknown"],
            vec!["ru\0"],
        ] {
            assert!(matches!(
                Keymaps::new(&selected),
                Err(MappingError::InvalidSelection)
            ));
        }
        let maps = Keymaps::new(&["us", "de"]).unwrap();
        assert_eq!(
            maps.convert("us", "ru", &strokes(&[34])).unwrap_err(),
            MappingError::NotSelected
        );
        for keys in [vec![], strokes(&[34; 33])] {
            assert_eq!(
                maps.convert("us", "de", &keys).unwrap_err(),
                MappingError::InvalidRange
            );
        }
        for code in [0, 1, 14, 15, 28, 42, 57, 256, u16::MAX, 13] {
            // German '=' physical position is a dead acute key, also refused.
            assert_eq!(
                maps.convert("us", "de", &strokes(&[34, code])).unwrap_err(),
                MappingError::UnsupportedInput
            );
        }
        let shortcut = Stroke {
            evdev_code: 34,
            unsupported_modifier: true,
            ..Stroke::default()
        };
        assert_eq!(
            maps.convert("us", "de", &[shortcut]).unwrap_err(),
            MappingError::UnsupportedInput
        );
        assert_eq!(
            maps.convert("us", "us", &strokes(&[34])).unwrap_err(),
            MappingError::InvalidRange
        );
        assert_eq!(
            maps.convert("us", "de", &strokes(&[34; 32]))
                .unwrap()
                .replacement
                .len(),
            32
        );
    }

    #[test]
    fn debug_output_redacts_text_and_physical_keys() {
        let keys = strokes(&[34, 35, 48, 32, 20, 49]);
        assert_eq!(format!("{:?}", keys[0]), "Stroke([redacted])");
        let result = Keymaps::new(&["us", "ru"])
            .unwrap()
            .convert("us", "ru", &keys)
            .unwrap();
        assert_eq!(format!("{result:?}"), "Conversion([redacted])");
    }
}
