use anyhow::Context;
use rayon::prelude::*;
use rdev::Key;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

use anyhow::Result;
use kira::sound::static_sound::StaticSoundData;

#[derive(Serialize, Deserialize)]
struct RawPack {
    creator: String,
    source: String,
    default_volume: String,
    keys: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Pack {
    pub name: String,
    pub default_volume: u32,
    pub keys: HashMap<String, StaticSoundData>,
}

impl Pack {
    pub fn load_from(folder: &Path, pack_name: &str) -> Result<Self> {
        let path = folder.join(pack_name);

        let config = Self::read_config_file(&path)?;
        let parsed_config: RawPack = json5::from_str(&config)
            .with_context(|| format!("Invalid configuration file in {}", path.display()))?;

        let pack_keys = parsed_config
            .keys
            .par_iter()
            .map(|(key, value)| {
                let filepath = path.join(value);

                let sound_data = StaticSoundData::from_file(&filepath).with_context(|| {
                    format!(
                        "Failed to load sound for key '{key}' from '{}'",
                        filepath.display()
                    )
                })?;

                Ok((key.to_owned(), sound_data))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        let default_volume = parsed_config.default_volume.parse().with_context(|| {
            format!("Invalid default_volume: '{}'", parsed_config.default_volume)
        })?;

        Ok(Pack {
            name: pack_name.to_owned(),
            default_volume,
            keys: pack_keys,
        })
    }

    fn read_config_file(path: &Path) -> Result<String> {
        fs::read_to_string(path.join("config.json5"))
            .or_else(|_| fs::read_to_string(path.join("config.json")))
            .with_context(|| format!("No config file found at path {}", path.display()))
    }
}

/// Create a new pack folder inside `base_path` and populate it with a default
/// `config.json5` copied from the embedded template.
pub fn create_new_pack(base_path: &Path) -> Result<()> {
    // Find an available folder name: "New pack", "New pack (2)", ...
    let mut folder_name = String::from("New pack");
    let mut candidate_path = base_path.join(&folder_name);
    let mut counter: u32 = 1;

    while candidate_path.exists() {
        counter += 1;
        folder_name = format!("New pack ({counter})");
        candidate_path = base_path.join(&folder_name);
    }

    fs::create_dir_all(&candidate_path).with_context(|| {
        format!(
            "Failed to create pack directory at {}",
            candidate_path.display()
        )
    })?;

    // Embed the template at compile time and write it as config.json5
    let template_contents = include_str!("config_template.json5");
    fs::write(candidate_path.join("config.json5"), template_contents).with_context(|| {
        format!(
            "Failed to write config.json5 into {}",
            candidate_path.display()
        )
    })?;

    Ok(())
}

pub fn list_installed(path: &Path) -> Result<Vec<String>> {
    let entries = fs::read_dir(path)
        .with_context(|| format!("Failed to read directory: {}", path.display()))?;

    let mut packs = Vec::new();

    for entry in entries {
        let entry = entry.with_context(|| "Failed to read directory entry")?;
        let entry_path = entry.path();

        if !entry_path.is_dir() {
            continue;
        }

        // Check if directory contains a config file
        if has_config_file(&entry_path)? {
            let dir_name = entry.file_name();
            let pack_name = dir_name.to_string_lossy().into_owned();
            packs.push(pack_name);
        }
    }

    Ok(packs)
}

fn has_config_file(dir_path: &Path) -> Result<bool> {
    Ok(dir_path.join("config.json5").exists() || dir_path.join("config.json").exists())
}

pub fn from_mechvibes(path: &Path) -> Result<()> {
    #[derive(Deserialize)]
    struct MechvibesPack {
        defines: HashMap<String, Option<String>>,
    }

    let config_path = path.join("config.json");

    let config = fs::read_to_string(&config_path)
        .with_context(|| format!("Config file not found at path '{}'", path.display()))?;

    let parsed: MechvibesPack = serde_json::from_str(&config)
        .with_context(|| format!("Config at path '{}' is not valid", path.display()))?;

    let keys: HashMap<String, String> = parsed
        .defines
        .into_iter()
        .filter_map(|(key, value)| {
            let value = value?;

            let keycode = key.parse::<u16>().ok()?;
            let key = key_from_code(keycode);

            let key_str = match key {
                Key::Unknown(_) => String::from("Unknown"),
                _ => format!("{key:?}"),
            };

            Some((key_str, value))
        })
        .collect();

    if !keys.contains_key("Unknown") {
        eprintln!(
            "WARNING: No unknown key found in the config. \
            This means that the keylogger will not be able to \
            detect unknown keys. And will probably crash \
            Please add a key named \"unknown\" to your config."
        );
    }

    let pack = RawPack {
        creator: String::new(),
        source: String::new(),
        default_volume: "50".to_string(),
        keys,
    };

    let serialized =
        serde_json::to_string_pretty(&pack).context("Failed to serialize pack configuration")?;

    let backup_path = config_path.with_extension("json.bak");
    fs::rename(&config_path, &backup_path)
        .with_context(|| format!("Failed to create backup at {}", backup_path.display()))?;

    fs::write(path.join("config.json5"), serialized).context("Failed to write new config file")?;

    Ok(())
}

// from https://github.com/hainguyents13/mechvibes/blob/master/src/libs/keycodes.js
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn key_from_code(code: u16) -> Key {
    match code {
        1 => Key::Escape,
        59 => Key::F1,
        60 => Key::F2,
        61 => Key::F3,
        62 => Key::F4,
        63 => Key::F5,
        64 => Key::F6,
        65 => Key::F7,
        66 => Key::F8,
        67 => Key::F9,
        68 => Key::F10,
        87 => Key::F11,
        88 => Key::F12,

        41 => Key::BackQuote,

        2 => Key::Num1,
        3 => Key::Num2,
        4 => Key::Num3,
        5 => Key::Num4,
        6 => Key::Num5,
        7 => Key::Num6,
        8 => Key::Num7,
        9 => Key::Num8,
        10 => Key::Num9,
        11 => Key::Num0,

        12 => Key::Minus,
        13 | 3597 => Key::Equal,
        14 => Key::Backspace,

        15 => Key::Tab,
        58 => Key::CapsLock,

        30 => Key::KeyA,
        48 => Key::KeyB,
        46 => Key::KeyC,
        32 => Key::KeyD,
        18 => Key::KeyE,
        33 => Key::KeyF,
        34 => Key::KeyG,
        35 => Key::KeyH,
        23 => Key::KeyI,
        36 => Key::KeyJ,
        37 => Key::KeyK,
        38 => Key::KeyL,
        50 => Key::KeyM,
        49 => Key::KeyN,
        24 => Key::KeyO,
        25 => Key::KeyP,
        16 => Key::KeyQ,
        19 => Key::KeyR,
        31 => Key::KeyS,
        20 => Key::KeyT,
        22 => Key::KeyU,
        47 => Key::KeyV,
        17 => Key::KeyW,
        45 => Key::KeyX,
        21 => Key::KeyY,
        44 => Key::KeyZ,

        26 => Key::LeftBracket,
        27 => Key::RightBracket,
        43 => Key::BackSlash,

        39 => Key::SemiColon,
        40 => Key::Quote,
        28 => Key::Return,

        51 => Key::Comma,
        52 | 83 => Key::Dot,
        53 => Key::Slash,

        57 => Key::Space,

        3639 => Key::PrintScreen,
        70 => Key::ScrollLock,
        3653 => Key::Pause,

        3636 | 61010 => Key::Insert,
        3667 | 61011 => Key::Delete,
        3655 | 60999 => Key::Home,
        3663 | 61007 => Key::End,
        3657 | 61001 => Key::PageUp,
        3665 | 61009 => Key::PageDown,

        57416 | 61000 => Key::UpArrow,
        57419 | 61003 => Key::LeftArrow,
        57421 | 61005 => Key::RightArrow,
        57424 | 61008 => Key::DownArrow,

        42 => Key::ShiftLeft,
        54 => Key::ShiftRight,
        29 => Key::ControlLeft,
        3613 => Key::ControlRight,
        56 => Key::Alt,
        3640 => Key::AltGr,
        3675 => Key::MetaLeft,
        3676 => Key::MetaRight,

        69 => Key::NumLock,
        3637 => Key::KpDivide,
        55 => Key::KpMultiply,
        74 => Key::KpMinus,
        78 => Key::KpPlus,
        3612 => Key::KpReturn,

        79 => Key::Kp1,
        80 => Key::Kp2,
        81 => Key::Kp3,
        75 => Key::Kp4,
        76 => Key::Kp5,
        77 => Key::Kp6,
        71 => Key::Kp7,
        72 => Key::Kp8,
        73 => Key::Kp9,
        82 => Key::Kp0,

        3666 => Key::Function,

        _ => Key::Unknown(code.into()),
    }
}
