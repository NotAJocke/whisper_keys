use anyhow::{Context, Result};
use iced::daemon::Appearance;
use iced::widget::{Column, Space, button, column, container, pick_list, row, slider, text};
use iced::{Alignment, Color, Element, Length, Padding, Size, Task};
use lib::audio_manager::{AudioManager, AudioMessage};
use lib::pack::Pack;
use std::path::PathBuf;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    thread,
};

mod style;

fn helper_path() -> PathBuf {
    let self_path = std::env::current_exe().unwrap();

    if cfg!(debug_assertions) {
        return self_path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("release/key_listener");
    }

    if cfg!(target_os = "macos") {
        self_path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("Resources")
            .join("key_listener")
    } else {
        panic!(
            "Unsupported platform, only macOS is supported for now, please open an issue on GitHub if you want to see support for other platforms"
        );
    }
}

fn main() -> Result<()> {
    let helper_path = helper_path();

    let mut child = Command::new(helper_path)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to launch helper");

    let stdout = child
        .stdout
        .take()
        .context("Failed to get stdout from child process")?;
    let reader = BufReader::new(stdout);

    let audio_manager = AudioManager::new().context("Failed to create audio manager")?;

    let home = std::env::home_dir().context("Couldn't get the user's home directory")?;
    let packs_dir = home.join("WhisperKeys");

    let am = audio_manager.clone();
    thread::spawn(move || {
        for line in reader.lines() {
            match line {
                Ok(key) => {
                    if let Err(e) = am.send(AudioMessage::KeyPressed(key.trim().to_string())) {
                        eprintln!("Failed to send key press message: {}", e);
                        break;
                    }
                }
                Err(err) => {
                    eprintln!("Pipe from key_listener broke: {err}");
                    break;
                }
            }
        }
    });

    let installed_packs = lib::pack::list_installed(&packs_dir).unwrap_or_else(|e| {
        eprintln!("Failed to load initial packs: {}", e);
        Vec::new()
    });
    let installed_packs = format_pack_list(installed_packs);

    iced::application("", WhisperKeys::update, WhisperKeys::view)
        .level(iced::window::Level::AlwaysOnTop)
        .resizable(false)
        .window_size(Size::new(400.0, 600.0))
        .style(|_, _| Appearance {
            background_color: *style::BACKGROUND_COLOR,
            text_color: Color::WHITE,
        })
        .run_with(move || {
            (
                WhisperKeys {
                    audio_manager,
                    error_msg: None,
                    installed_packs,
                    selected_pack: None,
                    packs_path: packs_dir,
                    volume: None,
                    muted: false,
                },
                Task::none(),
            )
        })?;

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
    CreateNewPack,
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
                if let Err(e) = self.audio_manager.send(AudioMessage::SetVolume(v)) {
                    self.error_msg = Some(format!("Failed to set volume: {}", e));
                }
            }
            PackSelected(p) => {
                self.error_msg = None;
                match Pack::load_from(&self.packs_path, &p) {
                    Ok(pack) => {
                        self.selected_pack = Some(p);
                        self.volume = Some(pack.default_volume);
                        if let Err(e) = self.audio_manager.send(AudioMessage::SetPack(pack)) {
                            self.error_msg = Some(format!("Failed to set pack: {}", e));
                        }
                    }
                    Err(e) => self.error_msg = Some(e.to_string()),
                }
            }
            PackListRefreshed => {
                self.error_msg = None;
                let packs = lib::pack::list_installed(&self.packs_path).unwrap_or_default();
                self.installed_packs = format_pack_list(packs);
            }
            TranslatePack => {
                self.error_msg = None;
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    if let Err(e) = lib::pack::from_mechvibes(&folder) {
                        self.error_msg = Some(format!("Translation failed: {}", e));
                    }
                }
            }
            OpenConfigsPath => {
                if let Err(e) = open::that(&self.packs_path) {
                    self.error_msg = Some(format!("Failed to open folder: {}", e));
                }
            }
            ToggleMute => {
                self.muted = !self.muted;
                if let Err(e) = self.audio_manager.send(AudioMessage::ToggleMute) {
                    self.error_msg = Some(format!("Failed to toggle mute: {}", e));
                }
            }
            CreateNewPack => {
                self.error_msg = None;
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    if let Err(e) = lib::pack::create_new_pack(&folder) {
                        self.error_msg = Some(format!("Failed to create new pack: {}", e));
                    }

                    if let Err(e) = open::that(folder) {
                        self.error_msg = Some(format!("Failed to open folder: {}", e))
                    }
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        container(
            Column::new()
                .push_maybe(self.error_display())
                .push(self.header())
                .push(Space::with_height(15))
                .push(self.pack_selection())
                .push_maybe((self.volume.is_some()).then_some(Space::with_height(15)))
                .push_maybe(self.volume_control())
                .push(Space::with_height(15))
                .push(self.utils_buttons()),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: 25.0,
            bottom: 25.0,
            right: 15.0,
            left: 15.0,
        })
        .into()
    }

    fn error_display(&self) -> Option<Element<'_, Message>> {
        let error_msg = self.error_msg.as_ref()?;
        let error_text = text(format!("Error: {}", error_msg)).color(style::ERROR_COLOR);

        Some(container(error_text).center_x(Length::Fill).into())
    }

    fn header(&self) -> Element<'_, Message> {
        let title = text("WhisperKeys").size(28);

        container(title)
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .into()
    }

    fn pack_selection(&self) -> Element<'_, Message> {
        let pick_list = pick_list(
            self.installed_packs.clone(),
            self.selected_pack.clone(),
            Message::PackSelected,
        )
        .placeholder("Choose a pack")
        .padding(Padding::default().right(10).left(10).top(10).bottom(10))
        .style(style::picklist());

        let refresh_button = button("Refresh")
            .on_press(Message::PackListRefreshed)
            .style(style::refresh_btn());

        let pack_selection = container(column!(pick_list, refresh_button).align_x(Alignment::End))
            .width(Length::Fill)
            .align_x(Alignment::Center);

        pack_selection.into()
    }

    fn volume_control(&self) -> Option<Element<'_, Message>> {
        let volume = self.volume?;

        let volume_text = text(format!("{}%", volume));
        let mut slider = slider(1..=100, volume, Message::VolumeChanged)
            .style(style::volume_slider())
            .step(10u32);

        if self.muted {
            slider = slider.style(style::volume_slider_muted());
        }

        let mute_button = if self.muted {
            button("Unmute")
                .on_press(Message::ToggleMute)
                .style(style::generic_button())
        } else {
            button("Mute")
                .on_press(Message::ToggleMute)
                .style(style::generic_button())
        };

        Some(
            row![
                volume_text,
                Space::with_width(10),
                slider,
                Space::with_width(10),
                mute_button
            ]
            .align_y(Alignment::Center)
            .into(),
        )
    }

    fn utils_buttons(&self) -> Element<'_, Message> {
        let from_mechvibes = button(text("Convert mechvibes config").align_x(Alignment::Center))
            .on_press(Message::TranslatePack)
            .width(Length::Fixed(200.0))
            .style(style::generic_button());

        let open_folder = button(text("Open WhisperKeys folder").align_x(Alignment::Center))
            .on_press(Message::OpenConfigsPath)
            .width(Length::Fixed(200.0))
            .style(style::generic_button());

        let create_pack = button(text("Create new empty pack").align_x(Alignment::Center))
            .on_press(Message::CreateNewPack)
            .width(Length::Fixed(200.0))
            .style(style::generic_button());

        column![from_mechvibes, open_folder, create_pack]
            .spacing(6)
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .into()
    }
}

fn format_pack_list(packs: Vec<String>) -> Vec<String> {
    packs
        .into_iter()
        .map(|p| {
            if p.len() > 28 {
                let mut s = String::with_capacity(28 + 3);
                s.extend(p.chars().take(28));
                s.push_str("…");
                s
            } else {
                p
            }
        })
        .collect()
}
