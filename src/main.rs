use std::collections::BTreeMap;

use crate::config::Config;
use crate::library::Library;
use iced::widget::{button, column, container, scrollable, stack, text};
use iced::{Element, Task};
use platform_dirs::AppDirs;
use rfd::AsyncFileDialog;

mod config;
mod track;

struct App {
    app_dirs: Option<AppDirs>,
    config: Config,
    loaded_libraries: Vec<Library>,
}

mod library;

#[derive(Clone)]
enum Message {
    PickFolder,
    LibraryPicked(Option<rfd::FileHandle>),
    ScanLibrary(std::path::PathBuf),
}

impl App {
    fn new() -> Self {
        let app_dirs = AppDirs::new(Some("roomba"), true);
        let config = Config::read_from_file_or_new(app_dirs.clone());
        let loaded_libraries = Vec::new();
        App {
            app_dirs,
            config,
            loaded_libraries,
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
                    dbg!(handle.path());
                    Task::none()
                } else {
                    println!("Hi");
                    Task::none()
                }
            }
            Message::ScanLibrary(path) => {
                dbg!(&path);
                self.loaded_libraries.push(Library::new_from_path(path));
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let tracks: iced::widget::Scrollable<'_, Message> = {
            let mut col = iced::widget::Column::new();
            for library in &self.loaded_libraries {
                for (album, list) in &library.tracks {
                    col = col.push(container(text(album.clone())).height(25));
                    for track in list {
                        col = col.push(button(text(track.title.clone()).width(iced::Fill)));
                    }
                }
            }
            scrollable(col)
        };
        let left: iced::widget::Container<'_, Message> =
            container(text("Left")).align_left(iced::Fill);

        let right: iced::widget::Container<'_, Message> =
            container(column![
                button(text("Pick Library")).on_press(Message::PickFolder),
                button(text("I have to use this button because for some reason, my xdg portal is broken and I don't know why!")).on_press(Message::ScanLibrary(std::path::Path::new("/home/user/Music").into())),
                tracks
            ]).align_right(iced::Fill);

        let content = column![left, right];

        stack![content].into()
    }
}
fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Roomba")
        .run()
}
