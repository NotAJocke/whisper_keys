use anyhow::{bail, Result};
use iced::widget::{button, column, text, Column, Text};
use iced::Alignment;
use lib::{
    audio_manager::{AudioManager, AudioMessage},
    pack::Pack,
};
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
    let pack = Pack::load_from(&packs_dir, "Mammoth75").unwrap();

    audio_manager.send(AudioMessage::SetPack(pack));

    thread::spawn(move || {
        for line in reader.lines() {
            let key = line.unwrap().trim().to_string();

            audio_manager.send(AudioMessage::KeyPressed(key));
        }
    });

    iced::run("WhisperKeys", Counter::update, Counter::view)?;

    child.kill().expect("Failed to kill key_listener");

    Ok(())
}

#[derive(Default)]
struct Counter {
    value: i64,
}

impl Counter {
    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => self.value += 1,
            Message::Decrement => self.value -= 1,
        }
    }

    fn view(&self) -> Column<'_, Message> {
        let increment_btn = button("Increment").on_press(Message::Increment);
        let counter = text(self.value);
        let decrement_btn = button("Decrement").on_press(Message::Decrement);

        column![HelloWorld {}.view(), increment_btn, counter, decrement_btn]
            .padding(20)
            .align_x(Alignment::Center)
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
    Decrement,
}

struct HelloWorld {}
impl HelloWorld {
    fn view(&self) -> Text<'_> {
        text("Hey !")
    }
}
