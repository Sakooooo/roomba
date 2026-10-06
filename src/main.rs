use std::collections::BTreeMap;

use crate::config::Config;
use crate::library::{Library, Track};
use crate::queue::Queue;
use discord_rich_presence::activity::Activity;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use iced::widget::{button, column, container, stack, text};
use iced::{Element, Task};
use platform_dirs::AppDirs;
use rfd::AsyncFileDialog;
use rusqlite::Connection;

mod config;
mod library;
mod player;
mod queue;

struct App {
    app_dirs: Option<AppDirs>,
    config: Config,
    library: Library,
    sorted_library: Library,
    queue: Queue,
    db_conn: Option<Connection>,
    player: player::Player,
    to_seek: Option<f32>,
    discord_rpc_client: Option<DiscordIpcClient>,
    search_query: String,
}

#[derive(Clone)]
enum Message {
    PickFolder,
    LibraryPicked(Option<rfd::FileHandle>),
    ScanLibrary(std::path::PathBuf),
    SaveLibrary(Library),
    PlayAlbumFrom { album: String, index: usize },
    AddToQueue(Track),
    RemoveFromQueue(usize),
    JumpToQueue(usize),
    PlayNext(Track),
    Next,
    Previous,
    TrackFinished,
    PlayPause,
    PlaybackTick,
    Seek(f32),
    ReleaseSeek,
    Volume(f32),
    SearchUpdated(String),
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

        let discord_rpc_client = {
            let mut client = DiscordIpcClient::new("1553362356375912458");

            match client.connect() {
                Ok(()) => Some(client),
                Err(e) => {
                    println!("Failed to connect to discord client! {}", e);
                    None
                }
            }
        };

        let mut app = App {
            app_dirs,
            config,
            queue: Queue::default(),
            sorted_library: library.clone(),
            library,
            db_conn,
            player,
            to_seek: None,
            discord_rpc_client,
            search_query: String::from(""),
        };

        app.update_discord_status();
        app
    }

    fn update_discord_status(&mut self) {
        let Some(client) = self.discord_rpc_client.as_mut() else {
            return;
        };

        let mut activity = Activity::new()
            .activity_type(activity::ActivityType::Listening)
            .status_display_type(activity::StatusDisplayType::Details)
            .assets(activity::Assets::new().large_image("roomba_grey"));

        if let Some(track) = &self.player.current_track {
            activity = activity
                .name("roomba")
                .details(format!("{} - {}", &track.title, &track.album_artist))
                .state(format!(
                    "Album {} on roomba {}",
                    &track.album_title,
                    env!("CARGO_PKG_VERSION")
                ));

            if !self.player.is_paused() {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                let start = now.saturating_sub(self.player.get_position());
                let mut ts = activity::Timestamps::new().start(start.as_secs() as i64);
                if let Some(total) = self.player.duration {
                    ts = ts.end((start + total).as_secs() as i64);
                }
                activity = activity.timestamps(ts);
            } else {
                activity = activity.state("Paused");
            }
        } else {
            activity = activity
                .details("roomba")
                .state("Not listening to anything currently...");
        }

        match client.set_activity(activity) {
            Ok(_) => println!("Successfully updated discord activity."),
            Err(e) => {
                println!("Failed to update discord status! {}", e);
                self.discord_rpc_client = None;
            }
        };
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        if !self.player.is_paused() && self.queue.current().is_some() {
            iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::PlaybackTick)
        } else {
            iced::Subscription::none()
        }
    }

    fn play_current_queue(&mut self) {
        let Some(track) = self.queue.current().cloned() else {
            return;
        };

        match self.player.play_path(&track.path) {
            Ok(_) => {
                println!("Playing {}", &track.title);

                let same = self
                    .player
                    .current_track
                    .as_ref()
                    .is_some_and(|prev| prev.album_title == track.album_title);

                if !same {
                    self.player.current_cover = Some(iced::widget::image::Handle::from_bytes(
                        track.get_cover_image(),
                    ));
                }
                self.player.current_track = Some(track);
                self.update_discord_status();
            }
            Err(e) => {
                println!("Failed to play track! {}", e)
            }
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
                self.sorted_library = self.library.clone();
                // Task::done(Message::SaveLibrary(library))
                Task::perform(Library::new_from_path_async(path), Message::SaveLibrary)
            }

            Message::SaveLibrary(library) => {
                if let Some(conn) = &self.db_conn {
                    library.save_to_db(conn);
                }
                Task::none()
            }
            Message::PlayPause => {
                self.player.toggle_pause();
                self.update_discord_status();
                Task::none()
            }
            Message::PlaybackTick => {
                if self.player.finished() {
                    Task::done(Message::TrackFinished)
                } else {
                    Task::none()
                }
            }
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
                self.update_discord_status();
                Task::none()
            }
            Message::Volume(vol) => {
                self.player.set_volume(vol);
                Task::none()
            }
            Message::PlayAlbumFrom { album, index } => {
                if let Some(tracks) = self.library.tracks.get(&album) {
                    self.queue.replace(tracks.clone(), index);
                    self.play_current_queue();
                }
                Task::none()
            }
            Message::AddToQueue(track) => {
                self.queue.append(track);
                Task::none()
            }
            Message::RemoveFromQueue(index) => {
                self.queue.remove(index);
                Task::none()
            }
            Message::PlayNext(track) => {
                self.queue.queue_next(track);
                Task::none()
            }
            Message::Next | Message::TrackFinished => {
                if self.queue.next().is_some() {
                    self.play_current_queue();
                } else {
                    self.player.stop();
                    self.update_discord_status();
                };
                Task::none()
            }
            Message::Previous => {
                // replay current song if 3s in
                // otherwise just go back normally
                // every player does this lol
                if self.player.get_position().as_secs() > 3 {
                    let _ = self.player.seek(std::time::Duration::ZERO);
                } else if self.queue.prev().is_some() {
                    self.play_current_queue();
                }
                Task::none()
            }
            Message::JumpToQueue(index) => {
                self.queue.jump_to(index);
                self.play_current_queue();
                Task::none()
            }
            Message::SearchUpdated(query) => {
                // TODO: Move this into an async fuction and call it with Task
                dbg!(&query);

                if query.is_empty() {
                    self.sorted_library = self.library.clone();
                    self.search_query = query;
                    return Task::none();
                }
                self.search_query = query.clone();

                let mut sorted: BTreeMap<String, Vec<Track>> = BTreeMap::new();

                if let Some(db_conn) = &self.db_conn {
                    let escaped = query
                        .replace("\\", "\\\\")
                        .replace("%", "\\%")
                        .replace("_", "\\_");
                    let pattern = format!("%{}%", escaped);

                    let mut result_query = match db_conn.prepare(
                        "SELECT path, title, track, album_title, album_artist FROM tracks as track
                       WHERE track.title LIKE ?1
                         OR track.album_title like ?1
                         OR track.album_artist like ?1",
                    ) {
                        Ok(q) => q,
                        Err(e) => {
                            println!("Failed to query db {}", e);
                            return Task::none();
                        }
                    };

                    let iter = result_query.query_map([&pattern], |row| {
                        Ok(Track {
                            path: row.get(0)?,
                            title: row.get(1)?,
                            track: row.get(2)?,
                            album_title: row.get(3)?,
                            album_artist: row.get(4)?,
                        })
                    });

                    match iter {
                        Ok(i) => {
                            for track in i {
                                if let Ok(track) = track {
                                    sorted
                                        .entry(track.album_title.clone())
                                        .or_default()
                                        .push(track);
                                }
                            }
                        }
                        Err(e) => {
                            println!("Failed to map rows to iter! {}", e);
                        }
                    }

                    self.sorted_library = Library { tracks: sorted };

                    Task::none()
                } else {
                    Task::none()
                }
            }
        }
    }

    fn tracks(&self) -> iced::widget::Column<'_, Message> {
        iced::widget::column![
            iced::widget::text_input("Search...", &self.search_query)
                .on_input(Message::SearchUpdated),
            // TODO: do something about this taking forever to load
            // probably try to create Space elements for unseen rows and only change what's visible when you scroll
            iced::widget::scrollable(iced::widget::column(self.sorted_library.tracks.iter().map(
                |(album, tracks)| {
                    container(iced::widget::column![
                        text(album),
                        iced::widget::column(tracks.iter().enumerate().map(|(index, track)| {
                            iced::widget::row![
                                button(text(track.title.as_str()))
                                    .on_press(Message::PlayAlbumFrom {
                                        album: album.clone(),
                                        index,
                                    })
                                    .width(iced::Fill),
                                button("play next").on_press(Message::PlayNext(track.clone())),
                                button("add to queue").on_press(Message::AddToQueue(track.clone())),
                            ]
                            .spacing(4)
                            .into()
                        }))
                    ])
                    .into()
                },
            )))
        ]
        .into()
    }

    fn now_playing(&self) -> Element<'_, Message> {
        let total = self.player.duration.unwrap_or_default().as_secs_f32();

        let position = self
            .to_seek
            .unwrap_or_else(|| self.player.get_position().as_secs_f32());

        container(column![
            container(self.player.current_cover.clone().map(iced::widget::image)).width(512),
            self.player
                .current_track
                .clone()
                .map(|t| text(format!("Now playing: {} - {}", t.album_title, t.title))),
            button("prev").on_press(Message::Previous),
            button(if self.player.is_paused() {
                "play"
            } else {
                "pause"
            })
            .on_press(Message::PlayPause),
            button("next").on_press(Message::Next),
            iced::widget::slider(0.0..=total.max(0.01), position, Message::Seek)
                .on_release(Message::ReleaseSeek)
                .step(0.1),
            iced::widget::slider(0.0..=1.0, self.player.volume, Message::Volume).step(0.025)
        ])
        .into()
    }

    fn queue_view(&self) -> Element<'_, Message> {
        let upcoming = self.queue.upcoming();
        if upcoming.is_empty() {
            return iced::widget::text("The queue is empty @_@").into();
        }

        let offset = self.queue.current.map(|i| i + 1).unwrap_or(0);

        column![
            text("next:"),
            iced::widget::scrollable(column(upcoming.iter().enumerate().map(|(i, track)| {
                iced::widget::row![
                    button(
                        text(format!("{} - {}", track.album_artist, track.title)).width(iced::Fill)
                    )
                    .on_press(Message::JumpToQueue(offset + i)),
                    button("remove").on_press(Message::RemoveFromQueue(offset + i)),
                ]
                .spacing(4)
                .into()
            })))
        ]
        .into()
    }

    fn view(&self) -> Element<'_, Message> {
        let left: iced::widget::Container<'_, Message> =
            container(column![self.now_playing(), self.queue_view()])
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
    let icon = iced::window::icon::from_file_data(
        include_bytes!("../assets/roomba_grey_square.png"),
        None,
    )
    .ok();
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .title("Roomba")
        .window(iced::window::Settings {
            icon,
            ..Default::default()
        })
        .run()
}
