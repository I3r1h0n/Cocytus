use std::path::{Path, PathBuf};
use std::process::Command;

use crate::debug;
use crate::error::AppError;
use crate::extractors::wim::find_wim;

/// Information about a mounted Windows ISO image.
#[derive(Debug, Clone)]
pub struct MountInfo {
    /// Original path to the .iso file (needed for unmount)
    pub iso_path: PathBuf,
    /// Drive letter assigned by Windows
    pub drive: String,
    /// Path to the WIM file
    pub wim_path: PathBuf,
}

/// Mount an ISO and locate the WIM file inside it
pub fn mount(iso_path: &Path) -> Result<MountInfo, AppError> {
    let abs_path = std::fs::canonicalize(iso_path)?;
    let path_str = abs_path.to_string_lossy().replace("\\\\?\\", "");

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Mount-DiskImage -ImagePath '{}' -PassThru | Get-Volume | Select-Object -ExpandProperty DriveLetter",
                path_str
            ),
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Iso(format!("failed to mount: {stderr}")));
    }

    let drive_letter = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if drive_letter.is_empty() {
        return Err(AppError::Iso("mount succeeded but no drive letter returned".into()));
    }

    let drive = format!("{drive_letter}:");
    debug!("ISO mounted at {drive}\\");

    let wim_path = match find_wim(&drive) {
        Ok(p) => p,
        Err(e) => {
            unmount(&abs_path);
            return Err(e);
        }
    };

    Ok(MountInfo {
        iso_path: abs_path,
        drive,
        wim_path,
    })
}

/// Unmount a previously mounted ISO image
pub fn unmount(iso_path: &Path) {
    let path_str = iso_path.to_string_lossy().replace("\\\\?\\", "");
    let _ = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!("Dismount-DiskImage -ImagePath '{path_str}'"),
        ])
        .output();
    debug!("ISO unmounted.");
}
