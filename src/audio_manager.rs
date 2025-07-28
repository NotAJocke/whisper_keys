use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crate::pack::Pack;

#[derive(Debug)]
pub enum AudioMessage {
    VolumeUp(i32),
    VolumeDown(i32),
    SetVolume(i32),
    ToggleMute,
    SetPack(Pack),
}

pub struct AudioManager {
    sender: Sender<AudioMessage>,
}

struct AudioManagerActor {
    receiver: Receiver<AudioMessage>,
    muted: bool,
    volume: i32,
    pack: Option<Pack>,
}

impl AudioManagerActor {
    pub fn new(rcv: Receiver<AudioMessage>) -> Self {
        Self {
            receiver: rcv,
            muted: false,
            volume: 50,
            pack: None,
        }
    }

    fn start(&mut self) {
        loop {
            if let Ok(msg) = self.receiver.recv() {
                match msg {
                    AudioMessage::ToggleMute => self.muted = !self.muted,
                    AudioMessage::VolumeUp(v) => self.volume += v,
                    AudioMessage::VolumeDown(v) => self.volume -= v,
                    AudioMessage::SetVolume(v) => self.volume = v,
                }
            }
        }
    }
}

impl AudioManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<AudioMessage>();
        let mut actor = AudioManagerActor::new(rx);

        thread::spawn(move || actor.start());

        Self { sender: tx }
    }

    pub fn send(&self, msg: AudioMessage) {
        self.sender.send(msg).unwrap();
    }
}
