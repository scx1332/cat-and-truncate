mod ops;
mod plan;

use crate::ops::{generate_random_file, generate_zero_file, truncate_file};
use crate::plan::{commit_plan, plan_chunks, plan_into_realization};
use clap::Parser;
use std::env;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the file to read, warning it will be removed
    #[clap(short, long)]
    file: String,

    /// Percent of file at which truncate should be performed
    #[clap(short, long)]
    chunk_size: Option<bytesize::ByteSize>,

    /// Dry run, do not perform any operations
    #[clap(long)]
    dry_run: bool,

    #[clap(long)]
    test_create_zero_file_size: Option<u64>,

    #[clap(long)]
    test_create_random_file_size: Option<u64>,

    #[clap(long)]
    test_create_ascii_file_size: Option<u64>,

    #[clap(long)]
    test_truncate_file_size: Option<u64>,

    #[clap(long)]
    plan_chunks: bool,

    #[clap(long)]
    test_random: bool,
}

fn cat_file(file_path: &str, chunk_size: u64, dry_run: bool) -> anyhow::Result<()> {
    let file_size = std::fs::metadata(file_path)?.len();
    let chunk_size = std::cmp::min(file_size, chunk_size) as u64;
    let plan = plan_chunks(chunk_size, file_size).unwrap();
    let operations = plan_into_realization(plan).unwrap();
    commit_plan(Some(file_path), &operations, dry_run)
}

fn main() -> anyhow::Result<()> {
    env::set_var(
        "RUST_LOG",
        env::var("RUST_LOG").unwrap_or("info".to_string()),
    );
    env_logger::init();
    let args = Args::parse();

    let mut test_run = false;

    if let Some(test_create_random_file_size) = args.test_create_random_file_size {
        generate_random_file(&args.file, test_create_random_file_size, false)?;
        test_run = true;
    }
    if let Some(test_create_ascii_file_size) = args.test_create_ascii_file_size {
        generate_random_file(&args.file, test_create_ascii_file_size, true)?;
        test_run = true;
    }
    if let Some(test_create_zero_file_size) = args.test_create_zero_file_size {
        generate_zero_file(&args.file, test_create_zero_file_size)?;
        test_run = true;
    }
    if let Some(test_truncate_size) = args.test_truncate_file_size {
        truncate_file(&args.file, test_truncate_size)?;
        test_run = true;
    }
    if args.plan_chunks {
        test_run = true;

        let plan = plan_chunks(1, 11).unwrap();
        let operations = plan_into_realization(plan).unwrap();
        commit_plan(None, &operations, true)?;
    }

    if !test_run {
        // get file size
        let file_size = std::fs::metadata(&args.file)?.len();
        let default_chunk_size = if file_size < 1024 * 1024 {
            50000
        } else if file_size < 1024 * 1024 * 1024 {
            file_size / 100
        } else if file_size < 1024 * 1024 * 1024 * 1024 {
            file_size / 500
        } else {
            file_size / 1000
        };
        return cat_file(
            &args.file,
            args.chunk_size
                .map(|v| v.as_u64())
                .unwrap_or(default_chunk_size),
            args.dry_run,
        );
    }
    Ok(())
}
