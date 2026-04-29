use iced::widget::{button, text};
use iced::Element;

mod views;

#[derive(Debug, Clone)]
enum Message {
    Increment,
}

enum Screen {
    Player(),
}

struct State {
    counter: u64,
}

fn new() -> State {
    State { counter: 0 }
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::Increment => state.counter += 1,
    }
}

fn view(state: &State) -> Element<'_, Message> {
    button(text(state.counter))
        .on_press(Message::Increment)
        .into()
}

fn main() -> iced::Result {
    iced::application(new, update, view).run()
}
