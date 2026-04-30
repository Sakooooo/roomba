use std::sync::Arc;

use iced::futures::lock::Mutex;
use iced::widget::{button, container, text};
use iced::{Element, Task};

use crate::views::player::{self, CurrentTrack, Track};

mod views;

#[derive(Debug, Clone)]
pub enum Message {
    Increment,
    SwitchScreen(Screen),
    ScanLibrary(String),
    LibraryScanned(Vec<Track>),
    ScanFail,
}

#[derive(Debug, Clone)]
pub enum Screen {
    Blah,
    Player,
}

pub struct State {
    counter: u64,
    screen: Screen,
    current_track: Option<CurrentTrack>,
    tracks: Vec<Track>,
}

fn new() -> State {
    State {
        counter: 0,
        screen: Screen::Blah,
        current_track: None,
        tracks: Vec::new(),
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
        Message::ScanLibrary(path) => {
            // Task::none()
            Task::perform(player::scan_library(path), |x| match x {
                Ok(result) => Message::LibraryScanned(result),
                Err(e) => Message::ScanFail,
            })
        }
        Message::LibraryScanned(tracks) => {
            println!("Wow!");
            state.tracks = tracks;
            Task::none()
        }
        Message::ScanFail => {
            println!("Fail!");
            Task::none()
        }
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
    iced::application(new, update, view).run()
}
