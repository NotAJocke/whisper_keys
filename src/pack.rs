use anyhow::anyhow;
use rayon::prelude::*;
use serde::Deserialize;
use std::{
    collections::HashMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use kira::sound::static_sound::StaticSoundData;

#[derive(Deserialize)]
struct RawPack {
    creator: String,
    source: String,
    default_volume: u32,
    keys: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Pack {
    name: String,
    default_volume: u32,
    keys: HashMap<String, StaticSoundData>,
}

impl Pack {
    pub fn load_from(folder: &PathBuf, pack_name: &str) -> Result<Self> {
        let path = Path::new(&folder).join(pack_name);
        let config = match fs::read_to_string(path.join("config.json5")) {
            Ok(config) => config,
            Err(_) => fs::read_to_string(path.join("config.json"))?,
        };
        let parsed_config: RawPack =
            json5::from_str(&config).map_err(|e| anyhow!("Invalid configuration file: {e}"))?;

        let pack_keys = parsed_config
            .keys
            .par_iter()
            .map(|(key, value)| {
                let filepath = path.join(value);

                let sound_data = StaticSoundData::from_file(filepath)?;

                Ok((key.into(), sound_data))
            })
            .collect::<anyhow::Result<_>>()?;

        let pack = Pack {
            name: pack_name.to_owned(),
            default_volume: parsed_config.default_volume,
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
