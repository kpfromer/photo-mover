use anyhow::Result;
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

pub fn apply_file_operations(operations: &[FileOperation]) -> Result<()> {
    for operation in operations {
        apply_file_operation(operation)?;
    }

    Ok(())
}
