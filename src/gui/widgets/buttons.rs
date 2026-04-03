use iced::widget::{button, container, mouse_area, row, text};
use iced::{Background, Border, Color, Element, Font};

use crate::gui::theme;
use crate::gui::Message;

pub fn dark_button<'a>(label: &'a str, msg: Message) -> iced::widget::Button<'a, Message> {
    button(text(label).size(14).color(theme::TEXT_SECONDARY).center())
        .on_press(msg)
        .style(|_theme, status| {
            let bg = match status {
                button::Status::Hovered => theme::BG_ELEMENT_HOVER,
                button::Status::Pressed => theme::BG_ELEMENT_PRESSED,
                _ => theme::BG_ELEMENT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                border: Border { radius: 6.0.into(), color: theme::BORDER, width: 1.0 },
                text_color: theme::TEXT_SECONDARY,
                ..Default::default()
            }
        })
        .padding([6, 16])
}

pub fn disabled_button<'a>(label: &'a str) -> iced::widget::Button<'a, Message> {
    button(text(label).size(14).color(theme::TEXT_DISABLED).center())
        .style(|_theme, _status| button::Style {
            background: Some(Background::Color(theme::BG_ELEMENT)),
            border: Border { radius: 6.0.into(), color: theme::BORDER, width: 1.0 },
            text_color: theme::TEXT_DISABLED,
            ..Default::default()
        })
        .padding([8, 24])
}

pub fn action_button<'a>(label: &'a str, msg: Message) -> iced::widget::Button<'a, Message> {
    button(text(label).size(14).color(theme::TEXT_PRIMARY).center())
        .on_press(msg)
        .style(|_theme, status| {
            let bg = match status {
                button::Status::Hovered => theme::BG_ELEMENT_HOVER,
                button::Status::Pressed => theme::BG_ELEMENT_PRESSED,
                _ => theme::BG_ELEMENT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                border: Border { radius: 6.0.into(), color: theme::BORDER, width: 1.0 },
                text_color: theme::TEXT_PRIMARY,
                ..Default::default()
            }
        })
        .padding([8, 24])
}

pub fn menu_button<'a>(label: &'a str, msg: Message) -> iced::widget::Button<'a, Message> {
    button(text(label).size(13).color(theme::TEXT_SECONDARY).center())
        .on_press(msg)
        .style(|_theme, status| {
            let bg = match status {
                button::Status::Hovered => theme::BG_ELEMENT_HOVER,
                button::Status::Pressed => theme::BG_ELEMENT_PRESSED,
                _ => iced::Color::TRANSPARENT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                border: Border { radius: 4.0.into(), color: iced::Color::TRANSPARENT, width: 0.0 },
                text_color: theme::TEXT_SECONDARY,
                ..Default::default()
            }
        })
        .padding([4, 12])
}

pub fn tab_button<'a>(label: &str, active: bool, msg: Message) -> Element<'a, Message> {
    let (text_color, bg_color, border_bottom) = if active {
        (theme::TEXT_PRIMARY, theme::BG_ELEMENT, theme::DIVIDER)
    } else {
        (theme::TEXT_MUTED, iced::Color::TRANSPARENT, iced::Color::TRANSPARENT)
    };

    button(text(label.to_string()).size(12).color(text_color).center())
        .on_press(msg)
        .style(move |_theme, status| {
            let bg = match status {
                button::Status::Hovered if !active => theme::BG_ELEMENT_HOVER,
                _ => bg_color,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                border: Border {
                    radius: iced::border::Radius::new(4.0).bottom(0.0),
                    color: border_bottom,
                    width: 1.0,
                },
                text_color,
                ..Default::default()
            }
        })
        .padding([4, 12])
        .into()
}

pub fn card<'a>(content: impl Into<Element<'a, Message>>) -> iced::widget::Container<'a, Message> {
    container(content)
        .padding(24)
        .style(|_theme| container::Style {
            background: Some(Background::Color(theme::BG_SURFACE)),
            border: Border { radius: 12.0.into(), color: theme::BORDER, width: 1.0 },
            ..Default::default()
        })
}

pub fn divider() -> iced::widget::Rule<'static> {
    use iced::widget::rule;
    iced::widget::horizontal_rule(1).style(|_theme| rule::Style {
        color: theme::DIVIDER,
        width: 1,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
    })
}

pub fn bold_font() -> Font {
    Font { weight: iced::font::Weight::Bold, ..Font::DEFAULT }
}

pub fn json_tab<'a>(label: &str, active: bool, idx: usize) -> Element<'a, Message> {
    let (text_color, bg_color) = if active {
        (theme::TEXT_PRIMARY, theme::BG_ELEMENT)
    } else {
        (theme::TEXT_MUTED, Color::TRANSPARENT)
    };

    let truncated = crate::gui::truncate(label, 25);

    let close = button(text("\u{00d7}").size(14).color(theme::TEXT_MUTED).center())
        .on_press(Message::CloseJsonTab(idx))
        .style(|_theme, status| {
            let bg = match status {
                button::Status::Hovered => theme::BG_ELEMENT_HOVER,
                button::Status::Pressed => theme::BG_ELEMENT_PRESSED,
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                border: Border { radius: 3.0.into(), color: Color::TRANSPARENT, width: 0.0 },
                text_color: theme::TEXT_MUTED,
                ..Default::default()
            }
        })
        .padding([0, 3]);

    let tab_content = row![
        text(truncated).size(11).color(text_color),
        close,
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center);

    let styled = container(tab_content)
        .padding([4, 8])
        .style(move |_theme| container::Style {
            background: Some(Background::Color(bg_color)),
            border: Border {
                radius: iced::border::Radius::new(4.0).bottom(0.0),
                color: if active { theme::DIVIDER } else { Color::TRANSPARENT },
                width: 1.0,
            },
            ..Default::default()
        });

    mouse_area(styled)
        .on_press(Message::SwitchJsonTab(idx))
        .into()
}

/// A two-option segmented toggle rendered as a single pill
pub fn segmented_toggle<'a>(
    left_label: &str,
    right_label: &str,
    left_active: bool,
    left_msg: Message,
    right_msg: Message,
) -> Element<'a, Message> {
    let left = {
        let active = left_active;
        let mut btn = button(
            text(left_label.to_string())
                .size(12)
                .color(if active { theme::TEXT_PRIMARY } else { theme::TEXT_MUTED })
                .center(),
        )
        .style(move |_theme, status| {
            let (bg, bc) = if active {
                (theme::ACCENT_MUTED, theme::ACCENT)
            } else {
                let bg = match status {
                    button::Status::Hovered => theme::BG_ELEMENT_HOVER,
                    button::Status::Pressed => theme::BG_ELEMENT_PRESSED,
                    _ => theme::BG_ELEMENT,
                };
                (bg, theme::BORDER)
            };
            button::Style {
                background: Some(Background::Color(bg)),
                border: Border {
                    radius: iced::border::Radius {
                        top_left: 4.0,
                        bottom_left: 4.0,
                        top_right: 0.0,
                        bottom_right: 0.0,
                    },
                    color: bc,
                    width: 1.0,
                },
                text_color: if active { theme::TEXT_PRIMARY } else { theme::TEXT_MUTED },
                ..Default::default()
            }
        })
        .padding([4, 12]);
        if !active {
            btn = btn.on_press(left_msg);
        }
        btn
    };

    let right = {
        let active = !left_active;
        let mut btn = button(
            text(right_label.to_string())
                .size(12)
                .color(if active { theme::TEXT_PRIMARY } else { theme::TEXT_MUTED })
                .center(),
        )
        .style(move |_theme, status| {
            let (bg, bc) = if active {
                (theme::ACCENT_MUTED, theme::ACCENT)
            } else {
                let bg = match status {
                    button::Status::Hovered => theme::BG_ELEMENT_HOVER,
                    button::Status::Pressed => theme::BG_ELEMENT_PRESSED,
                    _ => theme::BG_ELEMENT,
                };
                (bg, theme::BORDER)
            };
            button::Style {
                background: Some(Background::Color(bg)),
                border: Border {
                    radius: iced::border::Radius {
                        top_left: 0.0,
                        bottom_left: 0.0,
                        top_right: 4.0,
                        bottom_right: 4.0,
                    },
                    color: bc,
                    width: 1.0,
                },
                text_color: if active { theme::TEXT_PRIMARY } else { theme::TEXT_MUTED },
                ..Default::default()
            }
        })
        .padding([4, 12]);
        if !active {
            btn = btn.on_press(right_msg);
        }
        btn
    };

    row![left, right].into()
}
