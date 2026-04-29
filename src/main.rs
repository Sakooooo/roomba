use iced::widget::{button, container, text};
use iced::Element;

use crate::views::player::CurrentTrack;

mod views;

#[derive(Debug, Clone)]
pub enum Message {
    Increment,
    SwitchScreen(Screen),
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
}

fn new() -> State {
    State {
        counter: 0,
        screen: Screen::Blah,
        current_track: None,
    }
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::Increment => state.counter += 1,
        Message::SwitchScreen(screen) => state.screen = screen,
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
