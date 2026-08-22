use std::path::Path;

use audiotags::Tag;

pub struct Track {
    name: String,
    artist: String,
    album: String,
}

impl Track {
    /// reads the metadata of an audio file from a filepath and turns it into a struct
    pub fn new_from_path(
        path: impl AsRef<Path> + std::fmt::Display,
    ) -> Result<Self, audiotags::error::Error> {
        let metadata = match Tag::new().read_from_path(&path) {
            Ok(m) => m,
            Err(e) => {
                println!("Failed to read {}: {}", path, e);
                return Err(e);
            }
        };

        let name = metadata.title().unwrap_or("Unknown track").to_string();
        let artist = metadata.artist().unwrap_or("Unknown Artist").to_string();

        let album;

        if let Some(album_metadata) = metadata.album() {
            album = album_metadata.title.to_string();
        } else {
            album = String::from("Unknown album");
        }

        let track = Track {
            name,
            artist,
            album,
        };

        Ok(track)
    }
}
