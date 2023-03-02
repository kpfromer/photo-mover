use crate::file::*;
use anyhow::Result;
use chrono::NaiveDateTime;
use std::{collections::HashSet, fmt, path::PathBuf};

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
    pub output_folder: PathBuf,
    pub operation_type: OperationType,
    pub handle_conflicts: HandleFileConflict,
    pub handle_no_date: HandleNoDate,
    pub date_folder_format: String,
}

#[derive(Debug)]
pub struct DateTimeFile {
    pub date_time: NaiveDateTime,
    pub path: PathBuf,
}

#[derive(Debug)]
pub enum OperationFile {
    ExifFile(DateTimeFile),
    NoDateFile(PathBuf),
}

#[derive(Debug)]
pub struct Operation {
    pub files: Vec<OperationFile>,
    pub config: OperationConfig,
}

pub struct OperationResults {
    pub no_duplicates: usize,
    pub duplicates: usize,
    pub no_date: usize,
}

fn find_duplicates(
    file_operations: &[FileOperation],
) -> (Vec<&FileOperation>, Vec<&FileOperation>) {
    let mut not_duplicates = Vec::new();
    let mut duplicates = Vec::new();
    let mut seen = HashSet::new();
    for operation in file_operations {
        let destination = match operation {
            FileOperation::Copy { destination, .. } => destination,
            FileOperation::Move { destination, .. } => destination,
        };
        // check for duplicates in operations or disk
        if seen.contains(destination) || destination.exists() {
            duplicates.push(operation);
        } else {
            not_duplicates.push(operation);
        }
        seen.insert(destination.clone());
    }
    (not_duplicates, duplicates)
}

pub fn move_operation_to_file_operation(
    move_operation: &MoveOperation,
    operation_type: &OperationType,
) -> FileOperation {
    match operation_type {
        OperationType::Copy => FileOperation::Copy {
            source: move_operation.source.clone(),
            destination: move_operation.destination.clone(),
        },
        OperationType::Move => FileOperation::Move {
            source: move_operation.source.clone(),
            destination: move_operation.destination.clone(),
        },
    }
}

pub fn perform_operation(operation: &Operation, dry_run: bool) -> Result<OperationResults> {
    let mut no_date_files_len = 0;
    let all_file_operations = operation
        .files
        .iter()
        .filter_map(|op| match op {
            OperationFile::ExifFile(exif_file) => {
                let source = exif_file.path.clone();
                let mut destination = operation.config.output_folder.clone();
                destination.push(
                    exif_file
                        .date_time
                        .format(&operation.config.date_folder_format)
                        .to_string(),
                );
                destination.push(exif_file.path.file_name().unwrap());

                Some(match operation.config.operation_type {
                    OperationType::Copy => FileOperation::Copy {
                        source,
                        destination,
                    },
                    OperationType::Move => FileOperation::Move {
                        source,
                        destination,
                    },
                })
            }
            OperationFile::NoDateFile(path) => match &operation.config.handle_no_date {
                HandleNoDate::DoNothing => None,
                HandleNoDate::MoveToNoDateFolder(folder) => {
                    let source = path.clone();
                    let mut destination = folder.clone();
                    destination.push(path.file_name().unwrap());
                    no_date_files_len += 1;

                    Some(match operation.config.operation_type {
                        OperationType::Copy => FileOperation::Copy {
                            source,
                            destination,
                        },
                        OperationType::Move => FileOperation::Move {
                            source,
                            destination,
                        },
                    })
                }
            },
        })
        .collect::<Vec<FileOperation>>();

    // Find duplicates
    let (not_duplicates, duplicates) = find_duplicates(&all_file_operations);
    let not_duplicates_len = not_duplicates.len();
    let duplicates_len = duplicates.len();

    let mut file_operations = Vec::new();

    if !dry_run {
        // Handle duplicates
        match &operation.config.handle_conflicts {
            HandleFileConflict::DoNothing => {
                duplicates.into_iter().cloned().for_each(|op| {
                    file_operations.push(op);
                });
                not_duplicates.into_iter().cloned().for_each(|op| {
                    file_operations.push(op);
                });
            }
            HandleFileConflict::Overwrite => {
                not_duplicates.into_iter().cloned().for_each(|op| {
                    file_operations.push(op);
                });
                duplicates.into_iter().cloned().for_each(|op| {
                    file_operations.push(op);
                });
            }
            HandleFileConflict::Rename => {
                unimplemented!("Rename not implemented yet")
            }
            HandleFileConflict::MoveToDuplicateFolder(folder) => {
                not_duplicates.into_iter().cloned().for_each(|op| {
                    file_operations.push(op);
                });

                // TODO WHAT IF MULTIPLE DUPLICATES WITH SAME NAME
                for duplicate in duplicates.into_iter() {
                    let file_operation = match duplicate {
                        FileOperation::Copy {
                            source,
                            destination: _,
                        } => FileOperation::Copy {
                            source: source.clone(),
                            destination: folder.join(source.file_name().unwrap()),
                        },
                        FileOperation::Move {
                            source,
                            destination: _,
                        } => FileOperation::Move {
                            source: source.clone(),
                            destination: folder.join(source.file_name().unwrap()),
                        },
                    };
                    file_operations.push(file_operation);
                }
            }
        }

        apply_file_operations(&file_operations)?;
    }

    Ok(OperationResults {
        no_duplicates: not_duplicates_len,
        duplicates: duplicates_len,
        // TODO
        no_date: no_date_files_len,
    })
}
