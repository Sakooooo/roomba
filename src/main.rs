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
    to_seek: Option<f32>,
}

#[derive(Clone)]
enum Message {
    PickFolder,
    LibraryPicked(Option<rfd::FileHandle>),
    ScanLibrary(std::path::PathBuf),
    SaveLibrary(Library),
    PlayTrack(Track),
    PlayPause,
    PlaybackTick,
    Seek(f32),
    ReleaseSeek,
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

        let player: player::Player = player::Player::new();

        App {
            app_dirs,
            config,
            library,
            db_conn,
            player,
            to_seek: None,
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        if self.player.is_playing() {
            iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::PlaybackTick)
        } else {
            iced::Subscription::none()
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
                    Ok(_) => {
                        println!("Playing {}", &track.title);

                        let same = self
                            .player
                            .current_track
                            .as_ref()
                            .is_some_and(|prev| prev.album_title == track.album_title);

                        if !same {
                            self.player.current_cover = Some(
                                iced::widget::image::Handle::from_bytes(track.get_cover_image()),
                            );
                        }
                        self.player.current_track = Some(track);
                    }
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
            Message::PlaybackTick => Task::none(), // Redraws it
            Message::Seek(secs) => {
                self.to_seek = Some(secs);
                Task::none()
            }
            Message::ReleaseSeek => {
                if let Some(secs) = self.to_seek.take() {
                    if let Err(e) = self.player.seek(std::time::Duration::from_secs_f32(secs)) {
                        println!("Failed to seek {}", e);
                    }
                };
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
        let total = self.player.duration.unwrap_or_default().as_secs_f32();

        let position = self
            .to_seek
            .unwrap_or_else(|| self.player.get_position().as_secs_f32());

        let duration = self
            .player
            .duration
            .unwrap_or(std::time::Duration::new(0, 0))
            .as_secs_f32();
        container(column![
            self.player.current_cover.clone().map(iced::widget::image),
            text("Now playing"),
            button(if self.player.is_playing() {
                "pause"
            } else {
                "play"
            })
            .on_press(Message::PlayPause),
            // iced::widget::progress_bar(0.0..=duration, self.player.get_position().as_secs_f32())
            iced::widget::slider(0.0..=total.max(0.01), position, Message::Seek)
                .on_release(Message::ReleaseSeek)
                .step(0.1)
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
        .subscription(App::subscription)
        .title("Roomba")
        .run()
}
