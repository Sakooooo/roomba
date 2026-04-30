use std::path::Path;

use crate::{Message, State};
use audiotags::{Album, AudioTag, MimeType, Tag};
use iced::futures::lock::MutexGuard;
use iced::widget::{button, column, container, image as img, text};
use iced::Element;
use image::DynamicImage;

pub struct CurrentTrack {
    metadata: Box<dyn AudioTag>,
}

impl CurrentTrack {
    pub fn new(path: String) -> Result<Self, Box<dyn std::error::Error>> {
        let metadata = Tag::new().read_from_path(path)?;
        Ok(Self { metadata })
    }
}

#[derive(Debug)]
pub struct Track {}

pub fn update(state: &mut State, message: Message) {}

pub fn view(state: &State) -> Element<'static, Message> {
    column![
        {
            if let Some(current_track) = &state.current_track
                && let Some(album_cover) = current_track.metadata.album_cover()
            {
                container(img(img::Handle::from_bytes(album_cover.data.to_vec())))
            } else {
                container("no album cover")
            }
        },
        button("Scan Library").on_press(Message::ScanLibrary("~/music".to_string()))
    ]
    .into()
}
