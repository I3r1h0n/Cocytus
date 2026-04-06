use std::path::PathBuf;
use std::sync::Arc;

use iced::widget::{pane_grid, text_editor};
use iced::Point;

use crate::extractors::iso::MountInfo;
use crate::extractors::pdb::PdbExtractor;
use crate::extractors::wim::WimImage;

//Symbol types

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Struct,
    Function,
    Enum,
}

#[derive(Debug, Clone)]
pub struct SymbolEntry {
    pub name: String,
    pub kind: SymbolKind,
}

// PDB data (lazy JSON via Arc<PdbExtractor>)

#[derive(Debug, Clone)]
pub struct PdbData {
    pub file_name: String,
    pub symbols: Vec<SymbolEntry>,
    extractor: Arc<PdbExtractor>,
}

impl PdbData {
    pub fn new(file_name: String, symbols: Vec<SymbolEntry>, extractor: PdbExtractor) -> Self {
        Self {
            file_name,
            symbols,
            extractor: Arc::new(extractor),
        }
    }

    /// Serialize a single symbol
    pub fn get_json(&self, entry: &SymbolEntry) -> String {
        let json = match entry.kind {
            SymbolKind::Struct => self.extractor.get_struct(&entry.name),
            SymbolKind::Function => self.extractor.get_function(&entry.name),
            SymbolKind::Enum => self.extractor.get_enum(&entry.name),
        };
        json.unwrap_or_else(|| "No data".to_string())
    }

    /// Get a shared handle to the extractor
    pub fn extractor(&self) -> Arc<PdbExtractor> {
        Arc::clone(&self.extractor)
    }
}

/// Symbol tab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolTab {
    Symbols,
    Selected,
}

/// View mode for the right pane
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Json,
    Cpp,
}

/// Export dialog state
pub struct ExportDialog {
    pub format: ViewMode,
    pub path: Option<PathBuf>,
    pub symbols: Vec<(String, SymbolKind)>,
    pub enabled: Vec<bool>,
    pub exporting: bool,
    pub progress: usize,
    pub current_name: String,
    pub buffer: String,
}

/// Options dialog state — editable copy of config fields
pub struct OptionsDialog {
    pub pdb_path: String,
    pub default_view: ViewMode,
}

/// Menu actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileMenuAction {
    OpenNewIso,
    OpenPdb,
    Quit,
}

impl std::fmt::Display for FileMenuAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenNewIso => write!(f, "Open new ISO"),
            Self::OpenPdb => write!(f, "Open PDB"),
            Self::Quit => write!(f, "Quit"),
        }
    }
}

/// Context menu actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextMenuAction {
    Load,
    ExportPdb,
    ExportFile,
    Details,
}

impl std::fmt::Display for ContextMenuAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load => write!(f, "Load"),
            Self::ExportPdb => write!(f, "Export PDB"),
            Self::ExportFile => write!(f, "Export File"),
            Self::Details => write!(f, "Details"),
        }
    }
}

/// App Messages
#[derive(Debug, Clone)]
pub enum Message {
    BrowseIso,
    IsoPicked(Option<PathBuf>),
    IsoMounted(Result<(MountInfo, Vec<WimImage>), String>),
    ImageSelected(WimImage),
    Continue,
    BrowsePdb,
    PdbFilePicked(Option<PathBuf>),
    DirectPdbLoaded(Result<PdbData, String>),
    PeFilesLoaded(Vec<String>),
    PaneResized(pane_grid::ResizeEvent),
    ToggleFileMenu,
    FileMenu(FileMenuAction),
    PeFilterChanged(String),
    SelectFile(String),
    ShowContextMenu(String),
    ContextAction(ContextMenuAction),
    DismissContextMenu,
    CursorMoved(Point),
    PdbLoaded(Result<PdbData, String>),
    SelectSymbol(usize),
    ToggleSymbolCheck(usize),
    SymbolFilterChanged(String),
    ToggleShowStructs(bool),
    ToggleShowFunctions(bool),
    ToggleShowEnums(bool),
    JsonEditorAction(text_editor::Action),
    ShowJsonContextMenu,
    JsonContextCopy,
    JsonContextSearch,
    DismissJsonContextMenu,
    SwitchSymbolTab(SymbolTab),
    ClearSelected,
    SymbolJsonReady(usize, String),
    ShowSymbolContextMenu(usize),
    DismissSymbolContextMenu,
    SymbolContextToggleCheck(usize),
    OpenSymbolNewTab(usize),
    CloseJsonTab(usize),
    SwitchJsonTab(usize),
    ExportComplete(Result<String, String>),
    DismissIsoInfo,
    SetViewMode(ViewMode),
    ShowExportDialog,
    DismissExportDialog,
    SetExportDialogFormat(ViewMode),
    BrowseExportPath,
    ExportPathPicked(Option<PathBuf>),
    ToggleExportSymbol(usize),
    StartSymbolExport,
    ExportTick,
    ExportWriteComplete(Result<String, String>),
    ShowOptions,
    DismissOptions,
    OptionsPdbPathChanged(String),
    BrowseOptionsPdbPath,
    OptionsPdbPathPicked(Option<PathBuf>),
    OptionsDefaultViewChanged(ViewMode),
    SaveOptions,
    PeDetailsLoaded(Result<crate::utils::pe_info::PeDetails, String>),
    DismissPeDetails,
    ShowAbout,
    DismissAbout,
    OpenUrl(String),
    Noop,
}
