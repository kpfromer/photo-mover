use anyhow::{Context, Result};
use chrono::prelude::*;
use std::path::{Path, PathBuf};
use which::which;

#[derive(Debug)]
pub struct ExifFile {
    pub path: PathBuf,
    pub date_time: Option<NaiveDateTime>,
}

fn command_exists(cmd: &str) -> bool {
    which(cmd).is_ok()
}

pub fn exiftool_exists() -> bool {
    command_exists("exiftool")
}

pub fn get_date_time_multiple(path: &Path, recursive: bool) -> Result<Vec<ExifFile>> {
    let output = if recursive {
        std::process::Command::new("exiftool")
            .arg("-T")
            .arg("-r")
            .arg("-Directory")
            .arg("-Filename")
            .arg("-DateTimeOriginal")
            .arg(path)
            .output()
            .context("failed to execute exiftool")?
    } else {
        std::process::Command::new("exiftool")
            .arg("-T")
            .arg("-Directory")
            .arg("-Filename")
            .arg("-DateTimeOriginal")
            .arg(path)
            .output()
            .context("failed to execute exiftool")?
    };

    let output = String::from_utf8(output.stdout).context("failed to parse exiftool output")?;

    let exif_files = output
        .lines()
        .filter_map(|line| {
            let mut parts = line.split('\t');
            let first_part = parts.next().unwrap();

            if first_part.to_lowercase().starts_with("warning") {
                return None;
            }

            let mut path = PathBuf::from(first_part);
            path.push(parts.next().unwrap());

            let date_time = parts.next().and_then(|date_time| {
                NaiveDateTime::parse_from_str(date_time, "%Y:%m:%d %H:%M:%S").ok()
            });
            Some(ExifFile { path, date_time })
        })
        .collect();

    Ok(exif_files)
}
