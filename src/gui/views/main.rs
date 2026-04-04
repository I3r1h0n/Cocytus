use iced::widget::{
    button, checkbox, column, container, horizontal_rule, mouse_area, pane_grid, progress_bar,
    row, scrollable, svg, text, text_editor, text_input, Column, Row, Space, Stack, center,
};
use iced::{Background, Border, Color, Element, Length};

use crate::gui::app::{App, PaneVariant};
use crate::gui::theme::cpp_highlight::{self, CppHighlighter};
use crate::gui::theme::json_highlight::{self, JsonHighlighter};
use crate::gui::theme::{self as t, styles};
use crate::gui::types::{Message, SymbolKind, SymbolTab, ViewMode};
use crate::gui::widgets as w;
use crate::gui::{MAX_VISIBLE_FILES, MAX_VISIBLE_SYMBOLS};

impl App {
    pub(super) fn view_main(&self) -> Element<'_, Message> {
        let header = self.menu_bar();
        let file_bar = self.file_info_bar();
        let pane_grid_widget = self.pane_grid();

        let footer = container(
            text(format!("{} v{}", crate::gui::APP_NAME, crate::gui::APP_VERSION))
                .size(12)
                .color(t::TEXT_MUTED),
        )
        .padding([4, 8])
        .width(Length::Fill)
        .style(styles::footer_style);

        let layout = column![
            header,
            file_bar,
            container(pane_grid_widget).width(Length::Fill).height(Length::Fill),
            footer,
        ];

        let page = container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(styles::page_style);

        let dismiss_layer = self.dismiss_layer();
        let overlay_layer = self.overlay_layer();

        Stack::with_children(vec![page.into(), dismiss_layer, overlay_layer]).into()
    }

    // Top bar sections

    fn menu_bar(&self) -> iced::widget::Container<'_, Message> {
        let bar = row![
            w::menu_button("File", Message::ToggleFileMenu),
            w::menu_button("Options", Message::ShowOptions),
            w::menu_button("About", Message::ShowAbout),
        ]
        .spacing(2)
        .padding([4, 8])
        .align_y(iced::Alignment::Center);

        container(bar).width(Length::Fill).style(styles::header_style)
    }

    fn file_info_bar(&self) -> iced::widget::Container<'_, Message> {
        let (label_text, active) = if let Some(data) = &self.pdb_data {
            (format!("Loaded: {}", data.file_name), true)
        } else {
            ("No symbols loaded".to_string(), false)
        };

        let label = text(label_text)
            .size(12)
            .color(if active { t::TEXT_SECONDARY } else { t::TEXT_MUTED });

        let mut export_btn = button(
            text("Export")
                .size(13)
                .color(if active { t::TEXT_SECONDARY } else { t::TEXT_DISABLED })
                .center(),
        )
        .style(|_theme, status| {
            let bg = match status {
                button::Status::Hovered => t::BG_ELEMENT_HOVER,
                button::Status::Pressed => t::BG_ELEMENT_PRESSED,
                _ => t::BG_ELEMENT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                border: Border { radius: 4.0.into(), color: t::BORDER, width: 1.0 },
                text_color: t::TEXT_SECONDARY,
                ..Default::default()
            }
        })
        .padding([4, 20]);

        if active {
            export_btn = export_btn.on_press(Message::ShowExportDialog);
        }

        let view_toggle: Element<'_, Message> = if active {
            w::segmented_toggle(
                "JSON",
                "C/C++",
                self.view_mode == ViewMode::Json,
                Message::SetViewMode(ViewMode::Json),
                Message::SetViewMode(ViewMode::Cpp),
            )
        } else {
            Space::with_width(0).into()
        };

        container(
            row![
                label,
                Space::with_width(Length::Fill),
                view_toggle,
                Space::with_width(8),
                export_btn,
            ]
            .align_y(iced::Alignment::Center)
            .padding([4, 8]),
        )
        .width(Length::Fill)
        .style(styles::header_style)
    }

    // Pane grid

    fn pane_grid(&self) -> pane_grid::PaneGrid<'_, Message> {
        let pe_files = &self.pe_files;
        let pe_filter = &self.pe_filter;
        let selected_file = &self.selected_file;
        let context_menu_file = &self.context_menu_file;
        let pdb_data = &self.pdb_data;
        let symbol_filter = &self.symbol_filter;
        let selected_symbol_idx = &self.selected_symbol_idx;
        let symbol_checks = &self.symbol_checks;
        let loading_pdb = self.loading_pdb;
        let pdb_status = &self.pdb_status;
        let json_content = &self.json_content;
        let has_json = self.selected_symbol_idx.is_some();
        let loading_json = self.loading_json;
        let show_structs = self.show_structs;
        let show_functions = self.show_functions;
        let show_enums = self.show_enums;
        let symbol_tab = self.symbol_tab;
        let json_tabs = &self.json_tabs;
        let context_menu_symbol_idx = &self.context_menu_symbol_idx;
        let view_mode = self.view_mode;

        pane_grid::PaneGrid::new(&self.panes, move |_id, variant, _maximized| {
            let body: Element<'_, Message> = match variant {
                PaneVariant::LeftTop => view_symbol_pane(
                    pdb_data,
                    symbol_filter,
                    selected_symbol_idx,
                    symbol_checks,
                    loading_pdb,
                    pdb_status,
                    show_structs,
                    show_functions,
                    show_enums,
                    symbol_tab,
                    context_menu_symbol_idx,
                ),
                PaneVariant::LeftBottom => view_file_pane(
                    pe_files,
                    pe_filter,
                    selected_file,
                    context_menu_file,
                ),
                PaneVariant::Right => view_json_pane(
                    pdb_data,
                    selected_symbol_idx,
                    json_content,
                    has_json,
                    loading_json,
                    json_tabs,
                    view_mode,
                ),
            };

            match variant {
                PaneVariant::LeftBottom | PaneVariant::LeftTop => {
                    pane_grid::Content::new(w::panel_no_right_pad(body))
                }
                _ => pane_grid::Content::new(w::panel(body)),
            }
        })
        .on_resize(6, Message::PaneResized)
        .spacing(2)
    }

    // Overlay layers

    fn dismiss_layer(&self) -> Element<'_, Message> {
        let dismiss_msg = if self.show_iso_info {
            Some(Message::DismissIsoInfo)
        } else if self.export_dialog.is_some() {
            if self.export_dialog.as_ref().is_some_and(|d| !d.exporting) {
                Some(Message::DismissExportDialog)
            } else {
                Some(Message::Noop)
            }
        } else if self.options_dialog.is_some() {
            Some(Message::DismissOptions)
        } else if self.pe_details.is_some() {
            Some(Message::DismissPeDetails)
        } else if self.show_about {
            Some(Message::DismissAbout)
        } else if self.file_menu_open {
            Some(Message::ToggleFileMenu)
        } else if self.context_menu_file.is_some() {
            Some(Message::DismissContextMenu)
        } else if self.json_context_menu {
            Some(Message::DismissJsonContextMenu)
        } else if self.context_menu_symbol_idx.is_some() {
            Some(Message::DismissSymbolContextMenu)
        } else {
            None
        };

        if let Some(msg) = dismiss_msg {
            mouse_area(
                container(Space::new(Length::Fill, Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .on_press(msg)
            .into()
        } else {
            Space::new(0, 0).into()
        }
    }

    fn overlay_layer(&self) -> Element<'_, Message> {
        if self.show_iso_info {
            return self.iso_info_dialog();
        }
        if self.export_dialog.is_some() {
            return self.export_dialog_view();
        }
        if self.options_dialog.is_some() {
            return self.options_dialog_view();
        }
        if self.pe_details.is_some() {
            return self.pe_details_dialog();
        }
        if self.show_about {
            return self.about_dialog();
        }
        if self.file_menu_open {
            container(w::file_menu_dropdown())
                .padding(iced::Padding { top: 32.0, right: 0.0, bottom: 0.0, left: 8.0 })
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else if self.context_menu_file.is_some() {
            let pos = self.context_menu_pos;
            container(w::context_menu_panel())
                .padding(iced::Padding { top: pos.y, right: 0.0, bottom: 0.0, left: pos.x })
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else if self.json_context_menu {
            let pos = self.json_context_menu_pos;
            container(w::json_context_menu_panel())
                .padding(iced::Padding { top: pos.y, right: 0.0, bottom: 0.0, left: pos.x })
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else if let Some(idx) = self.context_menu_symbol_idx {
            let pos = self.context_menu_pos;
            let is_checked = self.symbol_checks.contains(&idx);
            container(w::symbol_context_menu_panel(idx, is_checked))
                .padding(iced::Padding { top: pos.y, right: 0.0, bottom: 0.0, left: pos.x })
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            Space::new(0, 0).into()
        }
    }

    fn iso_info_dialog(&self) -> Element<'_, Message> {
        let file_name = self
            .iso_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".into());

        let file_path = self
            .iso_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".into());

        let file_size = self
            .iso_file_size
            .map(format_file_size)
            .unwrap_or_else(|| "Unknown".into());

        let drive = self
            .mount_info
            .as_ref()
            .map(|i| format!("{}\\", i.drive))
            .unwrap_or_else(|| "Unknown".into());

        let wim_path = self
            .mount_info
            .as_ref()
            .map(|i| i.wim_path.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".into());

        let image_name = self
            .selected_image
            .as_ref()
            .map(|i| i.to_string())
            .unwrap_or_else(|| "Unknown".into());

        let pe_count = format!("{}", self.pe_files.len());

        let mut info_col = Column::new().spacing(6);
        info_col = info_col.push(text("ISO Loaded Successfully").size(16).color(t::ACCENT));
        info_col = info_col.push(Space::with_height(12));
        for (label, value) in [
            ("File name:", file_name),
            ("Path:", file_path),
            ("File size:", file_size),
            ("Mounted at:", drive),
            ("WIM file:", wim_path),
            ("Image:", image_name),
            ("PE files:", pe_count),
        ] {
            info_col = info_col.push(
                row![
                    text(label).size(12).color(t::TEXT_MUTED).width(100),
                    text(value).size(12).color(t::TEXT_SECONDARY),
                ]
                .spacing(8),
            );
        }
        info_col = info_col.push(Space::with_height(16));
        info_col = info_col.push(
            container(w::action_button("Ok", Message::DismissIsoInfo).width(120))
                .center_x(Length::Fill),
        );

        let dialog_content = info_col;

        let dialog_card = container(dialog_content)
            .padding(24)
            .max_width(480)
            .style(|_theme| container::Style {
                background: Some(Background::Color(t::BG_SURFACE)),
                border: Border {
                    radius: 12.0.into(),
                    color: t::BORDER,
                    width: 1.0,
                },
                ..Default::default()
            });

        // Semi-transparent backdrop + centered card
        container(center(dialog_card))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.6))),
                ..Default::default()
            })
            .into()
    }

    fn about_dialog(&self) -> Element<'_, Message> {
        let logo = svg(svg::Handle::from_memory(
            include_bytes!("../../../assets/logo.svg").as_slice(),
        ))
        .width(64)
        .height(64);

        let name = text(crate::gui::APP_NAME).size(28).color(t::TEXT_PRIMARY);
        let version = text(format!("v{}", crate::gui::APP_VERSION))
            .size(13)
            .color(t::TEXT_MUTED);
        let author = text(format!("by {}", crate::gui::APP_AUTHOR))
            .size(13)
            .color(t::TEXT_MUTED);

        let github_icon = svg(svg::Handle::from_memory(
            include_bytes!("../../../assets/github.svg").as_slice(),
        ))
        .width(16)
        .height(16);

        let github_btn = button(
            row![github_icon, text("GitHub").size(13).color(t::ACCENT),]
                .spacing(6)
                .align_y(iced::Alignment::Center),
        )
        .on_press(Message::OpenUrl(
            "https://github.com/I3r1h0n/Cocytus".into(),
        ))
        .style(|_theme, status| {
            let bg = match status {
                button::Status::Hovered => t::BG_ELEMENT_HOVER,
                button::Status::Pressed => t::BG_ELEMENT_PRESSED,
                _ => t::BG_ELEMENT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                border: Border {
                    radius: 6.0.into(),
                    color: t::BORDER,
                    width: 1.0,
                },
                text_color: t::ACCENT,
                ..Default::default()
            }
        })
        .padding([6, 16]);

        let issue_icon = svg(svg::Handle::from_memory(
            include_bytes!("../../../assets/issue.svg").as_slice(),
        ))
        .width(16)
        .height(16);

        let issues_btn = button(
            row![issue_icon, text("Report Issue").size(13).color(t::ACCENT),]
                .spacing(6)
                .align_y(iced::Alignment::Center),
        )
        .on_press(Message::OpenUrl(
            "https://github.com/I3r1h0n/Cocytus/issues".into(),
        ))
        .style(|_theme, status| {
            let bg = match status {
                button::Status::Hovered => t::BG_ELEMENT_HOVER,
                button::Status::Pressed => t::BG_ELEMENT_PRESSED,
                _ => t::BG_ELEMENT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                border: Border {
                    radius: 6.0.into(),
                    color: t::BORDER,
                    width: 1.0,
                },
                text_color: t::ACCENT,
                ..Default::default()
            }
        })
        .padding([6, 16]);

        let links = row![github_btn, issues_btn].spacing(12);

        let content = column![
            logo,
            name,
            version,
            author,
            Space::with_height(16),
            links,
            Space::with_height(12),
            container(w::action_button("Ok", Message::DismissAbout).width(120))
                .center_x(Length::Fill),
        ]
        .spacing(4)
        .align_x(iced::Alignment::Center);

        let dialog_card = container(content)
            .padding(32)
            .width(360)
            .style(|_theme| container::Style {
                background: Some(Background::Color(t::BG_SURFACE)),
                border: Border {
                    radius: 12.0.into(),
                    color: t::BORDER,
                    width: 1.0,
                },
                ..Default::default()
            });

        container(center(dialog_card))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.6))),
                ..Default::default()
            })
            .into()
    }

    fn pe_details_dialog(&self) -> Element<'_, Message> {
        let d = self.pe_details.as_ref().unwrap();

        let title = text(&d.file_name).size(16).color(t::ACCENT);

        let flags_str = if d.dll_characteristics.is_empty() {
            "None".to_string()
        } else {
            d.dll_characteristics.join(", ")
        };

        let detail_row = |label: &'static str, value: String| -> Element<'_, Message> {
            row![
                text(label).size(12).color(t::TEXT_MUTED).width(140),
                text(value).size(12).color(t::TEXT_SECONDARY),
            ]
            .spacing(8)
            .into()
        };

        let mut info_col = Column::new().spacing(6);
        info_col = info_col.push(title);
        info_col = info_col.push(Space::with_height(8));
        info_col = info_col.push(detail_row("File size", format_file_size(d.file_size)));
        info_col = info_col.push(detail_row("Architecture", d.machine.clone()));
        info_col = info_col.push(detail_row("Subsystem", d.subsystem.clone()));
        info_col = info_col.push(detail_row("Timestamp", d.timestamp.clone()));
        info_col = info_col.push(detail_row("Linker", d.linker_version.clone()));
        info_col = info_col.push(detail_row("Sections", d.sections.to_string()));
        info_col = info_col.push(detail_row("Image size", format!("{:#X}", d.image_size)));
        info_col = info_col.push(detail_row("Entry point", format!("{:#X}", d.entry_point)));
        info_col = info_col.push(detail_row("Checksum", format!("{:#X}", d.checksum)));
        info_col = info_col.push(detail_row("DLL characteristics", flags_str));

        info_col = info_col.push(Space::with_height(12));
        info_col = info_col.push(
            container(w::action_button("Ok", Message::DismissPeDetails).width(120))
                .center_x(Length::Fill),
        );

        let dialog_card = container(info_col)
            .padding(24)
            .max_width(520)
            .style(|_theme| container::Style {
                background: Some(Background::Color(t::BG_SURFACE)),
                border: Border {
                    radius: 12.0.into(),
                    color: t::BORDER,
                    width: 1.0,
                },
                ..Default::default()
            });

        container(center(dialog_card))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.6))),
                ..Default::default()
            })
            .into()
    }

    fn options_dialog_view(&self) -> Element<'_, Message> {
        let dlg = self.options_dialog.as_ref().unwrap();

        let title = text("Options").size(16).color(t::ACCENT);

        // --- PDB cache path ---
        let pdb_label = text("PDB cache path").size(12).color(t::TEXT_SECONDARY);

        let pdb_input = text_input("./pdb", &dlg.pdb_path)
            .on_input(Message::OptionsPdbPathChanged)
            .size(12)
            .padding([6, 8])
            .style(styles::search_input_style);

        let browse_btn = w::dark_button("Browse", Message::BrowseOptionsPdbPath);

        let pdb_row = row![pdb_input, browse_btn]
            .spacing(8)
            .align_y(iced::Alignment::Center);

        // --- Buttons ---
        let buttons = row![
            w::action_button("Save", Message::SaveOptions).width(100),
            w::dark_button("Cancel", Message::DismissOptions).width(100),
        ]
        .spacing(12);

        // --- Layout ---
        let content = column![
            title,
            Space::with_height(12),
            pdb_label,
            pdb_row,
            Space::with_height(16),
            container(buttons).center_x(Length::Fill),
        ]
        .spacing(6);

        let dialog_card = container(content)
            .padding(24)
            .width(440)
            .style(|_theme| container::Style {
                background: Some(Background::Color(t::BG_SURFACE)),
                border: Border {
                    radius: 12.0.into(),
                    color: t::BORDER,
                    width: 1.0,
                },
                ..Default::default()
            });

        container(center(dialog_card))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.6))),
                ..Default::default()
            })
            .into()
    }

    fn export_dialog_view(&self) -> Element<'_, Message> {
        let dlg = self.export_dialog.as_ref().unwrap();
        let enabled_count = dlg.enabled.iter().filter(|&&e| e).count();
        let total_count = dlg.symbols.len();

        // Title
        let title = text("Export Selected Symbols").size(16).color(t::ACCENT);
        let count_label =
            text(format!("{enabled_count}/{total_count} symbols")).size(12).color(t::TEXT_MUTED);

        // Symbol list with checkboxes
        let symbol_items: Vec<Element<'_, Message>> = dlg
            .symbols
            .iter()
            .zip(dlg.enabled.iter())
            .enumerate()
            .map(|(idx, ((name, kind), &checked))| {
                let kind_str = match kind {
                    SymbolKind::Struct => "struct",
                    SymbolKind::Function => "function",
                    SymbolKind::Enum => "enum",
                };
                let cb = checkbox("", checked)
                    .on_toggle(move |_| Message::ToggleExportSymbol(idx))
                    .size(14)
                    .spacing(0)
                    .style(|_theme, status| {
                        let (bg, border_color) = match status {
                            checkbox::Status::Active { is_checked }
                            | checkbox::Status::Hovered { is_checked } => {
                                if is_checked {
                                    (t::ACCENT_MUTED, t::ACCENT)
                                } else {
                                    (t::BG_ELEMENT, t::BORDER)
                                }
                            }
                            checkbox::Status::Disabled { .. } => (t::BG_ELEMENT, t::BORDER),
                        };
                        let icon_color = match status {
                            checkbox::Status::Hovered { .. } => t::TEXT_PRIMARY,
                            _ => t::ACCENT,
                        };
                        checkbox::Style {
                            background: Background::Color(bg),
                            icon_color,
                            border: Border {
                                radius: 3.0.into(),
                                color: border_color,
                                width: 1.0,
                            },
                            text_color: Some(t::TEXT_SECONDARY),
                        }
                    });
                let text_color = if checked { t::TEXT_SECONDARY } else { t::TEXT_MUTED };
                row![
                    cb,
                    text(name.as_str()).size(12).color(text_color),
                    Space::with_width(Length::Fill),
                    text(kind_str).size(11).color(t::TEXT_MUTED),
                ]
                .spacing(8)
                .padding([2, 12])
                .align_y(iced::Alignment::Center)
                .into()
            })
            .collect();

        let symbol_list = container(
            scrollable(
                Column::with_children(symbol_items)
                    .spacing(2)
                    .width(Length::Fill),
            )
            .height(180)
            .style(styles::scrollable_style),
        )
        .width(Length::Fill)
        .padding(4)
        .style(|_theme| container::Style {
            background: Some(Background::Color(t::BG_BASE)),
            border: Border {
                radius: 6.0.into(),
                color: t::BORDER,
                width: 1.0,
            },
            ..Default::default()
        });

        // Format picker
        let format_row = row![
            text("Format:").size(12).color(t::TEXT_MUTED).width(60),
            w::segmented_toggle(
                "JSON",
                "C/C++",
                dlg.format == ViewMode::Json,
                Message::SetExportDialogFormat(ViewMode::Json),
                Message::SetExportDialogFormat(ViewMode::Cpp),
            ),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        // Path picker
        let path_text = dlg
            .path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "No file selected".into());

        let path_display = container(
            text(path_text)
                .size(12)
                .color(if dlg.path.is_some() {
                    t::TEXT_SECONDARY
                } else {
                    t::TEXT_MUTED
                }),
        )
        .width(Length::Fill)
        .padding([4, 8])
        .style(|_theme| container::Style {
            background: Some(Background::Color(t::BG_ELEMENT)),
            border: Border {
                radius: 4.0.into(),
                color: t::BORDER,
                width: 1.0,
            },
            ..Default::default()
        });

        let path_row = row![
            text("Output:").size(12).color(t::TEXT_MUTED).width(60),
            path_display,
            w::dark_button("Browse", Message::BrowseExportPath),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        // Progress (visible only during export)
        let progress_section: Element<'_, Message> = if dlg.exporting {
            let pct = if total_count == 0 {
                1.0
            } else {
                dlg.progress as f32 / total_count as f32
            };
            column![
                progress_bar(0.0..=1.0, pct)
                    .height(6)
                    .style(|_theme| progress_bar::Style {
                        background: Background::Color(t::BG_ELEMENT),
                        bar: Background::Color(t::ACCENT),
                        border: Border {
                            radius: 3.0.into(),
                            color: Color::TRANSPARENT,
                            width: 0.0,
                        },
                    }),
                text(format!(
                    "Exporting: {} ({}/{})",
                    dlg.current_name, dlg.progress, total_count
                ))
                .size(11)
                .color(t::TEXT_MUTED),
            ]
            .spacing(4)
            .into()
        } else {
            Space::with_height(0).into()
        };

        // Export button
        let can_export = dlg.path.is_some() && !dlg.exporting && enabled_count > 0;
        let export_btn = if can_export {
            w::action_button("Export", Message::StartSymbolExport).width(120)
        } else {
            w::disabled_button("Export").width(120)
        };

        let content = column![
            title,
            Space::with_height(4),
            count_label,
            symbol_list,
            format_row,
            path_row,
            progress_section,
            Space::with_height(4),
            container(export_btn).center_x(Length::Fill),
        ]
        .spacing(12);

        let dialog_card = container(content)
            .padding(24)
            .max_width(520)
            .style(|_theme| container::Style {
                background: Some(Background::Color(t::BG_SURFACE)),
                border: Border {
                    radius: 12.0.into(),
                    color: t::BORDER,
                    width: 1.0,
                },
                ..Default::default()
            });

        container(center(dialog_card))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.6))),
                ..Default::default()
            })
            .into()
    }
}

fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

// Pane view functions (not in App to avoid borrow issues)

fn view_symbol_pane<'a>(
    pdb_data: &'a Option<super::super::types::PdbData>,
    symbol_filter: &'a str,
    selected_symbol_idx: &'a Option<usize>,
    symbol_checks: &'a std::collections::HashSet<usize>,
    loading_pdb: bool,
    pdb_status: &'a str,
    show_structs: bool,
    show_functions: bool,
    show_enums: bool,
    symbol_tab: SymbolTab,
    context_menu_symbol_idx: &'a Option<usize>,
) -> Element<'a, Message> {
    let selected_count = symbol_checks.len();

    // Tab bar
    let mut tab_row = row![
        w::tab_button("Symbols", symbol_tab == SymbolTab::Symbols, Message::SwitchSymbolTab(SymbolTab::Symbols)),
        w::tab_button(&format!("Selected ({selected_count})"), symbol_tab == SymbolTab::Selected, Message::SwitchSymbolTab(SymbolTab::Selected)),
        Space::with_width(Length::Fill),
    ]
    .spacing(2)
    .padding([4, 4])
    .align_y(iced::Alignment::Center);

    if symbol_tab == SymbolTab::Selected && selected_count > 0 {
        tab_row = tab_row.push(
            button(text("Clear").size(11).color(t::TEXT_MUTED).center())
                .on_press(Message::ClearSelected)
                .style(|_theme, status| {
                    let bg = match status {
                        button::Status::Hovered => t::BG_ELEMENT_HOVER,
                        button::Status::Pressed => t::BG_ELEMENT_PRESSED,
                        _ => iced::Color::TRANSPARENT,
                    };
                    button::Style {
                        background: Some(Background::Color(bg)),
                        border: Border { radius: 4.0.into(), color: iced::Color::TRANSPARENT, width: 0.0 },
                        text_color: t::TEXT_MUTED,
                        ..Default::default()
                    }
                })
                .padding([2, 8]),
        );
    }

    let tab_hdr = column![
        container(tab_row).width(Length::Fill),
        horizontal_rule(1).style(styles::divider_style),
    ];

    if loading_pdb {
        return column![
            tab_hdr,
            container(text(pdb_status).size(12).color(t::TEXT_MUTED))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        ]
        .height(Length::Fill)
        .into();
    }

    let Some(data) = pdb_data else {
        return column![
            tab_hdr,
            container(
                text("Right-click a PE file and select Load").size(12).color(t::TEXT_MUTED),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
        ]
        .height(Length::Fill)
        .into();
    };

    match symbol_tab {
        SymbolTab::Symbols => {
            let filter_lower = symbol_filter.to_ascii_lowercase();
            let filtered: Vec<(usize, &crate::gui::SymbolEntry)> = data
                .symbols
                .iter()
                .enumerate()
                .filter(|(_, entry)| {
                    let kind_ok = match entry.kind {
                        SymbolKind::Struct => show_structs,
                        SymbolKind::Function => show_functions,
                        SymbolKind::Enum => show_enums,
                    };
                    kind_ok
                        && (filter_lower.is_empty()
                            || entry.name.to_ascii_lowercase().contains(&filter_lower))
                })
                .collect();

            let total = filtered.len();
            let capped = total > MAX_VISIBLE_SYMBOLS;
            let count_text = if capped {
                format!("{total}, showing {MAX_VISIBLE_SYMBOLS}")
            } else {
                format!("{total}")
            };

            let filter_hdr = w::pane_header(
                row![
                    text(count_text).size(11).color(t::TEXT_MUTED),
                    Space::with_width(Length::Fill),
                    checkbox("Structures", show_structs)
                        .on_toggle(Message::ToggleShowStructs)
                        .size(14).text_size(11).spacing(4)
                        .style(styles::filter_checkbox_style),
                    checkbox("Functions", show_functions)
                        .on_toggle(Message::ToggleShowFunctions)
                        .size(14).text_size(11).spacing(4)
                        .style(styles::filter_checkbox_style),
                    checkbox("Enums", show_enums)
                        .on_toggle(Message::ToggleShowEnums)
                        .size(14).text_size(11).spacing(4)
                        .style(styles::filter_checkbox_style),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            );

            let list: Element<'_, Message> = if filtered.is_empty() {
                text("No matching symbols").size(12).color(t::TEXT_MUTED).into()
            } else {
                let items: Vec<Element<'_, Message>> = filtered
                    .iter()
                    .take(MAX_VISIBLE_SYMBOLS)
                    .map(|(orig_idx, entry)| {
                        let is_checked = symbol_checks.contains(orig_idx);
                        let is_selected = *selected_symbol_idx == Some(*orig_idx);
                        let is_context = *context_menu_symbol_idx == Some(*orig_idx);
                        w::symbol_item(&entry.name, is_checked, is_selected || is_context, *orig_idx)
                    })
                    .collect();

                scrollable(Column::with_children(items).spacing(1).width(Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(styles::scrollable_style)
                    .into()
            };

            let search = text_input("Search symbols...", symbol_filter)
                .on_input(Message::SymbolFilterChanged)
                .size(12)
                .padding([4, 8])
                .style(styles::search_input_style);

            column![tab_hdr, filter_hdr, list, search]
                .spacing(4)
                .height(Length::Fill)
                .padding(0)
                .into()
        }
        SymbolTab::Selected => {
            let checked: Vec<(usize, &crate::gui::SymbolEntry)> = data
                .symbols
                .iter()
                .enumerate()
                .filter(|(idx, _)| symbol_checks.contains(idx))
                .collect();

            let list: Element<'_, Message> = if checked.is_empty() {
                container(
                    text("No symbols selected.\nUse checkboxes in the Symbols tab.")
                        .size(12)
                        .color(t::TEXT_MUTED),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
            } else {
                let items: Vec<Element<'_, Message>> = checked
                    .iter()
                    .map(|(orig_idx, entry)| {
                        let is_selected = *selected_symbol_idx == Some(*orig_idx);
                        let is_context = *context_menu_symbol_idx == Some(*orig_idx);
                        w::symbol_item(&entry.name, true, is_selected || is_context, *orig_idx)
                    })
                    .collect();

                scrollable(Column::with_children(items).spacing(1).width(Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(styles::scrollable_style)
                    .into()
            };

            column![tab_hdr, list]
                .spacing(4)
                .height(Length::Fill)
                .padding(0)
                .into()
        }
    }
}

fn view_file_pane<'a>(
    pe_files: &'a [String],
    pe_filter: &'a str,
    selected_file: &'a Option<String>,
    context_menu_file: &'a Option<String>,
) -> Element<'a, Message> {
    let filter_lower = pe_filter.to_ascii_lowercase();
    let filtered: Vec<&str> = pe_files
        .iter()
        .filter_map(|path| {
            let name = path.rsplit(['\\', '/']).next().unwrap_or(path);
            if filter_lower.is_empty() || name.to_ascii_lowercase().contains(&filter_lower) {
                Some(name)
            } else {
                None
            }
        })
        .collect();

    let total = filtered.len();
    let capped = total > MAX_VISIBLE_FILES;
    let header_text = if capped {
        format!("PE Files ({total}, showing {MAX_VISIBLE_FILES})")
    } else {
        format!("PE Files ({total})")
    };
    let pane_hdr = w::pane_header(
        text(header_text).size(12).font(w::bold_font()).color(t::TEXT_SECONDARY),
    );

    let list: Element<'_, Message> = if filtered.is_empty() {
        text("No matching files").size(12).color(t::TEXT_MUTED).into()
    } else {
        let items: Vec<Element<'_, Message>> = filtered
            .iter()
            .take(MAX_VISIBLE_FILES)
            .map(|name| {
                let is_selected = selected_file.as_deref() == Some(*name);
                let is_context = context_menu_file.as_deref() == Some(*name);
                w::file_item(&name.to_string(), is_selected || is_context)
            })
            .collect();

        scrollable(Column::with_children(items).spacing(1).width(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(styles::scrollable_style)
            .into()
    };

    let search = text_input("Search files...", pe_filter)
        .on_input(Message::PeFilterChanged)
        .size(12)
        .padding([4, 8])
        .style(styles::search_input_style);

    column![pane_hdr, list, search]
        .spacing(4)
        .height(Length::Fill)
        .padding(0)
        .into()
}

fn view_json_pane<'a>(
    pdb_data: &'a Option<super::super::types::PdbData>,
    selected_symbol_idx: &'a Option<usize>,
    json_content: &'a text_editor::Content,
    has_json: bool,
    loading_json: bool,
    json_tabs: &'a [usize],
    view_mode: ViewMode,
) -> Element<'a, Message> {
    let Some(data) = pdb_data else {
        return text("").size(14).color(t::TEXT_MUTED).into();
    };

    // Tab bar or static header
    let tab_bar: Element<'_, Message> = if json_tabs.is_empty() {
        w::pane_header(
            text("Select a symbol").size(12).font(w::bold_font()).color(t::TEXT_SECONDARY),
        )
        .into()
    } else {
        let tabs: Vec<Element<'_, Message>> = json_tabs
            .iter()
            .map(|&idx| {
                let name = data.symbols.get(idx).map(|e| e.name.as_str()).unwrap_or("?");
                let is_active = *selected_symbol_idx == Some(idx);
                w::json_tab(name, is_active, idx)
            })
            .collect();

        let tab_row = scrollable(
            Row::with_children(tabs).spacing(2).padding([2, 4]),
        )
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::new().width(4).scroller_width(4),
        ))
        .style(styles::tab_scrollable_style)
        .width(Length::Fill);

        column![tab_row, horizontal_rule(1).style(styles::divider_style)].into()
    };

    let content: Element<'_, Message> = if loading_json {
        container(text("Loading...").size(12).color(t::TEXT_MUTED))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    } else if !has_json {
        text("Click a symbol to view").size(12).color(t::TEXT_MUTED).into()
    } else {
        let editor: Element<'_, Message> = if view_mode == ViewMode::Cpp {
            text_editor(json_content)
                .on_action(Message::JsonEditorAction)
                .highlight_with::<CppHighlighter>((), cpp_highlight::format)
                .font(iced::Font::MONOSPACE)
                .size(12)
                .line_height(iced::widget::text::LineHeight::Relative(1.6))
                .height(Length::Shrink)
                .padding([4, 8])
                .style(styles::json_editor_style)
                .into()
        } else {
            text_editor(json_content)
                .on_action(Message::JsonEditorAction)
                .highlight_with::<JsonHighlighter>((), json_highlight::format)
                .font(iced::Font::MONOSPACE)
                .size(12)
                .line_height(iced::widget::text::LineHeight::Relative(1.6))
                .height(Length::Shrink)
                .padding([4, 8])
                .style(styles::json_editor_style)
                .into()
        };

        scrollable(
            mouse_area(editor).on_right_press(Message::ShowJsonContextMenu),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(styles::scrollable_style)
        .into()
    };

    column![tab_bar, content].spacing(4).height(Length::Fill).into()
}
