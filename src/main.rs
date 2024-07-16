use clap::Parser;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the file to read, warning it will be removed
    #[clap(short, long)]
    file: String,

    /// Percent of file at which truncate should be performed
    #[clap(short, long, default_value = "50")]
    truncate: f64,
}

fn cat_file(file_path: &str, drop_percent: f64) -> anyhow::Result<()> {
    let mut buffer = Vec::new();
    let mut stdout = io::stdout();
    // Read the file
    'outer: loop {
        let drop_bytes =
            (drop_percent / 100.0) * std::fs::metadata(file_path).unwrap().len() as f64;
        let drop_bytes = drop_bytes as usize + 10000000;
        log::info!(
            "Reading file {}, drop at {}% - limit bytes {}",
            file_path,
            drop_percent,
            drop_bytes
        );
        //open file and check if you have write permission at the same time
        let mut file = OpenOptions::new()
            .read(true)
            .truncate(false)
            .open(file_path)
            .unwrap();
        let file_path_copy = format!("{file_path}.part");
        let mut local_bytes_read = 0;
        loop {
            buffer.resize(1000 * 1000, 0);
            let bytes_read = file.read(buffer.as_mut_slice()).unwrap();
            if bytes_read == 0 {
                break;
            }
            if bytes_read < buffer.len() {
                buffer.resize(bytes_read, 0);
            }
            local_bytes_read += bytes_read;

            stdout.write_all(&buffer)?;

            stdout.flush()?;
            if local_bytes_read > drop_bytes {
                break;
            }
        }
        let mut bytes_written = 0;
        let mut file_copy = File::create(&file_path_copy).unwrap();
        log::info!(
            "Writing rest of the file to {}. {}/{}",
            file_path_copy,
            local_bytes_read,
            std::fs::metadata(file_path).unwrap().len()
        );
        loop {
            buffer.resize(1000 * 1000, 0);
            let bytes_read = file.read(buffer.as_mut_slice()).unwrap();
            if bytes_read == 0 {
                break;
            }
            if bytes_read < buffer.len() {
                buffer.resize(bytes_read, 0);
            }
            file_copy.write_all(buffer.as_slice()).unwrap();
            bytes_written += bytes_read;
        }
        log::info!(
            "Finished reading and copying file, bytes written: {}",
            bytes_written
        );
        //remove the file

        file_copy.flush().unwrap();
        drop(file);
        drop(file_copy);
        std::fs::remove_file(file_path).unwrap();
        if bytes_written == 0 {
            log::info!("Removing empty file {}", file_path_copy);
            std::fs::remove_file(&file_path_copy).unwrap();
            break 'outer;
        }
        std::fs::rename(&file_path_copy, file_path).unwrap();
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    env::set_var(
        "RUST_LOG",
        env::var("RUST_LOG").unwrap_or("info".to_string()),
    );
    env_logger::init();
    let args = Args::parse();
    cat_file(&args.file, args.truncate)
}
