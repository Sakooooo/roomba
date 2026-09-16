use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

// what do you mean audiotags doesn't have support for opus lmfao
use multitag::{Tag, data::Album};

#[derive(Debug)]
pub struct Track {
    pub path: String,
    pub title: String,
    pub track: u16,
    pub album_title: String,
    pub album_artist: String,
}

impl Track {
    pub fn new_from_path(path: PathBuf) -> Result<Self, ()> {
        let metadata = Tag::read_from_path(&path);
        if let Ok(metadata) = metadata {
            let title = metadata
                .title()
                .unwrap_or(&path.file_name().unwrap().to_str().unwrap())
                .to_string();

            // multitags doesn't have this??
            // let track = metadata.track_number().unwrap_or(0);
            let track = 0;

            // let album = metadata
            //     .album()
            //     .unwrap_or(Album::with_title("Unknown album"));

            let album = metadata.get_album_info().unwrap_or(Album::default());

            // let album_title = album.title.to_string();
            // let album_artist = album.artist.unwrap_or("Unknown Album Artist").to_string();

            let album_title = album.title.unwrap_or(String::from("Unknown album"));
            let album_artist = album.artist.unwrap_or(String::from("Unknown Album Artist"));

            Ok(Track {
                path: path.to_string_lossy().to_string(),
                title,
                track,
                album_title,
                album_artist,
            })
        } else {
            println!("Failed to read metadata");
            Err(())
        }
    }
}

pub struct Library {
    path: String,
    pub(crate) tracks: BTreeMap<String, Vec<Track>>,
}

impl Library {
    pub fn new_from_path(path: impl AsRef<Path>) -> Self {
        if let Ok(library) = Self::new_from_path_impl(path.as_ref()) {
            library
        } else {
            // TODO: Replace me with Result
            panic!("Failed to make library");
        }
    }

    fn new_from_path_impl(path: &Path) -> Result<Self, ()> {
        if !path.exists() {
            println!("Library path doesn't exist.");
            return Err(());
        }

        let tracks = Self::scan(path);

        let path_string = path.to_string_lossy().to_string();

        Ok(Library {
            path: path_string,
            tracks,
        })
    }

    // TODO: make this async
    pub fn scan(path: &Path) -> BTreeMap<String, Vec<Track>> {
        let mut queued_folders: Vec<PathBuf> = vec![path.to_owned()];
        let mut collected_files: Vec<PathBuf> = Vec::new();

        while !queued_folders.is_empty() {
            dbg!(&queued_folders);
            for entry in queued_folders[0]
                .read_dir()
                .expect("Failed to read directory")
                .flatten()
            {
                if entry.path().is_dir() {
                    println!("Folder");
                    dbg!(&entry);
                    queued_folders.push(entry.path());
                } else {
                    println!("File");
                    dbg!(&entry);
                    collected_files.push(entry.path());
                }
            }
            queued_folders.remove(0);
        }

        dbg!(&collected_files);

        let mut tracks: Vec<Track> = Vec::new();

        for file in collected_files {
            let track = Track::new_from_path(file);
            if let Ok(track) = track {
                dbg!(&track);
                tracks.push(track);
            }
        }

        let mut result: BTreeMap<String, Vec<Track>> = BTreeMap::new();

        for track in tracks {
            result
                .entry(track.album_title.clone())
                .or_insert_with(Vec::new)
                .push(track);
        }

        result
    }
}
