use std::path::Path;

use crate::library::Track;

pub struct Player {
    handle: rodio::MixerDeviceSink,
    player: Option<rodio::Player>, // this plays the audio
    pub current_track: Option<Track>,
    pub current_cover: Option<iced::widget::image::Handle>,
}

impl Player {
    pub fn new() -> Self {
        let handle = rodio::DeviceSinkBuilder::open_default_sink()
            .expect("Failed to open default audio stream");

        Self {
            handle,
            player: None,
            current_track: None,
            current_cover: None,
        }
    }

    pub fn play_path(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        self.play_path_impl(path.as_ref())
    }

    fn play_path_impl(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let file = match std::fs::File::open(path) {
            Ok(c) => c,
            Err(e) => {
                println!("Failed to open file {:#?}, {}", path, e);
                return Err(Box::new(e));
            }
        };

        let player = match rodio::play(&self.handle.mixer(), file) {
            Ok(p) => p,
            Err(e) => {
                println!("Failed to play audio! {}", e);
                return Err(Box::new(e));
            }
        };

        self.player = Some(player);

        Ok(())
    }

    pub fn toggle_pause(&self) {
        if let Some(player) = &self.player {
            match player.is_paused() {
                true => player.play(),
                false => player.pause(),
            }
        }
    }
}
