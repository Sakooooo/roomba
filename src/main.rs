use std::collections::BTreeMap;

use crate::config::Config;
use crate::library::{Library, Track};
use iced::widget::{button, column, container, stack, text};
use iced::{Element, Task};
use platform_dirs::AppDirs;
use rfd::AsyncFileDialog;
use rusqlite::Connection;

mod config;
mod library;
mod player;

struct App {
    app_dirs: Option<AppDirs>,
    config: Config,
    library: Library,
    db_conn: Option<Connection>,
    player: player::Player,
}

#[derive(Clone)]
enum Message {
    PickFolder,
    LibraryPicked(Option<rfd::FileHandle>),
    ScanLibrary(std::path::PathBuf),
    SaveLibrary(Library),
    PlayTrack(Track),
    PlayPause,
}

impl App {
    fn new() -> Self {
        let app_dirs = AppDirs::new(Some("roomba"), true);
        let config = Config::read_from_file_or_new(app_dirs.clone());

        let db_path: std::path::PathBuf = if let Some(app_dirs) = &app_dirs {
            if !&app_dirs.data_dir.exists() {
                match std::fs::create_dir_all(&app_dirs.data_dir) {
                    Ok(_) => println!(
                        "Managed to create data directory at {:#?}",
                        &app_dirs.data_dir
                    ),
                    Err(e) => {
                        println!("failed to create data directory {}", e);
                    }
                };
            }
            std::path::Path::join(&app_dirs.data_dir, "roomba.db")
        } else {
            std::path::Path::new("./roomba.db").to_path_buf()
        };

        let db_conn = match Connection::open(db_path) {
            Ok(c) => Some(c),
            Err(e) => {
                println!(
                    "Failed to connect to database! Library will not load. {}",
                    e
                );
                None
            }
        };

        let library = if let Some(conn) = &db_conn {
            Library::new_from_db(conn)
        } else {
            Library {
                tracks: BTreeMap::new(),
            }
        };

        let player = player::Player::new();

        App {
            app_dirs,
            config,
            library,
            db_conn,
            player,
        }
    }
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PickFolder => Task::perform(
                AsyncFileDialog::new()
                    .set_directory(
                        std::env::home_dir()
                            .unwrap_or(std::path::Path::new("/").to_path_buf())
                            .to_string_lossy()
                            .to_string(),
                    )
                    .pick_folder(),
                Message::LibraryPicked,
            ),
            Message::LibraryPicked(file_handle) => {
                if let Some(handle) = file_handle {
                    dbg!(&handle.path());
                    Task::done(Message::ScanLibrary(handle.path().to_path_buf()))
                } else {
                    println!("Hi");
                    Task::none()
                }
            }
            Message::ScanLibrary(path) => {
                dbg!(&path);

                let library = Library::new_from_path(path);
                self.library = library.clone();
                Task::done(Message::SaveLibrary(library))
            }
            Message::SaveLibrary(library) => {
                if let Some(conn) = &self.db_conn {
                    library.save_to_db(conn);
                }
                Task::none()
            }
            Message::PlayTrack(track) => {
                match self.player.play_path(&track.path) {
                    Ok(_) => println!("Playing {}", &track.title),
                    Err(e) => {
                        println!("Failed to play track! {}", e)
                    }
                }
                Task::none()
            }
            Message::PlayPause => {
                self.player.toggle_pause();
                Task::none()
            }
        }
    }

    fn tracks(&self) -> iced::widget::Scrollable<'_, Message> {
        iced::widget::scrollable(iced::widget::column(self.library.tracks.iter().map(
            |(album, tracks)| {
                container(iced::widget::column![
                    text(album),
                    iced::widget::column(tracks.iter().map(|track| {
                        button(text(track.title.clone()))
                            .on_press(Message::PlayTrack(track.clone()))
                            .width(iced::Fill)
                            .into()
                    }))
                ])
                .into()
            },
        )))
        .into()
    }

    fn now_playing(&self) -> Element<'_, Message> {
        container(column![
            text("Now playing"),
            button("pause play button").on_press(Message::PlayPause)
        ])
        .into()
    }

    fn view(&self) -> Element<'_, Message> {
        let left: iced::widget::Container<'_, Message> = container(column![self.now_playing()])
            .align_left(iced::Fill)
            .height(iced::Fill);

        let right: iced::widget::Container<'_, Message> =
            container(column![
                button(text("Pick Library")).on_press(Message::PickFolder),
                button(text("I have to use this button because for some reason, my xdg portal is broken and I don't know why!")).on_press(Message::ScanLibrary(std::path::Path::new("/home/user/Music").into())),
                self.tracks()
            ]).align_right(iced::Fill).height(iced::Fill);

        let content = iced::widget::row![left, right];

        stack![content].into()
    }
}
fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Roomba")
        .run()
}
