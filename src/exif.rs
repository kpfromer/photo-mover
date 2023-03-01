use anyhow::{Context, Result};
use chrono::prelude::*;
use std::{collections::HashSet, path::Path};
use walkdir::{DirEntry, WalkDir};
use which::which;

lazy_static! {
    static ref EXIF_TOOL_VALID_FILE_EXTENSIONS: HashSet<&'static str> = {
        let extensions: HashSet<&str> = vec![
            "360", "3G2", "3GP2", "3GP", "3GPP", "AAX", "AI", "AIT", "ARQ", "ARW", "AVIF", "CR2",
            "CR3", "CRM", "CRW", "CIFF", "CS1", "DCP", "DNG", "DR4", "DVB", "EPS", "EPSF", "PS",
            "ERF", "EXIF", "EXV", "F4A", "F4B", "F4P", "F4V", "FFF", "FLIF", "GIF", "GPR", "HDP",
            "WDP", "JXR", "HEIC", "HEIF", "HIF", "ICC", "ICM", "IIQ", "IND", "INDD", "INDT",
            "INSP", "JP2", "JPF", "JPM", "JPX", "JPEG", "JPG", "JPE", "JXL", "LRV", "M4A", "M4B",
            "M4P", "M4V", "MEF", "MIE", "MOS", "MOV", "QT", "MP4", "MPO", "MQV", "MRW", "NEF",
            "NRW", "ORF", "PDF", "PEF", "PNG", "JNG", "MNG", "PPM", "PBM", "PGM", "PSD", "PSB",
            "PSDT", "QTIF", "QTI", "QIF", "RAF", "RAW", "RW2", "RWL", "SR2", "SRW", "THM", "TIFF",
            "TIF", "VRD", "WEBP", "X3F", "XMP",
        ]
        .into_iter()
        .collect();
        extensions
    };
}

fn command_exists(cmd: &str) -> bool {
    which(cmd).is_ok()
}

pub fn exiftool_exists() -> bool {
    command_exists("exiftool")
}

pub fn get_date_time_original(path: &Path) -> Result<Option<NaiveDateTime>> {
    let output = std::process::Command::new("exiftool")
        .arg("-T")
        .arg(path)
        .arg("-DateTimeOriginal")
        .output()
        .context("failed to execute exiftool")?;

    let output = String::from_utf8(output.stdout).context("failed to parse exiftool output")?;

    Ok(NaiveDateTime::parse_from_str(&output.trim(), "%Y:%m:%d %H:%M:%S").ok())
}

pub fn matches_file_extensions(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .map(|ext| {
                let file_extension = ext.to_str().unwrap().to_uppercase();
                EXIF_TOOL_VALID_FILE_EXTENSIONS.contains(file_extension.as_str())
            })
            .unwrap_or(false)
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}
fn get_exif_file_count(path: &Path) -> Result<(u32, u32)> {
    let mut valid_exif_count: u32 = 0;
    let mut invalid_exif_count: u32 = 0;

    let walker = WalkDir::new(path).into_iter();
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry.context("failed to read directory entry")?;
        if matches_file_extensions(entry.path()) && get_date_time_original(entry.path())?.is_some()
        {
            valid_exif_count += 1;
        } else {
            invalid_exif_count += 1;
        }
    }

    Ok((valid_exif_count, invalid_exif_count))
}