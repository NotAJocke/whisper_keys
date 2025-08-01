use anyhow::{Result, bail};
use iced::widget::{Column, button, pick_list, row, slider, text};
use iced::{Element, Task};
use lib::audio_manager::{AudioManager, AudioMessage};
use lib::pack::Pack;
use std::path::PathBuf;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    thread,
};

fn main() -> Result<()> {
    let mut child = Command::new("./target/release/key_listener")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to launch helper");

    let stdout = child.stdout.take().expect("No stdout");
    let reader = BufReader::new(stdout);

    let audio_manager = AudioManager::new().unwrap();

    let Some(home) = std::env::home_dir() else {
        bail!("Couldn't get the user's home dir");
    };
    let packs_dir = home.join("WhisperKeys");

    let am = audio_manager.clone();
    thread::spawn(move || {
        for line in reader.lines() {
            match line {
                Ok(key) => {
                    am.send(AudioMessage::KeyPressed(key.trim().to_string()));
                }
                Err(err) => {
                    eprintln!("Pipe from key_listener broke: {err}");
                    break;
                }
            }
        }
    });

    iced::application("WhisperKeys", WhisperKeys::update, WhisperKeys::view).run_with(
        move || {
            (
                WhisperKeys {
                    audio_manager,
                    error_msg: None,
                    installed_packs: lib::pack::list_installed(&packs_dir).unwrap(),
                    selected_pack: None,
                    packs_path: packs_dir,
                    volume: None,
                    muted: false,
                },
                Task::none(),
            )
        },
    )?;

    child.kill().expect("Failed to kill key_listener");

    Ok(())
}

#[derive(Debug, Clone)]
enum Message {
    PackSelected(String),
    PackListRefreshed,
    VolumeChanged(u32),
    TranslatePack,
    OpenConfigsPath,
    ToggleMute,
}

struct WhisperKeys {
    audio_manager: AudioManager,
    installed_packs: Vec<String>,
    selected_pack: Option<String>,
    packs_path: PathBuf,
    error_msg: Option<String>,
    volume: Option<u32>,
    muted: bool,
}

impl WhisperKeys {
    fn update(&mut self, msg: Message) {
        use Message::*;

        match msg {
            VolumeChanged(v) => {
                self.volume = Some(v);
                self.audio_manager.send(AudioMessage::SetVolume(v));
            }
            PackSelected(p) => {
                self.error_msg = None;
                match Pack::load_from(&self.packs_path, &p) {
                    Ok(pack) => {
                        self.selected_pack = Some(p);
                        self.volume = Some(pack.default_volume);
                        self.audio_manager.send(AudioMessage::SetPack(pack));
                    }
                    Err(e) => self.error_msg = Some(e.to_string()),
                }
            }
            PackListRefreshed => {
                self.error_msg = None;
                self.installed_packs =
                    lib::pack::list_installed(&self.packs_path).unwrap_or_default()
            }
            TranslatePack => {
                self.error_msg = None;
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    if let Err(e) = lib::pack::from_mechvibes(&folder) {
                        self.error_msg = Some(e.to_string());
                    }
                }
            }
            OpenConfigsPath => open::that(self.packs_path.clone()).unwrap(),
            ToggleMute => {
                self.muted = !self.muted;
                self.audio_manager.send(AudioMessage::ToggleMute);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let mut column = Column::new();

        if let Some(e) = &self.error_msg {
            column = column.push(text(e));
        };

        let pick_list = pick_list(
            self.installed_packs.clone(),
            self.selected_pack.clone(),
            Message::PackSelected,
        )
        .placeholder("Choose a pack");
        let refresh_button = button("Refresh").on_press(Message::PackListRefreshed);

        column = column.push(row![pick_list, refresh_button]);

        if let Some(v) = self.volume
            && !self.muted
        {
            let slider = slider(1..=100, v, Message::VolumeChanged);
            let volume_text = text(format!("{v}%"));
            let mute = button("Mute").on_press(Message::ToggleMute);

            column = column.push(row![volume_text, slider, mute]);
        }

        if self.muted {
            let mute = button("Unmute").on_press(Message::ToggleMute);
            column = column.push(mute);
        }

        let mechvibes_translate =
            button("Convert mechvibes config").on_press(Message::TranslatePack);

        let open_folder = button("Open WhisperKeys folder").on_press(Message::OpenConfigsPath);

        column = column.push(row![mechvibes_translate, open_folder]);

        column.into()
    }
}
