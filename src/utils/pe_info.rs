use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::utils::{read_u16, read_u32};

/// Parsed PE header details for display.
#[derive(Debug, Clone)]
pub struct PeDetails {
    pub file_name: String,
    pub file_size: u64,
    pub machine: String,
    pub subsystem: String,
    pub timestamp: String,
    pub sections: u16,
    pub image_size: u32,
    pub entry_point: u32,
    pub checksum: u32,
    pub linker_version: String,
    pub dll_characteristics: Vec<String>,
}

pub fn parse(path: &Path) -> Result<PeDetails, String> {
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".into());
    let file_size = std::fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| format!("metadata: {e}"))?;

    let mut f = File::open(path).map_err(|e| format!("open: {e}"))?;

    // DOS header
    let mut dos = [0u8; 64];
    f.read_exact(&mut dos).map_err(|e| format!("DOS header: {e}"))?;
    if &dos[0..2] != b"MZ" {
        return Err("not a valid PE (missing MZ)".into());
    }
    let pe_offset = read_u32(&dos, 60) as u64;

    // PE signature + COFF header (24 bytes)
    f.seek(SeekFrom::Start(pe_offset)).map_err(|e| format!("seek PE: {e}"))?;
    let mut pe_hdr = [0u8; 24];
    f.read_exact(&mut pe_hdr).map_err(|e| format!("PE header: {e}"))?;
    if &pe_hdr[0..4] != b"PE\0\0" {
        return Err("invalid PE signature".into());
    }

    let machine_raw = read_u16(&pe_hdr, 4);
    let sections = read_u16(&pe_hdr, 6);
    let timestamp_raw = read_u32(&pe_hdr, 8);
    let size_of_optional = read_u16(&pe_hdr, 20) as usize;

    // Optional header
    let mut opt = vec![0u8; size_of_optional];
    f.read_exact(&mut opt).map_err(|e| format!("optional header: {e}"))?;

    let magic = read_u16(&opt, 0);
    let linker_major = opt[2];
    let linker_minor = opt[3];

    let (entry_point, image_size, checksum, subsystem_raw, dll_char_raw) = match magic {
        0x20b => {
            // PE32+
            let ep = read_u32(&opt, 16);
            let isz = read_u32(&opt, 56);
            let cksum = read_u32(&opt, 64);
            let ss = read_u16(&opt, 68);
            let dc = read_u16(&opt, 70);
            (ep, isz, cksum, ss, dc)
        }
        0x10b => {
            // PE32
            let ep = read_u32(&opt, 16);
            let isz = read_u32(&opt, 56);
            let cksum = read_u32(&opt, 64);
            let ss = read_u16(&opt, 68);
            let dc = read_u16(&opt, 70);
            (ep, isz, cksum, ss, dc)
        }
        _ => return Err(format!("unknown PE magic: {magic:#X}")),
    };

    let machine = match machine_raw {
        0x014C => "x86 (i386)",
        0x0200 => "IA-64",
        0x8664 => "x64 (AMD64)",
        0xAA64 => "ARM64",
        0x01C0 => "ARM",
        0x01C4 => "ARMv7 Thumb-2",
        _ => "Unknown",
    }
    .to_string();

    let subsystem = match subsystem_raw {
        0 => "Unknown",
        1 => "Native",
        2 => "Windows GUI",
        3 => "Windows Console",
        5 => "OS/2 Console",
        7 => "POSIX Console",
        9 => "Windows CE",
        10 => "EFI Application",
        11 => "EFI Boot Service Driver",
        12 => "EFI Runtime Driver",
        13 => "EFI ROM",
        14 => "Xbox",
        16 => "Windows Boot Application",
        _ => "Other",
    }
    .to_string();

    let timestamp = format_timestamp(timestamp_raw);

    let linker_version = format!("{linker_major}.{linker_minor:02}");

    let mut dll_characteristics = Vec::new();
    if dll_char_raw & 0x0020 != 0 { dll_characteristics.push("High Entropy VA".into()); }
    if dll_char_raw & 0x0040 != 0 { dll_characteristics.push("Dynamic Base (ASLR)".into()); }
    if dll_char_raw & 0x0080 != 0 { dll_characteristics.push("Force Integrity".into()); }
    if dll_char_raw & 0x0100 != 0 { dll_characteristics.push("NX Compatible (DEP)".into()); }
    if dll_char_raw & 0x0200 != 0 { dll_characteristics.push("No Isolation".into()); }
    if dll_char_raw & 0x0400 != 0 { dll_characteristics.push("No SEH".into()); }
    if dll_char_raw & 0x0800 != 0 { dll_characteristics.push("No Bind".into()); }
    if dll_char_raw & 0x1000 != 0 { dll_characteristics.push("AppContainer".into()); }
    if dll_char_raw & 0x2000 != 0 { dll_characteristics.push("WDM Driver".into()); }
    if dll_char_raw & 0x4000 != 0 { dll_characteristics.push("Control Flow Guard".into()); }
    if dll_char_raw & 0x8000 != 0 { dll_characteristics.push("Terminal Server Aware".into()); }

    Ok(PeDetails {
        file_name,
        file_size,
        machine,
        subsystem,
        timestamp,
        sections,
        image_size,
        entry_point,
        checksum,
        linker_version,
        dll_characteristics,
    })
}

fn format_timestamp(ts: u32) -> String {
    if ts == 0 {
        return "N/A".into();
    }
    // Seconds since 1970-01-01 00:00:00 UTC
    let secs = ts as u64;
    let days = secs / 86400;
    let time = secs % 86400;
    let h = time / 3600;
    let m = (time % 3600) / 60;
    let s = time % 60;

    // Convert days since epoch to date (simplified leap year calculation)
    let (year, month, day) = days_to_ymd(days);
    format!("{year:04}-{month:02}-{day:02} {h:02}:{m:02}:{s:02} UTC")
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970;
    loop {
        let yd = if is_leap(year) { 366 } else { 365 };
        if days < yd { break; }
        days -= yd;
        year += 1;
    }
    let leap = is_leap(year);
    let month_days: [u64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut month = 0;
    for (i, &md) in month_days.iter().enumerate() {
        if days < md { month = i as u64 + 1; break; }
        days -= md;
    }
    (year, month, days + 1)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}
