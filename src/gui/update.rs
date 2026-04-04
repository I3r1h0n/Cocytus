use iced::widget::text_editor;
use iced::Task;

use crate::extractors::iso;
use crate::extractors::pdb::PdbExtractor;
use crate::extractors::wim;
use crate::utils::pdb_loader;

use super::app::{App, Screen};
use super::types::*;
use super::SYSTEM32_PATH;

impl App {
    /// Message dispatcher
    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Setup
            Message::BrowseIso => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Select Windows ISO")
                            .add_filter("ISO files", &["iso"])
                            .pick_file()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    Message::IsoPicked,
                );
            }
            Message::IsoPicked(Some(path)) => {
                if let Some(info) = self.mount_info.take() {
                    iso::unmount(&info.iso_path);
                }
                self.pdb_path = None;
                self.status = "Mounting ISO...".into();
                self.wim_images.clear();
                self.selected_image = None;
                self.iso_path = Some(path.clone());
                return Task::perform(
                    async move {
                        let info = iso::mount(&path).map_err(|e| e.to_string())?;
                        let images = wim::list_images(&info.wim_path).unwrap_or_default();
                        Ok((info, images))
                    },
                    Message::IsoMounted,
                );
            }
            Message::IsoPicked(None) => {}
            Message::IsoMounted(result) => match result {
                Ok((info, images)) => {
                    self.iso_file_size = self
                        .iso_path
                        .as_ref()
                        .and_then(|p| std::fs::metadata(p).ok())
                        .map(|m| m.len());
                    self.mount_info = Some(info);
                    if images.is_empty() {
                        self.status = "No images found in WIM".into();
                    } else {
                        self.status = format!("Found {} image(s)", images.len());
                        if images.len() == 1 {
                            self.selected_image = Some(images[0].clone());
                        }
                        self.wim_images = images;
                    }
                }
                Err(e) => {
                    self.status = format!("Mount failed: {e}");
                }
            },
            Message::ImageSelected(image) => {
                self.selected_image = Some(image);
            }
            // Direct PDB open
            Message::BrowsePdb => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Select PDB File")
                            .add_filter("PDB files", &["pdb"])
                            .pick_file()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    Message::PdbFilePicked,
                );
            }
            Message::PdbFilePicked(Some(path)) => {
                self.iso_path = None;
                self.wim_images.clear();
                self.selected_image = None;
                if let Some(info) = self.mount_info.take() {
                    iso::unmount(&info.iso_path);
                }
                self.status.clear();
                self.pdb_path = Some(path);
            }
            Message::PdbFilePicked(None) => {}
            Message::DirectPdbLoaded(result) => {
                self.loading_pdb = false;
                match result {
                    Ok(data) => {
                        self.pdb_status = format!(
                            "{} — {} symbols loaded",
                            data.file_name,
                            data.symbols.len()
                        );
                        self.pdb_data = Some(data);
                        self.screen = Screen::Main;
                        self.show_iso_info = false;
                        return iced::window::get_oldest().then(|id| {
                            id.map_or(Task::none(), |id| {
                                iced::window::resize(id, iced::Size::new(1500.0, 900.0))
                            })
                        });
                    }
                    Err(e) => {
                        self.status = format!("PDB error: {e}");
                    }
                }
            }

            Message::Continue => {
                // Direct PDB path
                if let Some(path) = self.pdb_path.take() {
                    return self.start_direct_pdb_load(path);
                }
                // ISO + WIM path
                if let (Some(info), Some(image)) =
                    (self.mount_info.clone(), self.selected_image.clone())
                {
                    self.status = "Loading PE files...".into();
                    let idx = image.index;
                    let wim_path = info.wim_path;
                    return Task::perform(
                        async move {
                            wim::list_dir_recursive(&wim_path, idx, SYSTEM32_PATH)
                                .unwrap_or_default()
                        },
                        Message::PeFilesLoaded,
                    );
                }
            }
            Message::PeFilesLoaded(files) => {
                self.pe_files = files;
                self.screen = Screen::Main;
                self.show_iso_info = true;
                return iced::window::get_oldest().then(|id| {
                    id.map_or(Task::none(), |id| {
                        iced::window::resize(id, iced::Size::new(1500.0, 900.0))
                    })
                });
            }

            // Layout / menus
            Message::PaneResized(e) => self.panes.resize(e.split, e.ratio),
            Message::ToggleFileMenu => self.file_menu_open = !self.file_menu_open,
            Message::FileMenu(action) => {
                self.file_menu_open = false;
                match action {
                    FileMenuAction::OpenNewIso | FileMenuAction::OpenPdb => {
                        if let Some(info) = self.mount_info.take() {
                            iso::unmount(&info.iso_path);
                        }
                        self.screen = Screen::Setup;
                        self.pe_files.clear();
                        self.pe_filter.clear();
                        self.iso_path = None;
                        self.pdb_path = None;
                        self.wim_images.clear();
                        self.selected_image = None;
                        self.selected_file = None;
                        self.context_menu_file = None;
                        self.status.clear();
                        self.pdb_data = None;
                        self.selected_symbol_idx = None;
                        self.json_content = text_editor::Content::with_text("");
                        self.json_context_menu = false;
                        self.symbol_filter.clear();
                        self.symbol_checks.clear();
                        self.json_cache.clear();
                        self.json_tabs.clear();
                        self.context_menu_symbol_idx = None;
                        self.loading_pdb = false;
                        self.pdb_status.clear();
                        self.symbol_tab = SymbolTab::Symbols;
                        self.view_mode = ViewMode::Json;
                        self.export_dialog = None;
                        self.show_iso_info = false;
                        self.iso_file_size = None;
                        let resize = iced::window::get_oldest().then(|id| {
                            id.map_or(Task::none(), |id| {
                                iced::window::resize(id, iced::Size::new(600.0, 680.0))
                            })
                        });
                        // If "Open PDB" was chosen, immediately open the file dialog
                        if matches!(action, FileMenuAction::OpenPdb) {
                            return Task::batch([
                                resize,
                                Task::done(Message::BrowsePdb),
                            ]);
                        }
                        return resize;
                    }
                    FileMenuAction::Quit => {
                        if let Some(info) = self.mount_info.take() {
                            iso::unmount(&info.iso_path);
                        }
                        return iced::window::get_oldest().then(|id| {
                            id.map_or(Task::none(), |id| iced::window::close(id))
                        });
                    }
                }
            }

            // PE file lists
            Message::PeFilterChanged(f) => {
                self.pe_filter = f;
                self.context_menu_file = None;
            }
            Message::SelectFile(name) => {
                self.selected_file = Some(name);
                self.context_menu_file = None;
            }
            Message::ShowContextMenu(name) => {
                self.selected_file = Some(name.clone());
                self.context_menu_file = Some(name);
                self.context_menu_pos = self.cursor_position;
                self.context_menu_symbol_idx = None;
            }
            Message::ContextAction(action) => {
                self.context_menu_file = None;
                match action {
                    ContextMenuAction::Load => return self.start_pdb_load(),
                    ContextMenuAction::ExportFile => return self.start_export_file(),
                    ContextMenuAction::ExportPdb => return self.start_export_pdb(),
                    ContextMenuAction::Details => return self.start_pe_details(),
                }
            }
            Message::DismissContextMenu => self.context_menu_file = None,

            // PDB loading
            Message::PdbLoaded(result) => {
                self.loading_pdb = false;
                match result {
                    Ok(data) => {
                        self.pdb_status = format!(
                            "{} — {} symbols loaded",
                            data.file_name,
                            data.symbols.len()
                        );
                        self.pdb_data = Some(data);
                    }
                    Err(e) => {
                        self.pdb_status = format!("Error: {e}");
                    }
                }
            }

            // Symbol list
            Message::SelectSymbol(idx) => {
                self.context_menu_symbol_idx = None;
                if self.selected_symbol_idx == Some(idx) {
                    return Task::none();
                }

                // If symbol already has a tab, switch to it
                if self.json_tabs.contains(&idx) {
                    self.stash_active_tab();
                    self.selected_symbol_idx = Some(idx);
                    return self.load_symbol_json(idx);
                }

                // Replace current active tab, or create first tab
                self.stash_active_tab();
                if let Some(old_idx) = self.selected_symbol_idx {
                    if let Some(pos) = self.json_tabs.iter().position(|&t| t == old_idx) {
                        self.json_tabs[pos] = idx;
                        if !self.symbol_checks.contains(&old_idx) {
                            self.json_cache.remove(&old_idx);
                        }
                    } else {
                        self.json_tabs.push(idx);
                    }
                } else {
                    self.json_tabs.push(idx);
                }

                self.selected_symbol_idx = Some(idx);
                return self.load_symbol_json(idx);
            }
            Message::OpenSymbolNewTab(idx) => {
                self.context_menu_symbol_idx = None;

                // If already has a tab, just switch
                if self.json_tabs.contains(&idx) {
                    if self.selected_symbol_idx == Some(idx) {
                        return Task::none();
                    }
                    self.stash_active_tab();
                    self.selected_symbol_idx = Some(idx);
                    return self.load_symbol_json(idx);
                }

                // Add new tab
                self.stash_active_tab();
                self.json_tabs.push(idx);
                self.selected_symbol_idx = Some(idx);
                return self.load_symbol_json(idx);
            }
            Message::CloseJsonTab(idx) => {
                if let Some(pos) = self.json_tabs.iter().position(|&t| t == idx) {
                    self.json_tabs.remove(pos);
                    if !self.symbol_checks.contains(&idx) {
                        self.json_cache.remove(&idx);
                    }
                    if self.selected_symbol_idx == Some(idx) {
                        if self.json_tabs.is_empty() {
                            self.selected_symbol_idx = None;
                            self.json_content = text_editor::Content::with_text("");
                            self.loading_json = false;
                        } else {
                            let new_pos = pos.min(self.json_tabs.len() - 1);
                            let new_idx = self.json_tabs[new_pos];
                            self.selected_symbol_idx = Some(new_idx);
                            return self.load_symbol_json(new_idx);
                        }
                    }
                }
            }
            Message::SwitchJsonTab(idx) => {
                if self.selected_symbol_idx == Some(idx) || !self.json_tabs.contains(&idx) {
                    return Task::none();
                }
                self.stash_active_tab();
                self.selected_symbol_idx = Some(idx);
                return self.load_symbol_json(idx);
            }
            Message::SymbolJsonReady(idx, json) => {
                self.loading_json = false;
                let content = text_editor::Content::with_text(&json);
                if self.selected_symbol_idx == Some(idx) {
                    self.json_content = content;
                } else if self.symbol_checks.contains(&idx) || self.json_tabs.contains(&idx) {
                    self.json_cache.insert(idx, content);
                }
            }
            Message::ToggleSymbolCheck(idx) => {
                if self.symbol_checks.remove(&idx) {
                    if !self.json_tabs.contains(&idx) {
                        self.json_cache.remove(&idx);
                    }
                } else {
                    self.symbol_checks.insert(idx);
                }
            }
            Message::ShowSymbolContextMenu(idx) => {
                self.context_menu_symbol_idx = Some(idx);
                self.context_menu_pos = self.cursor_position;
                self.context_menu_file = None;
                self.json_context_menu = false;
            }
            Message::DismissSymbolContextMenu => {
                self.context_menu_symbol_idx = None;
            }
            Message::SymbolContextToggleCheck(idx) => {
                self.context_menu_symbol_idx = None;
                if self.symbol_checks.remove(&idx) {
                    if !self.json_tabs.contains(&idx) {
                        self.json_cache.remove(&idx);
                    }
                } else {
                    self.symbol_checks.insert(idx);
                }
            }
            Message::SymbolFilterChanged(f) => {
                self.symbol_filter = f;
            }
            Message::ToggleShowStructs(v) => self.show_structs = v,
            Message::ToggleShowFunctions(v) => self.show_functions = v,
            Message::ToggleShowEnums(v) => self.show_enums = v,

            // JSON editor
            Message::JsonEditorAction(action) => {
                if !action.is_edit() {
                    self.json_content.perform(action);
                }
            }
            Message::ShowJsonContextMenu => {
                self.json_context_menu = true;
                self.json_context_menu_pos = self.cursor_position;
                self.context_menu_symbol_idx = None;
            }
            Message::JsonContextCopy => {
                self.json_context_menu = false;
                let text = self
                    .json_content
                    .selection()
                    .unwrap_or_else(|| self.json_content.text());
                return iced::clipboard::write(text);
            }
            Message::JsonContextSearch => {
                self.json_context_menu = false;
                if let Some(selected) = self.json_content.selection() {
                    let trimmed = selected.trim().to_string();
                    if !trimmed.is_empty() {
                        self.symbol_filter = trimmed;
                    }
                }
            }
            Message::DismissJsonContextMenu => {
                self.json_context_menu = false;
            }

            // Tabs
            Message::SwitchSymbolTab(tab) => self.symbol_tab = tab,
            Message::ClearSelected => {
                self.symbol_checks.clear();
                self.json_cache.retain(|idx, _| self.json_tabs.contains(idx));
            }

            // Export results
            Message::ExportComplete(result) => {
                match result {
                    Ok(msg) => self.pdb_status = msg,
                    Err(e) => self.pdb_status = format!("Export error: {e}"),
                }
            }

            // ISO info dialog
            Message::DismissIsoInfo => {
                self.show_iso_info = false;
            }

            // Export dialog
            Message::ShowExportDialog => {
                let Some(ref data) = self.pdb_data else {
                    return Task::none();
                };
                let mut symbols: Vec<(String, SymbolKind)> = self
                    .symbol_checks
                    .iter()
                    .filter_map(|&idx| data.symbols.get(idx))
                    .map(|e| (e.name.clone(), e.kind.clone()))
                    .collect();
                if symbols.is_empty() {
                    self.pdb_status = "No symbols selected for export".into();
                    return Task::none();
                }
                symbols.sort_by(|a, b| a.0.cmp(&b.0));
                let enabled = vec![true; symbols.len()];
                self.export_dialog = Some(ExportDialog {
                    format: self.view_mode,
                    path: None,
                    symbols,
                    enabled,
                    exporting: false,
                    progress: 0,
                    current_name: String::new(),
                    buffer: String::new(),
                });
            }
            Message::DismissExportDialog => {
                if self.export_dialog.as_ref().is_some_and(|d| !d.exporting) {
                    self.export_dialog = None;
                }
            }
            Message::SetExportDialogFormat(mode) => {
                if let Some(ref mut dlg) = self.export_dialog {
                    if !dlg.exporting {
                        dlg.format = mode;
                    }
                }
            }
            Message::BrowseExportPath => {
                let Some(ref dlg) = self.export_dialog else {
                    return Task::none();
                };
                if dlg.exporting {
                    return Task::none();
                }
                let (ext, filter_name) = match dlg.format {
                    ViewMode::Json => ("json", "JSON files"),
                    ViewMode::Cpp => ("h", "C/C++ header files"),
                };
                let ext = ext.to_string();
                let filter_name = filter_name.to_string();
                return Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .set_title("Export Symbols")
                            .add_filter(&filter_name, &[ext.as_str()])
                            .save_file()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    Message::ExportPathPicked,
                );
            }
            Message::ExportPathPicked(path) => {
                if let (Some(p), Some(dlg)) = (path, &mut self.export_dialog) {
                    dlg.path = Some(p);
                }
            }
            Message::ToggleExportSymbol(idx) => {
                if let Some(ref mut dlg) = self.export_dialog {
                    if !dlg.exporting {
                        if let Some(val) = dlg.enabled.get_mut(idx) {
                            *val = !*val;
                        }
                    }
                }
            }
            Message::StartSymbolExport => {
                if let Some(ref mut dlg) = self.export_dialog {
                    if dlg.path.is_none() || dlg.exporting {
                        return Task::none();
                    }
                    dlg.exporting = true;
                    dlg.progress = 0;
                    dlg.buffer.clear();
                    dlg.current_name.clear();
                    return Task::perform(async {}, |()| Message::ExportTick);
                }
            }
            Message::ExportTick => {
                let extractor = match self.pdb_data {
                    Some(ref data) => data.extractor(),
                    None => return Task::none(),
                };
                let Some(ref mut dlg) = self.export_dialog else {
                    return Task::none();
                };
                if dlg.progress < dlg.symbols.len() {
                    // Skip unchecked symbols
                    if !dlg.enabled[dlg.progress] {
                        dlg.progress += 1;
                        return Task::perform(async {}, |()| Message::ExportTick);
                    }
                    let (name, kind) = dlg.symbols[dlg.progress].clone();
                    dlg.current_name = name.clone();
                    let text = match (dlg.format, kind) {
                        (ViewMode::Json, SymbolKind::Struct) => extractor.get_struct(&name),
                        (ViewMode::Json, SymbolKind::Function) => extractor.get_function(&name),
                        (ViewMode::Json, SymbolKind::Enum) => extractor.get_enum(&name),
                        (ViewMode::Cpp, SymbolKind::Struct) => extractor.get_struct_cpp(&name),
                        (ViewMode::Cpp, SymbolKind::Function) => extractor.get_function_cpp(&name),
                        (ViewMode::Cpp, SymbolKind::Enum) => extractor.get_enum_cpp(&name),
                    };
                    if let Some(t) = text {
                        if !dlg.buffer.is_empty() {
                            dlg.buffer.push('\n');
                        }
                        dlg.buffer.push_str(&t);
                    }
                    dlg.progress += 1;
                    return Task::perform(async {}, |()| Message::ExportTick);
                } else {
                    let buffer = std::mem::take(&mut dlg.buffer);
                    let path = dlg.path.clone().unwrap();
                    let count = dlg.enabled.iter().filter(|&&e| e).count();
                    return Task::perform(
                        async move {
                            std::fs::write(&path, buffer.as_bytes())
                                .map(|()| format!("Exported {count} symbols"))
                                .map_err(|e| e.to_string())
                        },
                        Message::ExportWriteComplete,
                    );
                }
            }
            Message::ExportWriteComplete(result) => {
                self.export_dialog = None;
                match result {
                    Ok(msg) => self.pdb_status = msg,
                    Err(e) => self.pdb_status = format!("Export error: {e}"),
                }
            }

            // View mode
            Message::SetViewMode(mode) => {
                if self.view_mode != mode {
                    self.view_mode = mode;
                    self.json_cache.clear();
                    self.json_content = text_editor::Content::with_text("");
                    if let Some(idx) = self.selected_symbol_idx {
                        return self.load_symbol_json(idx);
                    }
                }
            }

            // PE details dialog
            Message::PeDetailsLoaded(result) => {
                match result {
                    Ok(details) => self.pe_details = Some(details),
                    Err(e) => self.pdb_status = format!("Details error: {e}"),
                }
            }
            Message::DismissPeDetails => {
                self.pe_details = None;
            }

            // Options dialog
            Message::ShowOptions => {
                self.options_dialog = Some(OptionsDialog {
                    pdb_path: self.config.pdb_path.clone(),
                });
            }
            Message::DismissOptions => {
                self.options_dialog = None;
            }
            Message::OptionsPdbPathChanged(val) => {
                if let Some(ref mut dlg) = self.options_dialog {
                    dlg.pdb_path = val;
                }
            }
            Message::BrowseOptionsPdbPath => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Select PDB cache folder")
                            .pick_folder()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    Message::OptionsPdbPathPicked,
                );
            }
            Message::OptionsPdbPathPicked(path) => {
                if let (Some(p), Some(dlg)) = (path, &mut self.options_dialog) {
                    dlg.pdb_path = p.to_string_lossy().to_string();
                }
            }
            Message::SaveOptions => {
                if let Some(dlg) = self.options_dialog.take() {
                    self.config.pdb_path = dlg.pdb_path;
                    if let Err(e) = crate::utils::config::save(&self.config) {
                        self.pdb_status = format!("Failed to save config: {e}");
                    } else {
                        self.pdb_status = "Settings saved".into();
                    }
                }
            }

            // About dialog
            Message::ShowAbout => {
                self.show_about = true;
            }
            Message::DismissAbout => {
                self.show_about = false;
            }
            Message::OpenUrl(url) => {
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "start", "", &url])
                    .spawn();
                
            }

            // Misc
            Message::CursorMoved(pos) => self.cursor_position = pos,
            Message::Noop => {}
        }
        Task::none()
    }

    /// Stash the active tabs content into the cache before switching
    fn stash_active_tab(&mut self) {
        if let Some(old_idx) = self.selected_symbol_idx {
            if !self.loading_json {
                let content = std::mem::replace(
                    &mut self.json_content,
                    text_editor::Content::with_text(""),
                );
                self.json_cache.insert(old_idx, content);
            }
        }
    }

    /// Restore a tabs JSON from cache or load it
    fn load_symbol_json(&mut self, idx: usize) -> Task<Message> {
        if let Some(cached) = self.json_cache.remove(&idx) {
            self.json_content = cached;
            self.loading_json = false;
            return Task::none();
        }

        self.json_content = text_editor::Content::with_text("");
        self.loading_json = true;
        if let Some(ref data) = self.pdb_data {
            if let Some(entry) = data.symbols.get(idx) {
                let extractor = data.extractor();
                let name = entry.name.clone();
                let kind = entry.kind.clone();
                let view_mode = self.view_mode;
                return Task::perform(
                    async move {
                        let text = match view_mode {
                            ViewMode::Json => match kind {
                                SymbolKind::Struct => extractor.get_struct(&name),
                                SymbolKind::Function => extractor.get_function(&name),
                                SymbolKind::Enum => extractor.get_enum(&name),
                            },
                            ViewMode::Cpp => match kind {
                                SymbolKind::Struct => extractor.get_struct_cpp(&name),
                                SymbolKind::Function => extractor.get_function_cpp(&name),
                                SymbolKind::Enum => extractor.get_enum_cpp(&name),
                            },
                        };
                        (idx, text.unwrap_or_else(|| "No data".to_string()))
                    },
                    |(idx, text)| Message::SymbolJsonReady(idx, text),
                );
            }
        }
        self.loading_json = false;
        Task::none()
    }

    /// Kick off async PDB download + parse
    /// 
    /// No JSON serialization at load time - symbols are serialized lazily when selected
    fn start_pdb_load(&mut self) -> Task<Message> {
        let (Some(info), Some(image), Some(file_name)) =
            (self.mount_info.clone(), self.selected_image.clone(), &self.selected_file)
        else {
            return Task::none();
        };

        let Some(wim_inner_path) = self.find_full_pe_path(file_name) else {
            return Task::none();
        };

        self.loading_pdb = true;
        self.pdb_status = format!("Loading PDB for {}...", file_name);
        self.pdb_data = None;
        self.selected_symbol_idx = None;
        self.json_content = text_editor::Content::with_text("");
        self.json_context_menu = false;
        self.symbol_filter.clear();
        self.symbol_checks.clear();
        self.json_cache.clear();
        self.json_tabs.clear();
        self.context_menu_symbol_idx = None;
        self.symbol_tab = SymbolTab::Symbols;

        let image_idx = image.index;
        let file_name_owned = file_name.clone();
        let wim_path = info.wim_path;
        let pdb_cache_dir = self.config.pdb_dir();

        Task::perform(
            async move {
                let temp_dir = std::env::temp_dir().join("cocytus_pdb");
                std::fs::create_dir_all(&temp_dir).map_err(|e| format!("Temp dir: {e}"))?;

                wim::extract_file(&wim_path, &temp_dir, image_idx, &wim_inner_path)
                    .map_err(|e| format!("Extract PE: {e}"))?;

                let pe_filename = wim_inner_path
                    .rsplit(['\\', '/'])
                    .next()
                    .unwrap_or(&wim_inner_path);
                let pe_path = temp_dir.join(pe_filename);

                let pdb_info =
                    pdb_loader::parse_pdb_info(&pe_path).map_err(|e| format!("PDB info: {e}"))?;

                let pdb_path = pdb_loader::resolve_pdb(&pdb_info, Some(&pdb_cache_dir), &temp_dir)
                    .map_err(|e| format!("Download PDB: {e}"))?;

                let extractor =
                    PdbExtractor::open(&pdb_path).map_err(|e| format!("Parse PDB: {e}"))?;

                // Only collect names + kind — no JSON serialization here
                let mut symbols = Vec::new();
                for name in extractor.struct_names() {
                    symbols.push(SymbolEntry {
                        name: name.to_string(),
                        kind: SymbolKind::Struct,
                    });
                }
                for name in extractor.function_names() {
                    symbols.push(SymbolEntry {
                        name: name.to_string(),
                        kind: SymbolKind::Function,
                    });
                }
                for name in extractor.enum_names() {
                    symbols.push(SymbolEntry {
                        name: name.to_string(),
                        kind: SymbolKind::Enum,
                    });
                }
                symbols.sort_by(|a, b| a.name.cmp(&b.name));

                Ok(PdbData::new(file_name_owned, symbols, extractor))
            },
            Message::PdbLoaded,
        )
    }

    /// Export the selected PE file from WIM to a user-chosen location
    fn start_export_file(&mut self) -> Task<Message> {
        let (Some(info), Some(image), Some(file_name)) =
            (self.mount_info.clone(), self.selected_image.clone(), self.selected_file.clone())
        else {
            return Task::none();
        };

        let wim_inner_path = match self.find_full_pe_path(&file_name) {
            Some(p) => p,
            None => return Task::none(),
        };

        self.pdb_status = format!("Exporting {file_name}...");
        let image_idx = image.index;
        let wim_path = info.wim_path;

        Task::perform(
            async move {
                let save_handle = rfd::AsyncFileDialog::new()
                    .set_title("Export PE File")
                    .set_file_name(&file_name)
                    .save_file()
                    .await;

                let Some(handle) = save_handle else {
                    return Ok("Export cancelled".into());
                };
                let dest = handle.path().to_path_buf();

                let temp_dir = std::env::temp_dir().join("cocytus_export");
                std::fs::create_dir_all(&temp_dir).map_err(|e| format!("Temp dir: {e}"))?;

                wim::extract_file(&wim_path, &temp_dir, image_idx, &wim_inner_path)
                    .map_err(|e| format!("Extract: {e}"))?;

                let pe_filename = wim_inner_path
                    .rsplit(['\\', '/'])
                    .next()
                    .unwrap_or(&wim_inner_path);
                let extracted = temp_dir.join(pe_filename);

                std::fs::copy(&extracted, &dest)
                    .map_err(|e| format!("Copy: {e}"))?;

                Ok(format!("Exported {pe_filename}"))
            },
            Message::ExportComplete,
        )
    }

    /// Export the PDB for the selected PE file to a user-chosen location
    fn start_export_pdb(&mut self) -> Task<Message> {
        let (Some(info), Some(image), Some(file_name)) =
            (self.mount_info.clone(), self.selected_image.clone(), self.selected_file.clone())
        else {
            return Task::none();
        };

        let wim_inner_path = match self.find_full_pe_path(&file_name) {
            Some(p) => p,
            None => return Task::none(),
        };

        self.pdb_status = format!("Exporting PDB for {file_name}...");
        let image_idx = image.index;
        let wim_path = info.wim_path;
        let pdb_cache_dir = self.config.pdb_dir();

        Task::perform(
            async move {
                let temp_dir = std::env::temp_dir().join("cocytus_export_pdb");
                std::fs::create_dir_all(&temp_dir).map_err(|e| format!("Temp dir: {e}"))?;

                // Extract PE to temp so we can parse PDB info from it
                wim::extract_file(&wim_path, &temp_dir, image_idx, &wim_inner_path)
                    .map_err(|e| format!("Extract PE: {e}"))?;

                let pe_filename = wim_inner_path
                    .rsplit(['\\', '/'])
                    .next()
                    .unwrap_or(&wim_inner_path);
                let pe_path = temp_dir.join(pe_filename);

                let pdb_info = pdb_loader::parse_pdb_info(&pe_path)
                    .map_err(|e| format!("PDB info: {e}"))?;

                let pdb_name = pdb_info.pdb_name.clone();

                // Show save dialog with the actual PDB filename
                let save_handle = rfd::AsyncFileDialog::new()
                    .set_title("Export PDB File")
                    .set_file_name(&pdb_name)
                    .add_filter("PDB files", &["pdb"])
                    .save_file()
                    .await;

                let Some(handle) = save_handle else {
                    return Ok("Export cancelled".into());
                };
                let dest = handle.path().to_path_buf();

                // Resolve PDB (cache or download)
                let downloaded = pdb_loader::resolve_pdb(&pdb_info, Some(&pdb_cache_dir), &temp_dir)
                    .map_err(|e| format!("Download PDB: {e}"))?;

                std::fs::copy(&downloaded, &dest)
                    .map_err(|e| format!("Copy: {e}"))?;

                Ok(format!("Exported {pdb_name}"))
            },
            Message::ExportComplete,
        )
    }

    /// Extract the selected PE to temp and parse its header details
    fn start_pe_details(&mut self) -> Task<Message> {
        let (Some(info), Some(image), Some(file_name)) =
            (self.mount_info.clone(), self.selected_image.clone(), self.selected_file.clone())
        else {
            return Task::none();
        };

        let wim_inner_path = match self.find_full_pe_path(&file_name) {
            Some(p) => p,
            None => return Task::none(),
        };

        let image_idx = image.index;
        let wim_path = info.wim_path;

        Task::perform(
            async move {
                let temp_dir = std::env::temp_dir().join("cocytus_details");
                std::fs::create_dir_all(&temp_dir).map_err(|e| format!("Temp dir: {e}"))?;

                wim::extract_file(&wim_path, &temp_dir, image_idx, &wim_inner_path)
                    .map_err(|e| format!("Extract PE: {e}"))?;

                let pe_filename = wim_inner_path
                    .rsplit(['\\', '/'])
                    .next()
                    .unwrap_or(&wim_inner_path);

                crate::utils::pe_info::parse(&temp_dir.join(pe_filename))
            },
            Message::PeDetailsLoaded,
        )
    }

    /// Load a PDB file directly (no ISO/WIM extraction)
    fn start_direct_pdb_load(&mut self, pdb_path: std::path::PathBuf) -> Task<Message> {
        let file_name = pdb_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        self.loading_pdb = true;
        self.status = format!("Loading {}...", file_name);
        self.pdb_data = None;
        self.selected_symbol_idx = None;
        self.json_content = text_editor::Content::with_text("");
        self.json_context_menu = false;
        self.symbol_filter.clear();
        self.symbol_checks.clear();
        self.json_cache.clear();
        self.json_tabs.clear();
        self.context_menu_symbol_idx = None;
        self.symbol_tab = SymbolTab::Symbols;

        Task::perform(
            async move {
                let extractor =
                    PdbExtractor::open(&pdb_path).map_err(|e| format!("Parse PDB: {e}"))?;

                let mut symbols = Vec::new();
                for name in extractor.struct_names() {
                    symbols.push(SymbolEntry {
                        name: name.to_string(),
                        kind: SymbolKind::Struct,
                    });
                }
                for name in extractor.function_names() {
                    symbols.push(SymbolEntry {
                        name: name.to_string(),
                        kind: SymbolKind::Function,
                    });
                }
                for name in extractor.enum_names() {
                    symbols.push(SymbolEntry {
                        name: name.to_string(),
                        kind: SymbolKind::Enum,
                    });
                }
                symbols.sort_by(|a, b| a.name.cmp(&b.name));

                Ok(PdbData::new(file_name, symbols, extractor))
            },
            Message::DirectPdbLoaded,
        )
    }

    /// Find the full WIM inner path for a PE filename
    fn find_full_pe_path(&self, file_name: &str) -> Option<String> {
        self.pe_files
            .iter()
            .find(|p| {
                p.rsplit(['\\', '/'])
                    .next()
                    .is_some_and(|n| n.eq_ignore_ascii_case(file_name))
            })
            .cloned()
    }
}
