use iced::{
    widget::{column, container, text, Container},
    Element,
};

use crate::Message;

impl crate::App {
    pub fn settings_view(&self) -> Element<'_, Message> {
        container(column![
            text("test"),
            text("test"),
            text("test"),
            text("test"),
            text("test"),
            text("test"),
            text("test"),
            text("test"),
            text("test"),
            text("test"),
        ])
        .width(800)
        .height(400)
        .padding(10)
        .style(container::rounded_box)
        .into()
    }
}
