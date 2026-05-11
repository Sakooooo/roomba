use std::collections::BTreeMap;
use std::fs::DirEntry;
use std::path::Path;

use crate::{Message, State};
use audiotags::{Album, AudioTag, MimeType, Tag};
use iced::application::IntoBoot;
use iced::futures::lock::MutexGuard;
use iced::widget::{
    button, column, container, image as img, row, scrollable, text, Column, Row, Scrollable,
};
use iced::Element;
use image::DynamicImage;

pub struct CurrentTrack {
    pub track: Track, // state
}

pub enum PlayerError {
    LibraryNotFound,
    IoError,
}

#[derive(Debug, Default, Clone)]
pub struct Track {
    pub track_number: Option<u16>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album_artist: Option<String>,
    pub album_title: Option<String>,
    pub cover: Option<Vec<u8>>,
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
            if let Some(cover) = album_metadata.cover {
                // surely theres a better way
                result.cover = Some(cover.data.to_vec());
            }
        };

        result.filepath = path;

        Ok(result)
    }
}

pub async fn scan_library(path: String) -> Result<BTreeMap<String, Vec<Track>>, PlayerError> {
    let library_path = Path::new(&path);
    if !library_path.exists() {
        println!("fail");
        return Err(PlayerError::LibraryNotFound);
    }

    let mut tracks: Vec<Track> = vec![];

    match library_path.read_dir() {
        Ok(files) => {
            for file in files {
                match file {
                    Ok(entry) => {
                        let mut collected_folders: Vec<DirEntry> = Vec::new();
                        match entry.file_type() {
                            Ok(info) => {
                                if info.is_dir() {
                                    let to_search = entry.path().read_dir();
                                    match to_search {
                                        Ok(dirs) => {
                                            for file in dirs {
                                                match file {
                                                    Ok(entry) => {
                                                        match entry.file_type() {
                                                            Ok(filetype) => {
                                                                if filetype.is_dir() {
                                                                    collected_folders.push(entry);
                                                                };
                                                            }
                                                            Err(e) => {
                                                                println!("Fail {}", e);
                                                                return Err(PlayerError::IoError);
                                                            }
                                                        };
                                                    }
                                                    Err(e) => {
                                                        println!("Fail {}", e);
                                                        return Err(PlayerError::IoError);
                                                    }
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            println!("Fail {}", err);
                                            return Err(PlayerError::IoError);
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                println!("Fail {}", err);
                                return Err(PlayerError::IoError);
                            }
                        };

                        for dir in collected_folders {
                            match dir.path().read_dir() {
                                Ok(collected_folder) => {
                                    for file in collected_folder {
                                        match file {
                                            Ok(result) => {
                                                let item_path =
                                                    result.path().to_string_lossy().to_string();
                                                match Track::new(item_path.clone()) {
                                                    Ok(track) => {
                                                        tracks.push(track);
                                                    }
                                                    Err(
                                                        audiotags::error::Error::UnsupportedFormat(
                                                            e,
                                                        ),
                                                    ) => {
                                                        println!(
                                                            "File {} is unsupported by audiotags library, skipping... {}",
                                                            item_path, e
                                                        );
                                                    }
                                                    Err(e) => {
                                                        dbg!(&e);
                                                        println!(
                                                            "Error while importing {}: {}",
                                                            item_path, e
                                                        );
                                                        return Err(PlayerError::IoError);
                                                    }
                                                };
                                            }
                                            Err(err) => {
                                                println!("Fail {}", err);
                                                return Err(PlayerError::IoError);
                                            }
                                        }
                                    }
                                }
                                Err(err) => {
                                    println!("Fail {}", err);
                                    return Err(PlayerError::IoError);
                                }
                            };
                        }
                    }
                    Err(e) => {
                        println!("Error! {}", e);
                        return Err(PlayerError::IoError);
                    }
                }
            }
        }
        Err(e) => {
            println!("Error! {}", e);
            return Err(PlayerError::IoError);
        }
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

    return Ok(map);
}

pub fn update(state: &mut State, message: Message) {}

pub fn view(state: &State) -> Element<'static, Message> {
    let album_cover = if let Some(current_track) = &state.current_track && let Some(cover) = &current_track.track.cover {
        container(img(img::Handle::from_bytes(cover.to_vec())).width(512))
    } else {
        container("No Cover!")
    };

    let current_track = container(
        column![
            album_cover,
            row![
                button("previous"),
                button("play/pause"),
                button("next")
            ]
        ]
    );

    // let tracks: Column<Message> = state
    //     .tracks
    //     .clone()
    //     .into_iter()
    //     .map(|(album, tracks)| -> Element<Message> {
    //         let album_column: Column<Message> =
    //             tracks.into_iter().fold(Column::new(), |column, track| {
    //                 column.push(button(text(track.title.clone().unwrap())))
    //             });

    //         Column::new().push(text(album)).push(album_column).into()
    //     });

    let tracks: Column<Message> = state.tracks.clone()
        .into_iter().fold(
            Column::new(),
            |col, (album, tracks)| {
                let album_column: Column<Message> = tracks.into_iter().fold(Column::new(), |col, track| {
                    col.push(button(text(track.title.clone().unwrap())).on_press(Message::PlaySong(track)))
                });

                col.push(text(album)).push(album_column)
            });

    container(row![
        current_track,
        button("Pick Library").on_press(Message::PickLibrary),
        button("Scan Library").on_press(Message::ScanLibrary("/home/user/music".to_string())),
        // this lags the shit out of the app
        container(scrollable(tracks))
    ])
    .center_x(iced::Fill)
    .center_y(iced::Fill)
    .into()
}
