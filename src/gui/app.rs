use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use iced::widget::{pane_grid, text_editor};
use iced::{Point, Task, Theme};

use crate::extractors::iso::{self, MountInfo};
use crate::extractors::wim::WimImage;
use crate::utils::config::AppConfig;

use super::types::{ExportDialog, Message, OptionsDialog, PdbData, SymbolTab, ViewMode};
use super::APP_NAME;
use super::APP_VERSION;

// Layout enums

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Screen {
    Setup,
    Main,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PaneVariant {
    LeftTop,
    LeftBottom,
    Right,
}

// App state
pub struct App {
    pub(super) screen: Screen,
    pub(super) iso_path: Option<PathBuf>,
    pub(super) mount_info: Option<MountInfo>,
    pub(super) wim_images: Vec<WimImage>,
    pub(super) selected_image: Option<WimImage>,
    pub(super) status: String,
    pub(super) pe_files: Vec<String>,
    pub(super) pe_filter: String,
    pub(super) panes: pane_grid::State<PaneVariant>,
    pub(super) selected_file: Option<String>,
    pub(super) context_menu_file: Option<String>,
    pub(super) context_menu_pos: Point,
    pub(super) cursor_position: Point,
    pub(super) file_menu_open: bool,
    pub(super) pdb_data: Option<PdbData>,
    pub(super) selected_symbol_idx: Option<usize>,
    pub(super) json_content: text_editor::Content,
    pub(super) json_context_menu: bool,
    pub(super) json_context_menu_pos: Point,
    pub(super) symbol_filter: String,
    pub(super) symbol_checks: HashSet<usize>,
    pub(super) loading_pdb: bool,
    pub(super) pdb_status: String,
    pub(super) show_structs: bool,
    pub(super) show_functions: bool,
    pub(super) show_enums: bool,
    pub(super) symbol_tab: SymbolTab,
    pub(super) view_mode: ViewMode,
    pub(super) loading_json: bool,
    pub(super) json_cache: HashMap<usize, text_editor::Content>,
    pub(super) json_tabs: Vec<usize>,
    pub(super) context_menu_symbol_idx: Option<usize>,
    pub(super) show_iso_info: bool,
    pub(super) iso_file_size: Option<u64>,
    pub(super) export_dialog: Option<ExportDialog>,
    pub(super) options_dialog: Option<OptionsDialog>,
    pub(super) pe_details: Option<crate::utils::pe_info::PeDetails>,
    pub(super) show_about: bool,
    pub(super) config: AppConfig,
}

impl App {
    pub(crate) fn new() -> (Self, Task<Message>) {
        let config = pane_grid::Configuration::Split {
            axis: pane_grid::Axis::Vertical,
            ratio: 0.4,
            a: Box::new(pane_grid::Configuration::Split {
                axis: pane_grid::Axis::Horizontal,
                ratio: 0.5,
                a: Box::new(pane_grid::Configuration::Pane(PaneVariant::LeftTop)),
                b: Box::new(pane_grid::Configuration::Pane(PaneVariant::LeftBottom)),
            }),
            b: Box::new(pane_grid::Configuration::Pane(PaneVariant::Right)),
        };
        (
            Self {
                screen: Screen::Setup,
                iso_path: None,
                mount_info: None,
                wim_images: Vec::new(),
                selected_image: None,
                status: String::new(),
                pe_files: Vec::new(),
                pe_filter: String::new(),
                panes: pane_grid::State::with_configuration(config),
                selected_file: None,
                context_menu_file: None,
                context_menu_pos: Point::ORIGIN,
                cursor_position: Point::ORIGIN,
                file_menu_open: false,
                pdb_data: None,
                selected_symbol_idx: None,
                json_content: text_editor::Content::with_text(""),
                json_context_menu: false,
                json_context_menu_pos: Point::ORIGIN,
                symbol_filter: String::new(),
                symbol_checks: HashSet::new(),
                loading_pdb: false,
                pdb_status: String::new(),
                show_structs: true,
                show_functions: true,
                show_enums: true,
                symbol_tab: SymbolTab::Symbols,
                view_mode: ViewMode::Json,
                loading_json: false,
                json_cache: HashMap::new(),
                json_tabs: Vec::new(),
                context_menu_symbol_idx: None,
                show_iso_info: false,
                iso_file_size: None,
                export_dialog: None,
                options_dialog: None,
                pe_details: None,
                show_about: false,
                config: crate::utils::config::load(),
            },
            Task::none(),
        )
    }

    pub(crate) fn title(&self) -> String {
        format!("{APP_NAME} v{APP_VERSION}")
    }

    pub(crate) fn theme(&self) -> Theme {
        Theme::Dark
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if let Some(info) = self.mount_info.take() {
            iso::unmount(&info.iso_path);
        }
    }
}
