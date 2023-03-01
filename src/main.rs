#[macro_use]
extern crate lazy_static;

pub mod exif;
pub mod operation;

use anyhow::Result;
use clap::Parser;
use exif::*;
use operation::*;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    search: PathBuf,

    #[arg(short, long)]
    out: PathBuf,

    #[arg(short, long, default_value_t = OperationType::Copy)]
    movement_type: OperationType,
    // TODO: implement conflicts option
    // TODO: implement no date option
}

fn main() -> Result<()> {
    let args = Args::parse();

    if !exiftool_exists() {
        anyhow::bail!("exiftool is not installed");
    }

    let operations = get_move_operations(&args.search, &args.out)?;
    println!("Found {} files", operations.len());
    let operation = Operation {
        config: OperationConfig {
            operation_type: args.movement_type,
            handle_conflicts: HandleFileConflict::MoveToDuplicateFolder(PathBuf::from(
                "duplicates",
            )),
            // TODO: get working
            handle_no_date: HandleNoDate::DoNothing,
        },
        move_operations: operations,
    };
    perform_operation(&operation)?;

    Ok(())
}
