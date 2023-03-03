#[macro_use]
extern crate lazy_static;

pub mod exif;
pub mod file;
pub mod operation;

use anyhow::Result;
use clap::Parser;
use exif::*;
use operation::*;
use spinoff::{spinners, Color, Spinner};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    source: PathBuf,

    #[arg(short, long)]
    destination: PathBuf,

    #[arg(short, long, default_value_t = OperationType::Copy)]
    movement_type: OperationType,

    #[arg(long)]
    recursive: bool,

    #[arg(short, long)]
    overwrite_duplicates: bool,
    #[arg(long)]
    duplicate_folder: Option<PathBuf>,

    #[arg(long)]
    dry_run: bool,

    #[arg(long)]
    no_date_folder: Option<PathBuf>,

    #[arg(long, default_value = "%Y/%m/%d")]
    date_folder_format: String,
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
        let handle_no_date = match args.no_date_folder {
            Some(path) => HandleNoDate::MoveToNoDateFolder(path),
            None => HandleNoDate::DoNothing,
        };
        OperationConfig {
            output_folder: args.destination,
            operation_type: args.movement_type,
            handle_conflicts,
            handle_no_date,
            date_folder_format: args.date_folder_format,
        }
    };

    let spinner = Spinner::new(spinners::Dots, "Finding files...", Color::Blue);
    let files = get_date_time_multiple(&args.source, args.recursive)?
        .into_iter()
        .map(|file| match file.date_time {
            Some(datetime) => OperationFile::ExifFile(DateTimeFile {
                path: file.path,
                date_time: datetime,
            }),
            None => OperationFile::NoDateFile(file.path),
        })
        .collect();
    spinner.success("Loaded files!");

    let operation = Operation { config, files };
    // TODO: fix multiple duplicates with same name
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
