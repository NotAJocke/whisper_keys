use anyhow::Result;
use fastrand::Rng;
use kira::{AudioManagerSettings, Decibels, DefaultBackend, Semitones};
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::{self},
};

use crate::pack::Pack;

#[derive(Debug)]
pub enum AudioMessage {
    SetVolume(u32),
    ToggleMute,
    SetPack(Pack),
    KeyPressed(String),
    Shutdown,
}

#[derive(Clone)]
pub struct AudioManager {
    sender: Sender<AudioMessage>,
}

struct AudioManagerActor {
    receiver: Receiver<AudioMessage>,
    muted: bool,
    volume: u32,
    cached_db: f32,
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
            cached_db: 20.0 * 0.5_f32.log10(),
        })
    }

    fn start(&mut self) {
        loop {
            match self.receiver.recv() {
                Ok(AudioMessage::ToggleMute) => self.muted = !self.muted,
                Ok(AudioMessage::SetVolume(v)) => self.update_volume(v),
                Ok(AudioMessage::SetPack(pack)) => {
                    self.update_volume(pack.default_volume);
                    self.pack = Some(pack);
                }
                Ok(AudioMessage::KeyPressed(key)) => {
                    if !self.muted {
                        self.handle_keypress(key);
                    }
                }
                Ok(AudioMessage::Shutdown) => break,
                Err(_) => break,
            }
        }
    }

    fn update_volume(&mut self, volume: u32) {
        self.volume = volume;

        // dB = 20 * log_10(Amplitude)
        self.cached_db = 20.0 * (volume as f32 * 0.01).log10();
    }

    fn handle_keypress(&mut self, key: String) {
        if let Some(pack) = &self.pack {
            // generates value in [-0.25, 0.25]
            let semitone_shift = self.rng.f64() * 0.5 - 0.25;
            let db_variation = self.rng.f32() * 2.0 - 1.0; // random float in [-1.0, 1.0]
            let final_db = self.cached_db + db_variation;

            let sound_data = pack
                .keys
                .get(&key)
                .or_else(|| pack.keys.get("Unknown"))
                .map(|sound| {
                    sound
                        .volume(Decibels(final_db))
                        .playback_rate(Semitones(semitone_shift))
                });

            if let Some(sound_data) = sound_data {
                if let Err(e) = self.manager.play(sound_data) {
                    eprintln!("Failed to play sound: {}", e);
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

    pub fn shutdown(self) {
        self.sender.send(AudioMessage::Shutdown).unwrap();
    }

    pub fn send(&self, msg: AudioMessage) -> Result<()> {
        self.sender.send(msg)?;
        Ok(())
    }
}
