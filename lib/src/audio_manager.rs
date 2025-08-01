use anyhow::Result;
use fastrand::Rng;
use kira::{AudioManagerSettings, Decibels, DefaultBackend, Semitones};
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crate::pack::Pack;

#[derive(Debug)]
pub enum AudioMessage {
    SetVolume(u32),
    ToggleMute,
    SetPack(Pack),
    KeyPressed(String),
}

#[derive(Clone)]
pub struct AudioManager {
    sender: Sender<AudioMessage>,
}

struct AudioManagerActor {
    receiver: Receiver<AudioMessage>,
    muted: bool,
    volume: u32,
    pack: Option<Pack>,
    manager: kira::AudioManager,
    rng: Rng,
}

impl AudioManagerActor {
    pub fn new(rcv: Receiver<AudioMessage>) -> Result<Self> {
        Ok(Self {
            receiver: rcv,
            muted: false,
            volume: 50,
            pack: None,
            manager: kira::AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?,
            rng: Rng::new(),
        })
    }

    fn start(&mut self) {
        loop {
            if let Ok(msg) = self.receiver.recv() {
                match msg {
                    AudioMessage::ToggleMute => self.muted = !self.muted,
                    AudioMessage::SetVolume(v) => self.volume = v,
                    AudioMessage::SetPack(pack) => {
                        self.volume = pack.default_volume;
                        self.pack = Some(pack);
                    }
                    AudioMessage::KeyPressed(key) => {
                        if self.muted {
                            continue;
                        }

                        if let Some(pack) = &self.pack {
                            // generates value in [-0.25, 0.25]
                            let semitone_shift = self.rng.f64() * 0.5 - 0.25;

                            // dB = 20 * log_10(Amplitude)
                            let db = 20.0 * (self.volume as f32 * 0.01).log10();
                            let db_variation = self.rng.f32() * 2.0 - 1.0; // random float in [-1.0, 1.0]
                            let db = db + db_variation;

                            let sound_data = pack
                                .keys
                                .get(&key)
                                .unwrap_or_else(|| pack.keys.get("Unknown").unwrap())
                                .volume(Decibels(db))
                                .playback_rate(Semitones(semitone_shift));

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
