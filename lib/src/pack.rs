use anyhow::{Context, anyhow, bail};
use rayon::prelude::*;
use rdev::Key;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    u32,
};

use anyhow::Result;
use kira::sound::static_sound::StaticSoundData;

#[derive(Deserialize)]
struct MechvibesPack {
    name: String,
    defines: HashMap<String, Option<String>>,
}

#[derive(Serialize, Deserialize)]
struct RawPack {
    creator: String,
    source: String,
    default_volume: String,
    keys: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Pack {
    name: String,
    default_volume: u32,
    pub keys: HashMap<String, StaticSoundData>,
}

impl Pack {
    pub fn load_from(folder: &PathBuf, pack_name: &str) -> Result<Self> {
        let path = Path::new(&folder).join(pack_name);
        let config = match fs::read_to_string(path.join("config.json5")) {
            Ok(config) => config,
            Err(_) => fs::read_to_string(path.join("config.json"))
                .with_context(|| format!("No config file at path {}", path.display()))?,
        };
        let parsed_config: RawPack =
            json5::from_str(&config).map_err(|e| anyhow!("Invalid configuration file: {e}"))?;

        let pack_keys = parsed_config
            .keys
            .par_iter()
            .map(|(key, value)| {
                let filepath = path.join(value);

                let sound_data =
                    StaticSoundData::from_file(filepath.as_path()).with_context(|| {
                        format!(
                            "Failed to load sound for key '{key}' from '{}'",
                            filepath.as_path().display()
                        )
                    })?;

                Ok((key.into(), sound_data))
            })
            .collect::<anyhow::Result<_>>()?;

        let pack = Pack {
            name: pack_name.to_owned(),
            default_volume: parsed_config.default_volume.parse()?,
            keys: pack_keys,
        };

        Ok(pack)
    }
}

pub fn list_installed(path: &PathBuf) -> Result<Vec<String>> {
    let items = fs::read_dir(path)?;

    let subdirs: Vec<OsString> = items
        .filter_map(|d| {
            let entry = d.ok()?;
            let path = entry.path();
            if path.is_dir() {
                Some(entry.file_name())
            } else {
                None
            }
        })
        .collect();

    let mut packs: Vec<String> = Vec::new();
    for dir in &subdirs {
        let path = Path::new(&path).join(dir);
        let files = fs::read_dir(&path).unwrap();
        let filesnames = files
            .filter_map(|f| {
                let entry = f.ok()?;
                let path = entry.path();
                if path.is_file() {
                    Some(entry.file_name())
                } else {
                    None
                }
            })
            .collect::<Vec<OsString>>();
        let has_config_file = filesnames.contains(&OsString::from("config.json"))
            || filesnames.contains(&OsString::from("config.json5"));
        if has_config_file {
            packs.push(dir.to_str().unwrap().to_owned());
        }
    }

    Ok(packs)
}

pub fn from_mechvibes(path: &Path) -> Result<()> {
    let config_path = path.join("config.json");

    let Ok(config) = fs::read_to_string(&config_path) else {
        bail!("Config file not found at path '{}'", path.display());
    };

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
        default_volume: "0".to_string(),
        keys,
    };

    let serialized = serde_json::to_string_pretty(&pack)?;

    fs::rename(&config_path, config_path.with_extension("json.bak"))?;
    fs::write(path.join("config.json5"), serialized)?;

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
