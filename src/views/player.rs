use std::collections::BTreeMap;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use crate::{Message, State};
use audiotags::Tag;
use iced::Element;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{
    Column, Row, button, column, container, image as img, lazy, row, scrollable, text,
};

use rkyv::{Archive, Deserialize, Serialize, deserialize, rancor::Error};

const MISSING_COVER_BYTES: &[u8] = include_bytes!("./missing.png");
pub struct CurrentTrack {
    pub track: Track, // state
}

#[derive(Debug, Clone)]
pub enum PlayerError {
    LibraryNotFound,
    IoError,
    LibraryPickerCanceled,
}

impl std::fmt::Display for PlayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerError::IoError => write!(f, "Failed to read library path"),
            PlayerError::LibraryNotFound => write!(f, "Could not find library"),
            PlayerError::LibraryPickerCanceled => write!(f, "Library dialog menu was closed"),
        }
    }
}

#[derive(Debug, Default, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Track {
    pub track_number: Option<u16>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album_artist: Option<String>,
    pub album_title: Option<String>,
    pub filepath: String,
}

impl Track {
    pub fn new(path: String) -> Result<Self, audiotags::error::Error> {
        let mut result = Self::default();

        let metadata = Tag::new().read_from_path(&path)?;

        result.track_number = metadata.track_number();

        if let Some(title) = metadata.title() {
            result.title = Some(title.to_string());
        };

        if let Some(artist) = metadata.artist() {
            result.artist = Some(artist.to_string());
        };

        if let Some(album_metadata) = metadata.album() {
            result.album_title = Some(album_metadata.title.to_string());
        };

        result.filepath = path;

        Ok(result)
    }

    pub fn get_album_cover(&self) -> Result<Vec<u8>, audiotags::error::Error> {
        let metadata = Tag::new().read_from_path(&self.filepath)?;

        if let Some(album_metadata) = metadata.album()
            && let Some(cover) = album_metadata.cover
        {
            Ok(cover.data.to_vec())
        } else {
            Ok(MISSING_COVER_BYTES.to_owned())
        }
    }
}

pub async fn scan_library(path: String) -> Result<BTreeMap<String, Vec<Track>>, PlayerError> {
    let library_path = Path::new(&path);
    if !library_path.exists() {
        println!("fail");
        return Err(PlayerError::LibraryNotFound);
    }

    let mut tracks: Vec<Track> = vec![];

    let mut queued_dirs: Vec<PathBuf> = vec![library_path.to_path_buf()];

    while let Some(path) = queued_dirs.pop() {
        match path.read_dir() {
            Ok(files) => {
                for file in files {
                    match file {
                        Ok(res) => {
                            let path = res.path();
                            if let Ok(filetype) = res.file_type()
                                && filetype.is_dir()
                            {
                                queued_dirs.push(path);
                            } else {
                                if let Ok(track) =
                                    Track::new(res.path().to_string_lossy().to_string())
                                {
                                    tracks.push(track)
                                } else {
                                    println!("Skipping {}", path.to_string_lossy().to_string());
                                };
                            };
                        }
                        Err(e) => {
                            println!("Failed to read file {}", e);
                            return Err(PlayerError::IoError);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to open library path: {}", e);
                return Err(PlayerError::IoError);
            }
        };
    }

    let mut map: BTreeMap<String, Vec<Track>> = BTreeMap::new();

    for track in tracks {
        if let Some(album_title) = track.album_title.clone() {
            map.entry(album_title).or_insert_with(Vec::new).push(track);
        } else {
            map.entry("No Album".to_string())
                .or_insert_with(Vec::new)
                .push(track);
        };
    }

    for tracks in map.values_mut() {
        tracks.sort_by_key(|track| track.track_number.unwrap_or(67));
    }

    Ok(map)
}

fn track_list(tracks: &BTreeMap<String, Vec<Track>>) -> Element<'static, Message> {
    let mut col = Column::new();
    for (album, list) in tracks {
        col = col.push(container(text(album.clone())).height(25));
        for (i, track) in list.iter().enumerate() {
            let label = track.title.clone().unwrap_or_else(|| "Unknown".to_string());
            col = col.push(
                button(text(label))
                    .on_press(Message::PlaySong(album.clone(), i))
                    .width(iced::Fill),
            );
        }
    }
    col.into()
}

// pub fn update(state: &mut State, message: Message) {}

pub fn view(state: &State) -> Element<'_, Message> {
    let album_cover = if let Some(cover) = &state.current_track_cover {
        container(img(cover).expand(true))
            .width(512)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
    } else {
        container(img(img::Handle::from_bytes(MISSING_COVER_BYTES)).width(512))
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
    };

    let current_track = container(
        column![
            album_cover,
            container(row![
                button("previous"),
                button("play/pause").on_press(Message::PlayPause),
                button("next"),
                button("stop").on_press(Message::Stop)
            ])
        ]
        .align_x(Horizontal::Center),
    );

    let tracks = lazy(state.track_rev, |_| track_list(&state.tracks));

    container(
        Row::new()
            .push(current_track)
            .push(button("Pick Library").on_press(Message::PickLibrary))
            .push(
                button("Scan Library")
                    .on_press(Message::ScanLibrary("/home/user/music".to_string())),
            )
            // this lags the shit out of the app
            .push(container(scrollable(tracks)).width(1000))
            .align_y(Vertical::Center)
            .spacing(10),
    )
    .center_x(iced::Fill)
    .center_y(iced::Fill)
    .into()
}
