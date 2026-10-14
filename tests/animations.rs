//! Guards the animation clips authored in the Tiled map.
//!
//! The clips are data, so a typo in `assets/Main.tmx` would only surface at
//! runtime. These tests parse the map the same way the game does and assert the
//! Warrior's clips are present and well-formed.

use std::collections::HashMap;

use tiled::{LayerType, PropertyValue};

/// The `Animations` object layer of `assets/Main.tmx`, keyed by clip name.
fn load_clips() -> HashMap<String, HashMap<String, PropertyValue>> {
    let map = tiled::Loader::new()
        .load_tmx_map("assets/Main.tmx")
        .expect("assets/Main.tmx should load");

    let layer = map
        .layers()
        .find_map(|layer| match layer.layer_type() {
            LayerType::Objects(objects) if layer.name == "Animations" => Some(objects),
            _ => None,
        })
        .expect("an `Animations` object layer should exist");

    layer
        .objects()
        .map(|object| (object.name.clone(), object.properties.clone()))
        .collect()
}

fn int(clip: &HashMap<String, PropertyValue>, key: &str) -> i32 {
    match clip.get(key) {
        Some(PropertyValue::IntValue(value)) => *value,
        other => panic!("`{key}` should be an int, got {other:?}"),
    }
}

fn boolean(clip: &HashMap<String, PropertyValue>, key: &str) -> bool {
    match clip.get(key) {
        Some(PropertyValue::BoolValue(value)) => *value,
        other => panic!("`{key}` should be a bool, got {other:?}"),
    }
}

fn file(clip: &HashMap<String, PropertyValue>, key: &str) -> String {
    match clip.get(key) {
        Some(PropertyValue::FileValue(value) | PropertyValue::StringValue(value)) => value.clone(),
        other => panic!("`{key}` should be a file, got {other:?}"),
    }
}

#[test]
fn warrior_clips_are_defined_in_tiled() {
    let clips = load_clips();

    // clip name -> (frame count, looping)
    let expected = [
        ("Idle", 8, true),
        ("Run", 6, true),
        ("Guard", 6, true),
        ("Attack1", 4, false),
        ("Attack2", 4, false),
    ];

    for (name, frames, looping) in expected {
        let clip = clips
            .get(name)
            .unwrap_or_else(|| panic!("clip `{name}` should be defined"));

        assert_eq!(int(clip, "first"), 0, "`{name}` first frame");
        assert_eq!(int(clip, "last"), frames - 1, "`{name}` last frame");
        assert_eq!(int(clip, "columns"), frames, "`{name}` columns");
        assert_eq!(int(clip, "frame_width"), 192, "`{name}` frame width");
        assert_eq!(int(clip, "frame_height"), 192, "`{name}` frame height");
        assert_eq!(boolean(clip, "looping"), looping, "`{name}` looping");
        assert!(file(clip, "image").ends_with(".png"), "`{name}` image");
    }
}

#[test]
fn every_clip_references_an_existing_sheet() {
    for (name, clip) in load_clips() {
        let path = format!("assets/{}", file(&clip, "image"));
        assert!(
            std::path::Path::new(&path).exists(),
            "clip `{name}` references missing sheet `{path}`",
        );
    }
}
