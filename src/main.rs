mod audio_manager;
mod pack;

use audio_manager::AudioManager;

fn main() {
    let audio_manager = AudioManager::new();

    audio_manager.send(audio_manager::AudioMessage::ToggleMute);

    std::thread::sleep(std::time::Duration::from_secs(5));

    audio_manager.send(audio_manager::AudioMessage::ToggleMute);

    std::thread::park_timeout(std::time::Duration::from_secs(5));
}

// fn main() -> iced::Result {
//     iced::run("WhisperKeys", Counter::update, Counter::view)
// }

// #[derive(Default)]
// struct Counter {
//     value: i64,
// }

// impl Counter {
//     fn update(&mut self, message: Message) {
//         match message {
//             Message::Increment => self.value += 1,
//             Message::Decrement => self.value -= 1,
//         }
//     }

//     fn view(&self) -> Column<'_, Message> {
//         let increment_btn = button("Increment").on_press(Message::Increment);
//         let counter = text(self.value);
//         let decrement_btn = button("Decrement").on_press(Message::Decrement);

//         column![HelloWorld {}.view(), increment_btn, counter, decrement_btn]
//             .padding(20)
//             .align_x(Alignment::Center)
//     }
// }

// #[derive(Debug, Clone, Copy)]
// enum Message {
//     Increment,
//     Decrement,
// }

// struct HelloWorld {}
// impl HelloWorld {
//     fn view(&self) -> Text<'_> {
//         text("Hey !")
//     }
// }
