use std::fs::DirEntry;
use std::path::Path;

use crate::{Message, State};
use audiotags::{Album, AudioTag, MimeType, Tag};
use iced::application::IntoBoot;
use iced::futures::lock::MutexGuard;
use iced::widget::{button, column, container, image as img, text};
use iced::Element;
use image::DynamicImage;

pub struct CurrentTrack {
    track: Track, // state
}

pub enum PlayerError {
    LibraryNotFound,
    IoError,
}

#[derive(Debug, Default, Clone)]
pub struct Track {
    title: Option<String>,
    artist: Option<String>,
    album_artist: Option<String>,
    album_title: Option<String>,
    cover: Option<Vec<u8>>,
}

impl Track {
    pub fn new(path: String) -> Result<Self, audiotags::error::Error> {
        let mut result = Self::default();

        let metadata = Tag::new().read_from_path(path)?;

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

        Ok(result)
    }
}

pub async fn scan_library(path: String) -> Result<Vec<Track>, PlayerError> {
    let library_path = Path::new(&path);
    if !library_path.exists() {
        println!("fail");
        return Err(PlayerError::LibraryNotFound);
    }

    let mut tracks: Vec<Track> = vec![];

    match library_path.read_dir() {
        Ok(files) => {
            dbg!(&files);
            for file in files {
                match file {
                    Ok(entry) => {
                        let mut collected_folders: Vec<DirEntry> = Vec::new();
                        match entry.file_type() {
                            Ok(info) => {
                                dbg!(&info);
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

                        dbg!(&collected_folders);
                        for dir in collected_folders {
                            match dir.path().read_dir() {
                                Ok(collected_folder) => {
                                    dbg!(&collected_folder);
                                    for file in collected_folder {
                                        match file {
                                            Ok(result) => {
                                                let item_path =
                                                    result.path().to_string_lossy().to_string();
                                                match Track::new(item_path.clone()) {
                                                    Ok(track) => {
                                                        dbg!(&track.title);
                                                        tracks.push(track);
                                                    }
                                                    Err(
                                                        audiotags::error::Error::UnsupportedFormat(
                                                            e,
                                                        ),
                                                    ) => {
                                                        println!("File {} is unsupported by audiotags library, skipping... {}", item_path, e);
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

    dbg!(&tracks.len());
    return Ok(tracks);
}

pub fn update(state: &mut State, message: Message) {}

pub fn view(state: &State) -> Element<'static, Message> {
    column![
        {
            if let Some(current_track) = &state.current_track
                && let Some(album_cover) = &current_track.track.cover
            {
                container(img(img::Handle::from_bytes(album_cover.to_vec())))
            } else {
                container("no album cover")
            }
        },
        button("Scan Library").on_press(Message::ScanLibrary("/home/user/music".to_string()))
    ]
    .into()
}
