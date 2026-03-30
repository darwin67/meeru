//! Meeru Desktop Application

use iced::{Element, Task};

pub fn main() -> iced::Result {
    meeru_core::logging::init_logging();

    tracing::info!("Starting Meeru desktop application");

    iced::application(Meeru::new, Meeru::update, Meeru::view)
        .title("Meeru Email Client")
        .run()
}

struct Meeru;

#[derive(Debug, Clone)]
enum Message {
    // Application messages will go here
}

impl Meeru {
    fn new() -> Self {
        Self
    }

    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        iced::widget::text("Hello, Meeru!").into()
    }
}
