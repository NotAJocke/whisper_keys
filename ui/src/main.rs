use anyhow::{Result, bail};
use iced::widget::{button, column, pick_list, row};
use iced::{Element, Task};
use lib::audio_manager::{AudioManager, AudioMessage};
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
                    installed_packs: lib::pack::list_installed(&packs_dir).unwrap(),
                    selected_pack: None,
                    packs_path: packs_dir,
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
}

#[derive(Default)]
struct WhisperKeys {
    installed_packs: Vec<String>,
    selected_pack: Option<String>,
    packs_path: PathBuf,
}

impl WhisperKeys {
    fn update(&mut self, msg: Message) {
        use Message::*;

        match msg {
            PackSelected(p) => self.selected_pack = Some(p),
            PackListRefreshed => {
                self.installed_packs =
                    lib::pack::list_installed(&self.packs_path).unwrap_or_default()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let pick_list = pick_list(
            self.installed_packs.clone(),
            self.selected_pack.clone(),
            Message::PackSelected,
        )
        .placeholder("Choose a pack");
        let refresh_button = button("Refresh").on_press(Message::PackListRefreshed);

        column![row![pick_list, refresh_button]].into()
    }
}
