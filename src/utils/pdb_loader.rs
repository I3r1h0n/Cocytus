use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::debug;
use crate::error::AppError;
use crate::utils::{read_u16, read_u32};

const IMAGE_DEBUG_TYPE_CODEVIEW: u32 = 2;
const CODEVIEW_PDB70_SIGNATURE: [u8; 4] = *b"RSDS";

/// PDB identity
pub struct PdbInfo {
    /// GUID Ыекштп
    pub guid: String,
    /// Age field
    pub age: u32,
    /// PDB file name
    pub pdb_name: String,
}

/// Parse the PE debug directory
pub fn parse_pdb_info(pe_path: &Path) -> Result<PdbInfo, AppError> {
    let mut f = File::open(pe_path)
        .map_err(|e| AppError::Pdb(format!("cannot open {}: {e}", pe_path.display())))?;

    // --- DOS header ---
    let mut dos = [0u8; 64];
    f.read_exact(&mut dos)
        .map_err(|e| AppError::Pdb(format!("cannot read DOS header: {e}")))?;

    if &dos[0..2] != b"MZ" {
        return Err(AppError::Pdb("not a valid PE file (missing MZ)".into()));
    }

    let pe_offset = read_u32(&dos, 60) as u64;

    // PE signature + COFF header (24 bytes)
    f.seek(SeekFrom::Start(pe_offset))
        .map_err(|e| AppError::Pdb(format!("seek to PE header: {e}")))?;

    let mut pe_hdr = [0u8; 24];
    f.read_exact(&mut pe_hdr)
        .map_err(|e| AppError::Pdb(format!("read PE header: {e}")))?;

    if &pe_hdr[0..4] != b"PE\0\0" {
        return Err(AppError::Pdb("invalid PE signature".into()));
    }

    let number_of_sections = read_u16(&pe_hdr, 6) as usize;
    let size_of_optional_header = read_u16(&pe_hdr, 20) as usize;

    if size_of_optional_header == 0 {
        return Err(AppError::Pdb("no optional header in PE".into()));
    }

    // Optional header
    let opt_offset = pe_offset + 24;
    f.seek(SeekFrom::Start(opt_offset))
        .map_err(|e| AppError::Pdb(format!("seek to optional header: {e}")))?;

    let mut opt_hdr = vec![0u8; size_of_optional_header];
    f.read_exact(&mut opt_hdr)
        .map_err(|e| AppError::Pdb(format!("read optional header: {e}")))?;

    let magic = read_u16(&opt_hdr, 0);
    // Data directories start at offset 112 (PE32+) or 96 (PE32)
    let dd_base = match magic {
        0x20b => 112, // PE32+
        0x10b => 96,  // PE32
        _ => return Err(AppError::Pdb(format!("unknown optional header magic: {magic:#x}"))),
    };

    let number_of_rva_and_sizes = read_u32(&opt_hdr, dd_base - 4) as usize;
    if number_of_rva_and_sizes <= 6 {
        return Err(AppError::Pdb("PE has no debug data directory entry".into()));
    }

    // Debug directory is entry index 6 (each entry = 8 bytes: RVA + Size)
    let debug_rva = read_u32(&opt_hdr, dd_base + 6 * 8) as usize;
    let debug_size = read_u32(&opt_hdr, dd_base + 6 * 8 + 4) as usize;

    if debug_rva == 0 || debug_size == 0 {
        return Err(AppError::Pdb("debug directory is empty".into()));
    }

    // Section headers: translate RVA to file offset
    let sections_offset = opt_offset + size_of_optional_header as u64;
    f.seek(SeekFrom::Start(sections_offset))
        .map_err(|e| AppError::Pdb(format!("seek to section headers: {e}")))?;

    let mut sections = vec![0u8; number_of_sections * 40];
    f.read_exact(&mut sections)
        .map_err(|e| AppError::Pdb(format!("read section headers: {e}")))?;

    let rva_to_offset = |rva: usize| -> Result<usize, AppError> {
        for i in 0..number_of_sections {
            let base = i * 40;
            let virtual_address = read_u32(&sections, base + 12) as usize;
            let virtual_size = read_u32(&sections, base + 8) as usize;
            let raw_data_ptr = read_u32(&sections, base + 20) as usize;
            if rva >= virtual_address && rva < virtual_address + virtual_size {
                return Ok(rva - virtual_address + raw_data_ptr);
            }
        }
        Err(AppError::Pdb(format!("cannot map RVA {rva:#x} to file offset")))
    };

    let debug_file_offset = rva_to_offset(debug_rva)?;

    // Read debug directory entries
    f.seek(SeekFrom::Start(debug_file_offset as u64))
        .map_err(|e| AppError::Pdb(format!("seek to debug directory: {e}")))?;

    let num_entries = debug_size / 28;
    let mut debug_dir = vec![0u8; num_entries * 28];
    f.read_exact(&mut debug_dir)
        .map_err(|e| AppError::Pdb(format!("read debug directory: {e}")))?;

    for i in 0..num_entries {
        let base = i * 28;
        let dir_type = read_u32(&debug_dir, base + 12);
        if dir_type != IMAGE_DEBUG_TYPE_CODEVIEW {
            continue;
        }

        let raw_data_size = read_u32(&debug_dir, base + 16) as usize;
        let raw_data_offset = read_u32(&debug_dir, base + 24) as u64;

        f.seek(SeekFrom::Start(raw_data_offset))
            .map_err(|e| AppError::Pdb(format!("seek to CodeView data: {e}")))?;

        let mut cv = vec![0u8; raw_data_size];
        f.read_exact(&mut cv)
            .map_err(|e| AppError::Pdb(format!("read CodeView data: {e}")))?;

        if cv.len() < 24 || cv[0..4] != CODEVIEW_PDB70_SIGNATURE {
            continue;
        }

        // GUID: bytes 4..20, formatted as Microsoft mixed-endian GUID hex (no dashes)
        let d1 = read_u32(&cv, 4);
        let d2 = read_u16(&cv, 8);
        let d3 = read_u16(&cv, 10);
        let d4 = &cv[12..20];
        let guid = format!(
            "{:08X}{:04X}{:04X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
            d1, d2, d3, d4[0], d4[1], d4[2], d4[3], d4[4], d4[5], d4[6], d4[7]
        );

        let age = read_u32(&cv, 20);

        // PDB filename is a null-terminated UTF-8 string at offset 24
        let name_bytes = &cv[24..];
        let name_end = name_bytes.iter().position(|&b| b == 0).unwrap_or(name_bytes.len());
        let pdb_name = String::from_utf8_lossy(&name_bytes[..name_end]).to_string();

        debug!(
            "PE debug info: GUID={guid}, Age={age}, PDB={pdb_name}"
        );

        return Ok(PdbInfo {
            guid,
            age,
            pdb_name,
        });
    }

    Err(AppError::Pdb("no CodeView PDB 7.0 entry found in debug directory".into()))
}

/// Resolve a PDB: return it from cache, or download and cache it
///
/// Cache layout follows the symbol server convention: {cache_dir}/{pdb_name}/{GUID}{Age}/{pdb_name}`
///
/// If `cache_dir` is `None`, downloads to `fallback_dir` without caching
pub fn resolve_pdb(
    info: &PdbInfo,
    cache_dir: Option<&Path>,
    fallback_dir: &Path,
) -> Result<PathBuf, AppError> {
    let id = format!("{}{:X}", info.guid, info.age);

    // Try cache first
    if let Some(cache) = cache_dir {
        let cached = cache.join(&info.pdb_name).join(&id).join(&info.pdb_name);
        if cached.is_file() {
            debug!("PDB cache hit: {}", cached.display());
            return Ok(cached);
        }
    }

    // Download
    let dest_dir = match cache_dir {
        Some(cache) => {
            let dir = cache.join(&info.pdb_name).join(&id);
            std::fs::create_dir_all(&dir)
                .map_err(|e| AppError::Pdb(format!("create cache dir: {e}")))?;
            dir
        }
        None => fallback_dir.to_path_buf(),
    };

    download_pdb(info, &dest_dir)
}

/// Download the PDB from the Microsoft symbol server
///
/// Returns the path to the downloaded file
fn download_pdb(info: &PdbInfo, output_dir: &Path) -> Result<PathBuf, AppError> {
    let url = format!(
        "https://msdl.microsoft.com/download/symbols/{}/{}{:X}/{}",
        info.pdb_name, info.guid, info.age, info.pdb_name
    );

    debug!("Downloading PDB from: {url}");

    let response = ureq::get(&url)
        .call()
        .map_err(|e| AppError::Pdb(format!("HTTP request failed: {e}")))?;

    let status = response.status();
    if status != 200 {
        return Err(AppError::Pdb(format!("server returned HTTP {status}")));
    }

    let dest = output_dir.join(&info.pdb_name);
    let mut out_file = File::create(&dest)
        .map_err(|e| AppError::Pdb(format!("cannot create {}: {e}", dest.display())))?;

    let mut body = response.into_body();
    std::io::copy(&mut body.as_reader(), &mut out_file)
        .map_err(|e| AppError::Pdb(format!("writing PDB: {e}")))?;

    debug!("Saved PDB to: {}", dest.display());
    Ok(dest)
}
