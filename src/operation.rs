use crate::exif::*;
use anyhow::{Context, Result};
use chrono::prelude::*;
use std::{
    collections::HashSet,
    fmt,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug)]
pub enum HandleFileConflict {
    DoNothing,
    Overwrite,
    Rename,
    MoveToDuplicateFolder(PathBuf),
}

#[derive(Debug)]
pub enum HandleNoDate {
    DoNothing,
    MoveToNoDateFolder(PathBuf),
}

#[derive(Clone, Debug)]
pub enum OperationType {
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
pub struct MoveOperation {
    pub source: PathBuf,
    pub destination: PathBuf,
}

#[derive(Debug)]
pub struct OperationConfig {
    pub operation_type: OperationType,
    pub handle_conflicts: HandleFileConflict,
    pub handle_no_date: HandleNoDate,
}

#[derive(Debug)]
pub struct Operation {
    pub move_operations: Vec<MoveOperation>,
    pub config: OperationConfig,
}

pub struct OperationResults {
    pub no_duplicates: usize,
    pub duplicates: usize,
    pub no_date: usize,
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

pub fn perform_operation(operation: &Operation, dry_run: bool) -> Result<OperationResults> {
    // Find duplicates
    let (not_duplicates, duplicates) = find_duplicates(&operation.move_operations);

    if !dry_run {
        // Handle duplicates
        match &operation.config.handle_conflicts {
            HandleFileConflict::DoNothing => {
                for op in not_duplicates.iter() {
                    perform_move_operation(op, &operation.config.operation_type)?;
                }
            }
            HandleFileConflict::Overwrite => {
                for op in not_duplicates.iter() {
                    perform_move_operation(op, &operation.config.operation_type)?;
                }
                for op in duplicates.iter() {
                    perform_move_operation(op, &operation.config.operation_type)?;
                }
            }
            HandleFileConflict::Rename => {
                unimplemented!("Rename not implemented yet")
            }
            HandleFileConflict::MoveToDuplicateFolder(folder) => {
                for op in not_duplicates.iter() {
                    perform_move_operation(op, &operation.config.operation_type)?;
                }

                // TODO WHAT IF MULTIPLE DUPLICATES WITH SAME NAME
                for duplicate in duplicates.iter() {
                    std::fs::create_dir_all(folder).context("failed to create duplicate folder")?;
                    match operation.config.operation_type {
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
    }

    Ok(OperationResults {
        no_duplicates: not_duplicates.len(),
        duplicates: duplicates.len(),
        no_date: 0,
    })
}
