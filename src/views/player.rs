use iced::widget;
use iced::Element;

pub fn view() -> Element<_> {
    widget::container(widget::text("Hi")).into()
}
