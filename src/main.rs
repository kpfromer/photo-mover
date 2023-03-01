#[macro_use]
extern crate lazy_static;

pub mod exif;

use anyhow::{Context, Result};
use chrono::prelude::*;
use clap::Parser;
use exif::*;
use std::{
    collections::HashSet,
    fmt,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

#[derive(Debug)]
enum HandleFileConflict {
    DoNothing,
    Overwrite,
    Rename,
    MoveToDuplicateFolder(PathBuf),
}

#[derive(Debug)]
enum HandleNoDate {
    DoNothing,
    MoveToNoDateFolder(PathBuf),
}

#[derive(Clone, Debug)]
enum OperationType {
    Copy,
    Move,
}
impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OperationType::Copy => "copy",
                OperationType::Move => "move",
            }
        )
    }
}

impl From<String> for OperationType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "copy" => OperationType::Copy,
            "move" => OperationType::Move,
            _ => panic!("Invalid operation type"),
        }
    }
}

#[derive(Debug)]
struct MoveOperation {
    source: PathBuf,
    destination: PathBuf,
}

#[derive(Debug)]
struct Operation {
    operation_type: OperationType,
    move_operations: Vec<MoveOperation>,
    handle_conflicts: HandleFileConflict,
    handle_no_date: HandleNoDate,
}

fn get_move_operations(path: &Path, output_folder: &Path) -> Result<Vec<MoveOperation>> {
    let mut operations = Vec::new();

    let walker = WalkDir::new(path).into_iter();
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry.context("failed to read directory entry")?;
        if matches_file_extensions(entry.path()) {
            let datetime = get_date_time_original(entry.path())?;

            if let Some(datetime) = datetime {
                let year = datetime.year();
                let month = datetime.month();
                let day = datetime.day();

                let mut destination = PathBuf::from(output_folder);
                destination.push(year.to_string());
                destination.push(month.to_string());
                destination.push(day.to_string());
                destination.push(entry.file_name());

                operations.push(MoveOperation {
                    source: entry.path().to_path_buf(),
                    destination,
                });
            }
        }
    }

    Ok(operations)
}

fn copy_files(operations: &[MoveOperation]) -> Result<()> {
    for operation in operations {
        std::fs::create_dir_all(operation.destination.parent().unwrap())
            .context("failed to create directories for file")?;
        let result = std::fs::copy(&operation.source, &operation.destination);
        if let Err(e) = result {
            println!("Failed to copy file: {}", e);
        }
        // std::fs::rename(operation.source, operation.destination)?;
    }

    Ok(())
}

fn find_duplicates(operations: &[MoveOperation]) -> (Vec<&MoveOperation>, Vec<&MoveOperation>) {
    let mut not_duplicates = Vec::new();
    let mut duplicates = Vec::new();
    let mut seen = HashSet::new();
    for operation in operations {
        // check for duplicates in operations or disk
        if seen.contains(&operation.destination) || operation.destination.exists() {
            duplicates.push(operation);
        } else {
            not_duplicates.push(operation);
        }
        seen.insert(operation.destination.clone());
    }
    (not_duplicates, duplicates)
}

fn perform_move_operation(
    move_operation: &MoveOperation,
    operation_type: &OperationType,
) -> Result<()> {
    std::fs::create_dir_all(move_operation.destination.parent().unwrap())
        .context("failed to create directories for file")?;

    match operation_type {
        OperationType::Copy => {
            std::fs::copy(&move_operation.source, &move_operation.destination)
                .context("failed to copy file")?;
        }
        OperationType::Move => {
            std::fs::rename(&move_operation.source, &move_operation.destination)
                .context("failed to move file")?;
        }
    }
    Ok(())
}

fn perform_operation(operation: &Operation) -> Result<()> {
    // Find duplicates
    let (not_duplicates, duplicates) = find_duplicates(&operation.move_operations);

    // Handle duplicates
    match &operation.handle_conflicts {
        HandleFileConflict::DoNothing => {
            for op in not_duplicates {
                perform_move_operation(op, &operation.operation_type)?;
            }
        }
        HandleFileConflict::Overwrite => {
            for op in not_duplicates {
                perform_move_operation(op, &operation.operation_type)?;
            }
            for op in duplicates {
                perform_move_operation(op, &operation.operation_type)?;
            }
        }
        HandleFileConflict::Rename => {
            unimplemented!("Rename not implemented yet")
        }
        HandleFileConflict::MoveToDuplicateFolder(folder) => {
            for op in not_duplicates {
                perform_move_operation(op, &operation.operation_type)?;
            }

            // TODO WHAT IF MULTIPLE DUPLICATES WITH SAME NAME
            for duplicate in duplicates {
                std::fs::create_dir_all(folder).context("failed to create duplicate folder")?;
                match operation.operation_type {
                    OperationType::Copy => {
                        std::fs::copy(
                            &duplicate.source,
                            folder.join(duplicate.source.file_name().unwrap()),
                        )
                        .context("failed to copy duplicate file")?;
                    }
                    OperationType::Move => {
                        std::fs::rename(
                            &duplicate.source,
                            folder.join(duplicate.source.file_name().unwrap()),
                        )
                        .context("failed to move duplicate file")?;
                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    search: PathBuf,

    #[arg(short, long)]
    out: PathBuf,

    #[arg(short, long, default_value_t = OperationType::Copy)]
    movement_type: OperationType,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if !exiftool_exists() {
        anyhow::bail!("exiftool is not installed");
    }

    let operations = get_move_operations(&args.search, &args.out)?;
    println!("Found {} files", operations.len());
    let operation = Operation {
        operation_type: OperationType::Copy,
        move_operations: operations,
        handle_conflicts: HandleFileConflict::MoveToDuplicateFolder(PathBuf::from("duplicates")),
        // TODO: get working
        handle_no_date: HandleNoDate::DoNothing,
    };
    perform_operation(&operation)?;

    Ok(())
}
