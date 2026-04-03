mod main;
mod setup;

use iced::{Element, Subscription};

use super::app::App;
use super::types::Message;

impl App {
    pub(crate) fn view(&self) -> Element<'_, Message> {
        match self.screen {
            super::app::Screen::Setup => self.view_setup(),
            super::app::Screen::Main => self.view_main(),
        }
    }

    pub(crate) fn subscription(&self) -> Subscription<Message> {
        iced::event::listen_with(|event, _status, _id| {
            if let iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) = event {
                Some(Message::CursorMoved(position))
            } else {
                None
            }
        })
    }
}
