use std::{path::Path, time::Duration};

use rodio::Source;

use crate::library::Track;

pub struct Player {
    handle: rodio::MixerDeviceSink,
    player: Option<rodio::Player>, // this plays the audio
    pub current_track: Option<Track>,
    pub duration: Option<Duration>,
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
            duration: None,
        }
    }

    pub fn get_position(&self) -> Duration {
        if let Some(player) = &self.player {
            player.get_pos()
        } else {
            Duration::new(0, 0)
        }
    }

    pub fn play_path(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        self.play_path_impl(path.as_ref())
    }

    pub fn seek(&self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        match &self.player {
            Some(player) => player.try_seek(pos),
            None => Ok(()),
        }
    }
    pub fn is_playing(&self) -> bool {
        self.player
            .as_ref()
            .is_some_and(|player| !player.is_paused() && !player.empty())
    }

    fn play_path_impl(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let file = match std::fs::File::open(path) {
            Ok(c) => c,
            Err(e) => {
                println!("Failed to open file {:#?}, {}", path, e);
                return Err(Box::new(e));
            }
        };

        let decoder = rodio::Decoder::try_from(file)?;

        let duration = decoder.total_duration();

        let player = rodio::Player::connect_new(&self.handle.mixer());
        player.append(decoder);

        self.player = Some(player);
        self.duration = duration;

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
