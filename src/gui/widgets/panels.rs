use iced::widget::container;
use iced::{Background, Border, Element, Length};

use crate::gui::theme;
use crate::gui::Message;

pub fn panel<'a>(content: impl Into<Element<'a, Message>>) -> iced::widget::Container<'a, Message> {
    container(content)
        .padding(8)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(theme::BG_SURFACE)),
            border: Border { radius: 0.0.into(), color: theme::BORDER, width: 1.0 },
            ..Default::default()
        })
}

pub fn panel_no_right_pad<'a>(content: impl Into<Element<'a, Message>>) -> iced::widget::Container<'a, Message> {
    container(content)
        .padding(iced::Padding { top: 8.0, right: 0.0, bottom: 8.0, left: 8.0 })
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(theme::BG_SURFACE)),
            border: Border { radius: 0.0.into(), color: theme::BORDER, width: 1.0 },
            ..Default::default()
        })
}

pub fn pane_header<'a>(content: impl Into<Element<'a, Message>>) -> iced::widget::Container<'a, Message> {
    container(content)
        .padding([4, 8])
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: None,
            border: Border { radius: 0.0.into(), color: theme::BORDER, width: 0.0 },
            ..Default::default()
        })
}
