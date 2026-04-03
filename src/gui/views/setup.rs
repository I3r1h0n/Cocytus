use iced::widget::{column, container, pick_list, row, svg, text, Space};
use iced::{Element, Length};

use crate::gui::app::App;
use crate::gui::truncate;
use crate::gui::theme::{self as t, styles};
use crate::gui::types::Message;
use crate::gui::widgets as w;

impl App {
    pub(super) fn view_setup(&self) -> Element<'_, Message> {
        let logo = svg(svg::Handle::from_memory(
            include_bytes!("../../../assets/logo.svg").as_slice(),
        ))
        .width(80)
        .height(80);

        let header = column![
            logo,
            text(crate::gui::APP_NAME).size(32).color(t::TEXT_PRIMARY),
            text(format!("v{}", crate::gui::APP_VERSION)).size(14).color(t::TEXT_MUTED),
            text(format!("by {}", crate::gui::APP_AUTHOR)).size(14).color(t::TEXT_MUTED),
        ]
        .spacing(4)
        .align_x(iced::Alignment::Center);

        let iso_section = self.iso_picker_section();
        let image_section = self.image_picker_section();

        let status_row: Element<'_, Message> = if self.status.is_empty() {
            Space::with_height(0).into()
        } else {
            text(&self.status).size(12).color(t::TEXT_MUTED).into()
        };

        let can_continue = self.iso_path.is_some() && self.selected_image.is_some();
        let continue_btn = if can_continue {
            w::action_button("Continue", Message::Continue).width(200)
        } else {
            w::disabled_button("Continue").width(200)
        };

        let form = column![
            iso_section,
            w::divider(),
            image_section,
            status_row,
            Space::with_height(4),
            container(continue_btn).center_x(Length::Fill),
        ]
        .spacing(16);

        let content = column![header, Space::with_height(24), w::card(form).width(420)]
            .align_x(iced::Alignment::Center)
            .width(Length::Fill);

        container(content)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(styles::page_style)
            .into()
    }

    fn iso_picker_section(&self) -> iced::widget::Column<'_, Message> {
        let display_text = self
            .iso_path
            .as_ref()
            .map(|p| truncate(&p.file_name().unwrap_or_default().to_string_lossy(), 35))
            .unwrap_or_else(|| "No file selected".to_string());
        let text_color = if self.iso_path.is_some() { t::TEXT_SECONDARY } else { t::TEXT_MUTED };

        column![
            text("Windows ISO File").size(14).color(t::TEXT_SECONDARY),
            row![
                text(display_text).size(14).color(text_color),
                Space::with_width(Length::Fill),
                w::dark_button("Browse...", Message::BrowseIso),
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(6)
    }

    fn image_picker_section(&self) -> iced::widget::Column<'_, Message> {
        let picker: Element<'_, Message> = if self.wim_images.is_empty() {
            let hint = if self.iso_path.is_some() && self.status.contains("Mounting") {
                "Loading images..."
            } else if self.iso_path.is_some() {
                "No images found"
            } else {
                "Select an ISO first"
            };
            text(hint).size(14).color(t::TEXT_MUTED).into()
        } else {
            pick_list(
                self.wim_images.clone(),
                self.selected_image.clone(),
                Message::ImageSelected,
            )
            .placeholder("Select an image...")
            .text_size(14)
            .padding([6, 10])
            .width(Length::Fill)
            .style(styles::pick_list_style)
            .menu_style(styles::pick_list_menu_style)
            .into()
        };

        column![text("WIM Image").size(14).color(t::TEXT_SECONDARY), picker].spacing(6)
    }
}
