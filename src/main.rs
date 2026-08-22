use iced::widget::{button, column, stack, text};
use iced::{Element, Task};
use platform_dirs::AppDirs;
use rfd::AsyncFileDialog;

use crate::config::Config;

mod config;
mod track;

struct App {
    app_dirs: Option<AppDirs>,
    config: Config,
}

#[derive(Clone)]
enum Message {
    PickFolder,
    ScanLibrary(Option<rfd::FileHandle>),
}

impl App {
    fn new() -> Self {
        let app_dirs = AppDirs::new(Some("roomba"), true);
        let config = Config::read_from_file_or_new(app_dirs.clone());
        App { app_dirs, config }
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
                Message::ScanLibrary,
            ),
            Message::ScanLibrary(file_handle) => {
                if let Some(handle) = file_handle {
                    dbg!(handle.path());
                    Task::none()
                } else {
                    println!("Hi");
                    Task::none()
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = column![button(text("Pick Library")).on_press(Message::PickFolder)];

        stack![content].into()
    }
}
fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Roomba")
        .run()
}
