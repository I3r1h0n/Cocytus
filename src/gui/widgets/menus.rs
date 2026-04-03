use iced::widget::{button, checkbox, column, container, mouse_area, row, text};
use iced::{Background, Border, Color, Element, Length};

use crate::gui::theme;
use crate::gui::{ContextMenuAction, FileMenuAction, Message};

/// Fixed width shared by all dropdown menus
const DROPDOWN_WIDTH: f32 = 150.0;

pub fn file_item<'a>(name: &str, selected: bool) -> Element<'a, Message> {
    let bg = if selected { theme::ACCENT_MUTED } else { Color::TRANSPARENT };
    let text_color = if selected { theme::TEXT_PRIMARY } else { theme::TEXT_SECONDARY };

    let name_owned = name.to_string();
    let name_for_ctx = name_owned.clone();

    let content = container(text(name_owned.clone()).size(12).color(text_color))
        .padding([2, 6])
        .width(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(bg)),
            border: Border { radius: 3.0.into(), color: Color::TRANSPARENT, width: 0.0 },
            ..Default::default()
        });

    mouse_area(content)
        .on_press(Message::SelectFile(name_owned))
        .on_right_press(Message::ShowContextMenu(name_for_ctx))
        .into()
}

pub fn context_menu_panel<'a>() -> Element<'a, Message> {
    let actions = [
        ContextMenuAction::Load,
        ContextMenuAction::ExportPdb,
        ContextMenuAction::ExportFile,
        ContextMenuAction::Details,
    ];
    let buttons: Vec<Element<'a, Message>> = actions
        .iter()
        .map(|a| 
            dropdown_button(&a.to_string(), Message::ContextAction(*a))
        )
        .collect();

    container(column(buttons).spacing(2).padding(5))
        .width(DROPDOWN_WIDTH)
        .style(|_theme| container::Style {
            background: Some(Background::Color(theme::BG_ELEMENT)),
            border: Border { radius: 5.0.into(), color: theme::BORDER, width: 1.0 },
            ..Default::default()
        })
        .into()
}

pub fn file_menu_dropdown<'a>() -> Element<'a, Message> {
    let items = vec![
        dropdown_button("Open new ISO", Message::FileMenu(FileMenuAction::OpenNewIso)),
        dropdown_button("Quit", Message::FileMenu(FileMenuAction::Quit)),
    ];

    container(column(items).spacing(2).padding(5))
        .width(DROPDOWN_WIDTH)
        .style(|_theme| container::Style {
            background: Some(Background::Color(theme::BG_ELEMENT)),
            border: Border { radius: 5.0.into(), color: theme::BORDER, width: 1.0 },
            ..Default::default()
        })
        .into()
}

pub fn symbol_item<'a>(
    name: &str,
    checked: bool,
    selected: bool,
    idx: usize,
) -> Element<'a, Message> {
    let bg = if selected { theme::ACCENT_MUTED } else { Color::TRANSPARENT };
    let text_color = if selected { theme::TEXT_PRIMARY } else { theme::TEXT_SECONDARY };

    let cb = checkbox("", checked)
        .on_toggle(move |_| Message::ToggleSymbolCheck(idx))
        .size(14)
        .spacing(0)
        .style(|_theme, status| {
            let (bg, border_color) = match status {
                checkbox::Status::Active { is_checked } | checkbox::Status::Hovered { is_checked } => {
                    if is_checked {
                        (theme::ACCENT_MUTED, theme::ACCENT)
                    } else {
                        (theme::BG_ELEMENT, theme::BORDER)
                    }
                }
                checkbox::Status::Disabled { .. } => (theme::BG_ELEMENT, theme::BORDER),
            };
            let icon_color = match status {
                checkbox::Status::Hovered { .. } => theme::TEXT_PRIMARY,
                _ => theme::ACCENT,
            };
            checkbox::Style {
                background: Background::Color(bg),
                icon_color,
                border: Border { radius: 3.0.into(), color: border_color, width: 1.0 },
                text_color: Some(theme::TEXT_SECONDARY),
            }
        });

    let label = text(name.to_string())
        .size(12)
        .color(text_color)
        .wrapping(text::Wrapping::None);

    let content = container(
        row![cb, label].spacing(6).align_y(iced::Alignment::Center),
    )
    .padding([2, 6])
    .width(Length::Fill)
    .height(Length::Shrink)
    .clip(true)
    .style(move |_theme| container::Style {
        background: Some(Background::Color(bg)),
        border: Border { radius: 3.0.into(), color: Color::TRANSPARENT, width: 0.0 },
        ..Default::default()
    });

    mouse_area(content)
        .on_press(Message::SelectSymbol(idx))
        .on_right_press(Message::ShowSymbolContextMenu(idx))
        .into()
}

pub fn symbol_context_menu_panel<'a>(idx: usize, is_checked: bool) -> Element<'a, Message> {
    let toggle_label = if is_checked { "Unselect" } else { "Select" };
    let items = vec![
        dropdown_button(toggle_label, Message::SymbolContextToggleCheck(idx)),
        dropdown_button("Open", Message::SelectSymbol(idx)),
        dropdown_button("Open in new tab", Message::OpenSymbolNewTab(idx)),
    ];

    container(column(items).spacing(2).padding(5))
        .width(DROPDOWN_WIDTH)
        .style(|_theme| container::Style {
            background: Some(Background::Color(theme::BG_ELEMENT)),
            border: Border { radius: 5.0.into(), color: theme::BORDER, width: 1.0 },
            ..Default::default()
        })
        .into()
}

pub fn json_context_menu_panel<'a>() -> Element<'a, Message> {
    let items = vec![
        dropdown_button("Copy", Message::JsonContextCopy),
        dropdown_button("Search symbol", Message::JsonContextSearch),
    ];

    container(column(items).spacing(2).padding(5))
        .width(DROPDOWN_WIDTH)
        .style(|_theme| container::Style {
            background: Some(Background::Color(theme::BG_ELEMENT)),
            border: Border { radius: 5.0.into(), color: theme::BORDER, width: 1.0 },
            ..Default::default()
        })
        .into()
}

fn dropdown_button<'a>(label: &str, msg: Message) -> Element<'a, Message> {
    button(text(label.to_string()).size(12).color(theme::TEXT_SECONDARY))
        .on_press(msg)
        .style(|_theme, status| {
            let bg = match status {
                button::Status::Hovered => theme::BG_ELEMENT_HOVER,
                button::Status::Pressed => theme::BG_ELEMENT_PRESSED,
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                border: Border { radius: 4.0.into(), color: Color::TRANSPARENT, width: 0.0 },
                text_color: theme::TEXT_SECONDARY,
                ..Default::default()
            }
        })
        .padding([4, 8])
        .width(Length::Fill)
        .into()
}
