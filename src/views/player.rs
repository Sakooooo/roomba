use crate::{Message, State};
use audiotags::{MimeType, Tag};
use iced::widget::{button, column, container, image as img, text};
use iced::Element;
use image::DynamicImage;

pub struct CurrentTrack {
    cover: img::Handle,
}

impl CurrentTrack {
    pub fn new(path: String) -> Result<Self, Box<dyn std::error::Error>> {
        let metadata = Tag::new().read_from_path(path)?;

        let album_cover = metadata.album_cover();

        // let filetype = if let Some(ref cover_data) = album_cover {
        //     match cover_data.mime_type {
        //         MimeType::Png => Some(image::ImageFormat::Png),
        //         MimeType::Jpeg => Some(image::ImageFormat::Jpeg),
        //         MimeType::Gif => Some(image::ImageFormat::Gif),
        //         MimeType::Bmp => Some(image::ImageFormat::Bmp),
        //         MimeType::Tiff => Some(image::ImageFormat::Tiff),
        //     }
        // } else {
        //     None
        // };

        let cover = img::Handle::from_bytes(album_cover.unwrap().data);

        // let cover =
        //     image::load_from_memory_with_format(album_cover.unwrap().data, filetype.unwrap())?
        //         .as_bytes();

        // let cover = if let Some(cover_data) = album_cover {
        //     if let Some(format) = filetype {
        //         Some(image::load_from_memory_with_format(
        //             cover_data.data,
        //             format,
        //         )?);
        //     } else {
        //         None
        //     }
        // } else {
        //     None
        // };

        Ok(Self { cover })
    }
}

pub fn update(state: &mut State, message: Message) {}

pub fn view(state: &State) -> Element<'static, Message> {
    column![
        {
            if let Some(track_info) = &state.current_track {
                container(img(track_info.cover.clone()))
            } else {
                container("test")
            }
        },
        button("test")
    ]
    .into()
}
