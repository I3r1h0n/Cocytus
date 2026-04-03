use iced::widget::{checkbox, container, overlay::menu, pick_list, scrollable, text_editor, text_input};
use iced::{Background, Border, Color};

use crate::gui::theme;

// Pick list

pub fn pick_list_style(_theme: &iced::Theme, status: pick_list::Status) -> pick_list::Style {
    let bg = match status {
        pick_list::Status::Hovered | pick_list::Status::Opened => theme::BG_ELEMENT_HOVER,
        _ => theme::BG_ELEMENT,
    };
    pick_list::Style {
        text_color: theme::TEXT_SECONDARY,
        placeholder_color: theme::TEXT_MUTED,
        handle_color: theme::TEXT_MUTED,
        background: Background::Color(bg),
        border: Border { radius: 6.0.into(), color: theme::BORDER, width: 1.0 },
    }
}

pub fn pick_list_menu_style(_theme: &iced::Theme) -> menu::Style {
    menu::Style {
        background: Background::Color(theme::BG_SURFACE),
        border: Border { radius: 6.0.into(), color: theme::BORDER, width: 1.0 },
        text_color: theme::TEXT_SECONDARY,
        selected_text_color: theme::TEXT_PRIMARY,
        selected_background: Background::Color(theme::ACCENT_MUTED),
    }
}

// Text input

pub fn search_input_style(_theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let bg = match status {
        text_input::Status::Focused | text_input::Status::Hovered => theme::BG_ELEMENT_HOVER,
        _ => theme::BG_ELEMENT,
    };
    text_input::Style {
        background: Background::Color(bg),
        border: Border { radius: 4.0.into(), color: theme::BORDER, width: 1.0 },
        icon: theme::TEXT_MUTED,
        placeholder: theme::TEXT_MUTED,
        value: theme::TEXT_SECONDARY,
        selection: theme::ACCENT_MUTED,
    }
}

// Scrollable

pub fn scrollable_style(_theme: &iced::Theme, status: scrollable::Status) -> scrollable::Style {
    let scroller_color = match status {
        scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. } => theme::TEXT_MUTED,
        _ => theme::BG_ELEMENT_HOVER,
    };
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail {
            background: None,
            border: Border { radius: 3.0.into(), color: Color::TRANSPARENT, width: 0.0 },
            scroller: scrollable::Scroller {
                color: scroller_color,
                border: Border { radius: 3.0.into(), color: Color::TRANSPARENT, width: 0.0 },
            },
        },
        horizontal_rail: scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller { color: Color::TRANSPARENT, border: Border::default() },
        },
        gap: None,
    }
}

pub fn tab_scrollable_style(_theme: &iced::Theme, status: scrollable::Status) -> scrollable::Style {
    let scroller_color = match status {
        scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. } => theme::TEXT_MUTED,
        _ => theme::BG_ELEMENT_HOVER,
    };
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller { color: Color::TRANSPARENT, border: Border::default() },
        },
        horizontal_rail: scrollable::Rail {
            background: None,
            border: Border { radius: 3.0.into(), color: Color::TRANSPARENT, width: 0.0 },
            scroller: scrollable::Scroller {
                color: scroller_color,
                border: Border { radius: 3.0.into(), color: Color::TRANSPARENT, width: 0.0 },
            },
        },
        gap: None,
    }
}

// Checkbox

pub fn filter_checkbox_style(
    _theme: &iced::Theme,
    status: checkbox::Status,
) -> checkbox::Style {
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
}

// JSON editor

pub fn json_editor_style(_theme: &iced::Theme, _status: text_editor::Status) -> text_editor::Style {
    text_editor::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border { radius: 0.0.into(), color: Color::TRANSPARENT, width: 0.0 },
        icon: theme::TEXT_MUTED,
        placeholder: theme::TEXT_MUTED,
        value: theme::TEXT_SECONDARY,
        selection: theme::ACCENT_MUTED,
    }
}

// Containers

pub fn header_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme::BG_SURFACE)),
        border: Border { width: 1.0, radius: 0.0.into(), color: theme::BORDER },
        ..Default::default()
    }
}

pub fn footer_style(_theme: &iced::Theme) -> container::Style {
    header_style(_theme)
}

pub fn page_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme::BG_BASE)),
        ..Default::default()
    }
}

pub fn divider_style(_theme: &iced::Theme) -> iced::widget::rule::Style {
    iced::widget::rule::Style {
        color: theme::BORDER,
        width: 1,
        radius: 0.0.into(),
        fill_mode: iced::widget::rule::FillMode::Full,
    }
}
