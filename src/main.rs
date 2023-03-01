#[macro_use]
extern crate lazy_static;

pub mod exif;
pub mod file;
pub mod operation;

use anyhow::Result;
use clap::Parser;
use exif::*;
use operation::*;
use std::path::{Path, PathBuf};

fn exif_files_to_operations(files: &[ExifFile], output_folder: &Path) -> Vec<MoveOperation> {
    let mut move_operations = Vec::new();
    for file in files {
        // TODO: handle no date files
        if let Some(date_time) = file.date_time {
            let date = date_time.date();
            let mut destination = PathBuf::from(output_folder);
            destination.push(date.format("%Y/%m/%d").to_string());
            destination.push(file.path.file_name().unwrap());

            move_operations.push(MoveOperation {
                source: file.path.clone(),
                destination,
            });
        }
    }
    move_operations
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    source: PathBuf,

    #[arg(short, long)]
    destination: PathBuf,

    #[arg(short, long, default_value_t = OperationType::Copy)]
    movement_type: OperationType,

    #[arg(short, long)]
    overwrite_duplicates: bool,
    #[arg(long)]
    duplicate_folder: Option<PathBuf>,
    // TODO: implement no date option
    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if !exiftool_exists() {
        anyhow::bail!("exiftool is not installed");
    }

    let config = {
        let handle_conflicts = match (args.overwrite_duplicates, args.duplicate_folder) {
            (true, Some(_)) => {
                anyhow::bail!("cannot specify both overwrite_duplicates and duplicate_folder");
            }
            (true, None) => HandleFileConflict::Overwrite,
            (false, Some(path)) => HandleFileConflict::MoveToDuplicateFolder(path),
            (false, None) => HandleFileConflict::DoNothing,
        };
        let handle_no_date = HandleNoDate::DoNothing;
        OperationConfig {
            operation_type: args.movement_type,
            handle_conflicts,
            // TODO: get working
            handle_no_date,
        }
    };

    let move_operations =
        exif_files_to_operations(&get_exif_files(&args.source)?, &args.destination);

    let operation = Operation {
        config,
        file_operations: move_operations,
    };
    let OperationResults {
        no_duplicates,
        duplicates,
        no_date,
    } = perform_operation(&operation, args.dry_run)?;

    println!("Valid EXIF files: {}", no_duplicates);
    println!("Duplicates: {}", duplicates);
    println!("No date files: {}", no_date);

    Ok(())
}
