use rdev::{EventType, Key};
use std::io::{Write, stdout};

fn main() {
    let mut last_pressed_key: Option<Key> = None;
    let mut last_event: Option<EventType> = None;

    rdev::listen(move |event| {
        if let EventType::KeyPress(key) = event.event_type {
            let is_same_key = last_pressed_key == Some(key);
            let last_event_is_keypress = matches!(last_event, Some(EventType::KeyPress(_)));

            if is_same_key && last_event_is_keypress {
                return;
            }

            let key_str = format!("{:?}\n", key);
            stdout().write_all(key_str.as_bytes()).unwrap();
            stdout().flush().unwrap();

            last_pressed_key = Some(key);
        }

        last_event = Some(event.event_type);
    })
    .unwrap();
}
