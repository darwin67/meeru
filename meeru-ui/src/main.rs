//! Meeru Desktop Application

use iced::{Application, Command, Element, Settings, Theme};

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    Meeru::run(Settings::default())
}

struct Meeru {
    // Application state will go here
}

#[derive(Debug, Clone)]
enum Message {
    // Application messages will go here
}

impl Application for Meeru {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self {}, Command::none())
    }

    fn title(&self) -> String {
        String::from("Meeru Email Client")
    }

    fn update(&mut self, _message: Message) -> Command<Message> {
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        "Hello, Meeru!".into()
    }
}
