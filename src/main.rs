use std::collections::BTreeMap;
use std::fs::File;

use futures::TryFutureExt;
use iced::wgpu::wgc::command::Command;
use iced::widget::{button, container, text};
use iced::{Element, Task};
use mpris_server::{
    LocalPlaylistsInterface, LocalTrackListInterface, LoopStatus, Metadata, PlaybackRate,
    PlaybackStatus, PlayerInterface, Property, RootInterface, Server, Time, TrackId, Uri, Volume,
};
use mpris_server::{LocalRootInterface, zbus::fdo};
use rfd::AsyncFileDialog;
use rodio::{Decoder, Player};
use std::sync::Arc;

use crate::views::player::{self, Track};

mod views;

#[derive(Debug, Clone)]
pub enum Message {
    None,
    Increment,
    SwitchScreen(Screen),
    PickLibrary,
    ScanLibrary(String),
    LibraryScanned(BTreeMap<String, Vec<Track>>),
    ScanFail,
    PlaySong(Track),
    PreviousTrack,
    PlayPause,
    Stop,
    NextTrack,
}

#[derive(Debug, Clone)]
pub enum Screen {
    Blah,
    Player,
}

#[derive(Default, Clone)]
pub struct MprisHandler {
    current_track: Option<Track>,
    current_track_cover: Option<iced::widget::image::Handle>,
}

impl RootInterface for MprisHandler {
    async fn raise(&self) -> fdo::Result<()> {
        println!("Raise");
        Ok(())
    }

    async fn quit(&self) -> fdo::Result<()> {
        println!("Quit");
        Ok(())
    }

    async fn can_quit(&self) -> fdo::Result<bool> {
        println!("CanQuit");
        Ok(true)
    }

    async fn fullscreen(&self) -> fdo::Result<bool> {
        println!("Fullscreen");
        Ok(false)
    }

    async fn set_fullscreen(&self, fullscreen: bool) -> mpris_server::zbus::Result<()> {
        println!("SetFullscreen({fullscreen})");
        Ok(())
    }

    async fn can_set_fullscreen(&self) -> fdo::Result<bool> {
        println!("CanSetFullscreen");
        Ok(false)
    }

    async fn can_raise(&self) -> fdo::Result<bool> {
        println!("CanRaise");
        Ok(true)
    }

    async fn has_track_list(&self) -> fdo::Result<bool> {
        println!("HasTrackList");
        Ok(false)
    }

    async fn identity(&self) -> fdo::Result<String> {
        println!("Identity");
        Ok("roomba".to_string())
    }

    async fn desktop_entry(&self) -> fdo::Result<String> {
        println!("DesktopEntry");
        Ok("roomba".to_string())
    }

    async fn supported_uri_schemes(&self) -> fdo::Result<Vec<String>> {
        println!("SupportedUriSchemes");
        Ok(vec![])
    }

    async fn supported_mime_types(&self) -> fdo::Result<Vec<String>> {
        println!("SupportedMimeTypes");
        Ok(vec![])
    }
}

impl PlayerInterface for MprisHandler {
    async fn next(&self) -> fdo::Result<()> {
        println!("Next");
        Ok(())
    }

    async fn previous(&self) -> fdo::Result<()> {
        println!("Previous");
        Ok(())
    }

    async fn pause(&self) -> fdo::Result<()> {
        println!("Pause");
        Ok(())
    }

    async fn play_pause(&self) -> fdo::Result<()> {
        println!("PlayPause");
        Ok(())
    }

    async fn stop(&self) -> fdo::Result<()> {
        println!("Stop");
        Ok(())
    }

    async fn play(&self) -> fdo::Result<()> {
        println!("Play");
        Ok(())
    }

    async fn seek(&self, offset: Time) -> fdo::Result<()> {
        println!("Seek({offset:?})");
        Ok(())
    }

    async fn set_position(&self, track_id: TrackId, position: Time) -> fdo::Result<()> {
        println!("SetPosition({track_id}, {position:?})");
        Ok(())
    }

    async fn open_uri(&self, uri: String) -> fdo::Result<()> {
        println!("OpenUri({uri})");
        Ok(())
    }

    async fn playback_status(&self) -> fdo::Result<PlaybackStatus> {
        println!("PlaybackStatus");
        Ok(PlaybackStatus::Playing)
    }

    async fn loop_status(&self) -> fdo::Result<LoopStatus> {
        println!("LoopStatus");
        Ok(LoopStatus::None)
    }

    async fn set_loop_status(&self, loop_status: LoopStatus) -> mpris_server::zbus::Result<()> {
        println!("SetLoopStatus({loop_status})");
        Ok(())
    }

    async fn rate(&self) -> fdo::Result<PlaybackRate> {
        println!("Rate");
        Ok(PlaybackRate::default())
    }

    async fn set_rate(&self, rate: PlaybackRate) -> mpris_server::zbus::Result<()> {
        println!("SetRate({rate})");
        Ok(())
    }

    async fn shuffle(&self) -> fdo::Result<bool> {
        println!("Shuffle");
        Ok(false)
    }

    async fn set_shuffle(&self, shuffle: bool) -> mpris_server::zbus::Result<()> {
        println!("SetShuffle({shuffle})");
        Ok(())
    }

    async fn metadata(&self) -> fdo::Result<Metadata> {
        let mut metadata = Metadata::builder();
        if let Some(track) = &self.current_track {
            if let Some(album_title) = &track.album_title {
                metadata = metadata.album(album_title);
            }

            if let Some(title) = &track.title {
                metadata = metadata.title(title);
            }
        };
        Ok(metadata.build())
    }

    async fn volume(&self) -> fdo::Result<Volume> {
        println!("Volume");
        Ok(Volume::default())
    }

    async fn set_volume(&self, volume: Volume) -> mpris_server::zbus::Result<()> {
        println!("SetVolume({volume})");
        Ok(())
    }

    async fn position(&self) -> fdo::Result<Time> {
        println!("Position");
        Ok(Time::ZERO)
    }

    async fn minimum_rate(&self) -> fdo::Result<PlaybackRate> {
        println!("MinimumRate");
        Ok(PlaybackRate::default())
    }

    async fn maximum_rate(&self) -> fdo::Result<PlaybackRate> {
        println!("MaximumRate");
        Ok(PlaybackRate::default())
    }

    async fn can_go_next(&self) -> fdo::Result<bool> {
        println!("CanGoNext");
        Ok(false)
    }

    async fn can_go_previous(&self) -> fdo::Result<bool> {
        println!("CanGoPrevious");
        Ok(false)
    }

    async fn can_play(&self) -> fdo::Result<bool> {
        println!("CanPlay");
        Ok(true)
    }

    async fn can_pause(&self) -> fdo::Result<bool> {
        println!("CanPause");
        Ok(true)
    }

    async fn can_seek(&self) -> fdo::Result<bool> {
        println!("CanSeek");
        Ok(false)
    }

    async fn can_control(&self) -> fdo::Result<bool> {
        println!("CanControl");
        Ok(true)
    }
}

pub struct State {
    counter: u64,
    screen: Screen,
    current_track: Option<Track>,
    current_track_cover: Option<iced::widget::image::Handle>,
    tracks: BTreeMap<String, Vec<Track>>,
    library: Option<String>,
    sink_handle: Option<rodio::MixerDeviceSink>,
    player: Option<Player>,
    mpris_state: MprisHandler,
    mpris_server: Option<Server<MprisHandler>>,
}

fn new() -> State {
    let sink_handle = match rodio::DeviceSinkBuilder::open_default_sink() {
        Ok(sink) => Some(sink),
        Err(e) => {
            println!("ERROR: Failed to get DeviceSink, audio will not play!!!");
            println!("{e}");
            None
        }
    };

    let player = sink_handle
        .as_ref()
        .map(|sink| Player::connect_new(sink.mixer()));

    State {
        counter: 0,
        screen: Screen::Blah,
        current_track: None,
        current_track_cover: None,
        tracks: BTreeMap::new(),
        library: None,
        sink_handle,
        player,
        mpris_state: MprisHandler::default(),
        mpris_server: None,
    }
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::None => Task::none(),
        Message::Increment => {
            state.counter += 1;
            Task::none()
        }
        Message::SwitchScreen(screen) => {
            state.screen = screen;
            Task::none()
        }
        Message::PickLibrary => Task::perform(AsyncFileDialog::new().pick_folder(), |x| match x {
            Some(x) => Message::ScanLibrary(x.path().to_string_lossy().to_string()),
            None => Message::ScanFail,
        }),
        Message::ScanLibrary(path) => {
            // Task::none()
            Task::perform(player::scan_library(path), |x| match x {
                Ok(result) => Message::LibraryScanned(result),
                Err(e) => Message::ScanFail,
            })
        }
        Message::LibraryScanned(tracks) => {
            state.tracks = tracks;
            Task::none()
        }
        Message::ScanFail => {
            println!("Fail!");
            Task::none()
        }
        Message::PlaySong(track) => {
            let file = std::io::BufReader::new(File::open(&track.filepath).unwrap());

            // TODO probably make this global later so to cause any random track to play
            let mpris =
                match futures::executor::block_on(Server::new("roomba", state.mpris_state.clone()))
                {
                    Ok(m) => Some(m),
                    Err(e) => {
                        println!("Failed to create mpris context {}", e);
                        None
                    }
                };

            state.mpris_server = mpris;

            if let Some(player) = &state.player {
                if !player.empty() {
                    player.stop();
                    player.clear();
                }
                let source = match Decoder::try_from(file) {
                    Ok(s) => s,
                    Err(e) => {
                        println!("Failed to decode audio!!! {e}");
                        return Task::none();
                    }
                };
                player.append(source);
                if player.is_paused() {
                    player.play();
                }
            }

            let cover = track
                .cover
                .as_ref()
                .map(|c| iced::widget::image::Handle::from_bytes(c.to_vec()));

            state.mpris_state.current_track_cover = cover.clone();
            state.mpris_state.current_track = Some(track.clone());
            state.current_track_cover = cover;
            state.current_track = Some(track);

            Task::none()
        }
        Message::PreviousTrack => Task::none(),
        Message::PlayPause => {
            if let Some(player) = &state.player {
                if player.is_paused() {
                    player.play();
                } else {
                    player.pause();
                };
            }
            Task::none()
        }
        Message::Stop => {
            // drop Server so Mpris context is gone
            state.mpris_server = None;
            if let Some(player) = &state.player {
                player.stop();

                state.current_track = None;
                state.current_track_cover = None;
            }

            Task::none()
        }
        Message::NextTrack => Task::none(),
    }
}

fn view(state: &State) -> Element<'_, Message> {
    match state.screen {
        Screen::Blah => {
            container(button(text("Press me!")).on_press(Message::SwitchScreen(Screen::Player)))
                .center_x(iced::Fill)
                .center_y(iced::Fill)
                .into()
        }
        Screen::Player => views::player::view(state),
    }
}

fn main() -> iced::Result {
    #[cfg(feature = "debug")]
    dioxus_devtools::connect_subsecond();

    iced::application(new, update, view).run()
}
