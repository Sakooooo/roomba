use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

// what do you mean audiotags doesn't have support for opus lmfao
use multitag::{data::Album, Tag};
// use audiotags::{Album, Tag};

#[derive(Debug, Clone)]
pub struct Track {
    pub path: String,
    pub title: String,
    pub track: u16,
    pub album_title: String,
    pub album_artist: String,
}

impl Track {
    pub fn new_from_path(path: PathBuf) -> Result<Self, ()> {
        // let metadata = Tag::new().read_from_path(&path);
        let metadata = Tag::read_from_path(&path);
        // let metadata = Tag::read_from_path(&path);
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

#[derive(Clone)]
pub struct Library {
    // path: String,
    pub tracks: BTreeMap<String, Vec<Track>>,
}

const MIGRATIONS: &'static [&str] = &["CREATE TABLE tracks (
id INTEGER PRIMARY KEY,
path TEXT,
title TEXT,
track INTEGER,
album_title TEXT,
album_artist TEXT
);"];

fn migrate_db(conn: &rusqlite::Connection) {
    println!("Applying migrations...");

    let version: rusqlite::Result<u32, rusqlite::Error> =
        conn.query_one("PRAGMA user_version", [], |row| match row.get(0) {
            Ok(v) => Ok(v),
            Err(e) => {
                println!("Failed to query for user_version! {}", e);
                Err(e)
            }
        });

    if let Ok(version) = version {
        let length = MIGRATIONS.len() as u32;
        if version + 1 > length {
            println!("You are using a newer database, with an older version!");
            println!("Exiting...");
            panic!("Database is newer than maximum migration version");
        }

        if version + 1 != length {
            for migration_number in version..length {
                match conn.execute(MIGRATIONS[migration_number as usize], ()) {
                    Ok(_) => println!("Migration {} was successful", migration_number),
                    Err(e) => println!("failed to run migration {}, {}", migration_number, e),
                };

                let pragma_update = format!("PRAGMA user_version = {}", migration_number + 1);
                match conn.execute(&pragma_update, []) {
                    Ok(_) => println!(
                        "bump version to Migration {} was successful",
                        migration_number
                    ),
                    Err(e) => println!("failed to bump migration {}, {}", migration_number, e),
                };
            }
        }
    } else {
        println!("Failed to query version, {:#?}", version);
    };
}

impl Library {
    pub fn new_from_db(conn: &rusqlite::Connection) -> Self {
        migrate_db(&conn);

        let mut track_query = match conn
            .prepare("SELECT path, title, track, album_title, album_artist FROM tracks")
        {
            Ok(q) => {
                println!("Got one");
                q
            }
            Err(e) => {
                println!("Failed to prepare query {}", e);
                return Self {
                    tracks: BTreeMap::new(),
                };
            }
        };

        let track_iter = track_query.query_map([], |row| {
            let path: String = if let Ok(path) = row.get(0) {
                path
            } else {
                String::from("")
            };

            let title: String = if let Ok(title) = row.get(1) {
                title
            } else {
                String::from("Unknown track")
            };

            let track: u16 = if let Ok(track) = row.get(2) { track } else { 0 };

            let album_title: String = if let Ok(album_title) = row.get(3) {
                album_title
            } else {
                String::from("Unknown album")
            };

            let album_artist: String = if let Ok(album_artist) = row.get(4) {
                album_artist
            } else {
                String::from("Unknown artist")
            };

            println!("Transforming");

            Ok(Track {
                path,
                title,
                track,
                album_title,
                album_artist,
            })
        });

        println!("Making library");

        if let Ok(track_iter) = track_iter {
            let mut tracks: BTreeMap<String, Vec<Track>> = BTreeMap::new();

            println!("track iter");

            for track in track_iter {
                println!("track iter thing");
                match track {
                    Ok(t) => {
                        tracks
                            .entry(t.album_title.clone())
                            .or_insert_with(Vec::new)
                            .push(t);
                    }
                    Err(e) => println!("Failed to add a track! {}", e),
                }
            }
            Self { tracks }
        } else {
            // TODO: Error checking here
            println!("it didn't work lmao");

            Self {
                tracks: BTreeMap::new(),
            }
        }
    }

    pub fn save_to_db(&self, conn: &rusqlite::Connection) {
        for (album, tracks) in &self.tracks {
            for track in tracks {
                match conn.execute(
                    "INSERT INTO tracks (path, title, track, album_title, album_artist) VALUES (?1, ?2, ?3, ?4, ?5)",
                    (&track.path, &track.title, &track.track, &album, &track.album_artist)
                ) {
                    Ok(_) => println!("Successfully saved track {}", &track.title),
                    Err(e) => println!("Failed to save track! {}", e),
                };
            }
        }
    }

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

        Ok(Library { tracks })
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
