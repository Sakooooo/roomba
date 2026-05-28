use std::collections::BTreeMap;
use std::fs::File;
use std::sync::Arc;

use iced::futures::lock::Mutex;
use iced::widget::{button, container, text};
use iced::{Element, Task};
use rfd::AsyncFileDialog;
use rodio::{Decoder, Player};

use crate::views::player::{self, Track};

mod views;

#[derive(Debug, Clone)]
pub enum Message {
    Increment,
    SwitchScreen(Screen),
    PickLibrary,
    ScanLibrary(String),
    LibraryScanned(BTreeMap<String, Vec<Track>>),
    ScanFail,
    PlaySong(Track),
    PreviousTrack,
    PlayPause,
    NextTrack,
}

#[derive(Debug, Clone)]
pub enum Screen {
    Blah,
    Player,
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

    let player = if let Some(sink) = &sink_handle {
        Some(Player::connect_new(sink.mixer()))
    } else {
        None
    };

    State {
        counter: 0,
        screen: Screen::Blah,
        current_track: None,
        current_track_cover: None,
        tracks: BTreeMap::new(),
        library: None,
        sink_handle,
        player,
    }
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
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

            state.current_track_cover = track
                .cover
                .as_ref()
                .map(|c| iced::widget::image::Handle::from_bytes(c.to_vec()));
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
    // TODO: cfg-if this
    dioxus_devtools::connect_subsecond();
    iced::application(new, update, view).run()
}
