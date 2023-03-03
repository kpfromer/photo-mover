use anyhow::Result;
use indicatif::ProgressBar;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum FileOperation {
    Copy {
        source: PathBuf,
        destination: PathBuf,
    },
    Move {
        source: PathBuf,
        destination: PathBuf,
    },
}

pub fn apply_file_operation(operation: &FileOperation) -> Result<()> {
    match operation {
        FileOperation::Copy {
            source,
            destination,
        } => {
            std::fs::create_dir_all(destination.parent().unwrap())?;
            std::fs::copy(source, destination).unwrap();
        }
        FileOperation::Move {
            source,
            destination,
        } => {
            std::fs::create_dir_all(destination.parent().unwrap())?;
            std::fs::rename(source, destination).unwrap();
        }
    }

    Ok(())
}

pub fn apply_file_operations(operations: &[FileOperation], progress_bars: bool) -> Result<()> {
    let pb = if progress_bars {
        Some(ProgressBar::new(operations.len() as u64))
    } else {
        None
    };
    for operation in operations {
        if let Some(pb) = &pb {
            pb.inc(1);
        }
        apply_file_operation(operation)?;
    }
    if let Some(pb) = &pb {
        pb.finish_with_message("Done moving files.");
    }

    Ok(())
}
