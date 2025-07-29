use anyhow::Result;
use kira::{AudioManagerSettings, DefaultBackend};
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
    KeyPressed(String),
}

pub struct AudioManager {
    sender: Sender<AudioMessage>,
}

struct AudioManagerActor {
    receiver: Receiver<AudioMessage>,
    muted: bool,
    volume: i32,
    pack: Option<Pack>,
    manager: kira::AudioManager,
}

impl AudioManagerActor {
    pub fn new(rcv: Receiver<AudioMessage>) -> Result<Self> {
        Ok(Self {
            receiver: rcv,
            muted: false,
            volume: 50,
            pack: None,
            manager: kira::AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?,
        })
    }

    fn start(&mut self) {
        loop {
            if let Ok(msg) = self.receiver.recv() {
                match msg {
                    AudioMessage::ToggleMute => self.muted = !self.muted,
                    AudioMessage::VolumeUp(v) => self.volume += v,
                    AudioMessage::VolumeDown(v) => self.volume -= v,
                    AudioMessage::SetVolume(v) => self.volume = v,
                    AudioMessage::SetPack(pack) => self.pack = Some(pack),
                    AudioMessage::KeyPressed(key) => {
                        if self.muted {
                            return;
                        }

                        if let Some(pack) = &self.pack {
                            let sound_data = pack
                                .keys
                                .get(&key)
                                .unwrap_or_else(|| pack.keys.get("unknown").unwrap());

                            self.manager.play(sound_data.clone()).unwrap();
                        }
                    }
                }
            }
        }
    }
}

impl AudioManager {
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::channel::<AudioMessage>();
        let mut actor = AudioManagerActor::new(rx)?;

        thread::spawn(move || actor.start());

        Ok(Self { sender: tx })
    }

    pub fn send(&self, msg: AudioMessage) {
        self.sender.send(msg).unwrap();
    }
}
