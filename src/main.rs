use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, Write, Read};
use std::path::Path;
use clap::Parser;

/// A simple program to read a file name and length from the command line
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the file to read
    #[clap(short, long)]
    file: Option<String>,

    /// Size of the chunk to read at once
    #[clap(short, long, default_value = "1000000")]
    chunk: usize,
}

fn cat_file(file_path: &Path) -> anyhow::Result<()> {
    let mut buffer = Vec::new();
    let mut stdout = io::stdout();
    // Read the file

    'outer: loop {
        //open file and check if you have write permission at the same time
        let mut file = OpenOptions::new().read(true).write(true).truncate(false).open(file_path).unwrap();
        let mut file_copy = File::create("random_1gb_file_copy.bin").unwrap();
        let mut local_bytes_read = 0;
        loop {
            let mut buffer = Vec::new();
            buffer.resize(1000 * 1000, 0);
            let bytes_read = file.read(&mut buffer.as_mut_slice())?;
            if bytes_read == 0 {
                break;
            }
            local_bytes_read += bytes_read;

            stdout.write_all(&buffer)?;

            stdout.flush()?;
            if local_bytes_read > 10000000 {
                break;
            }
        }
        log::info!("Writing rest of the file to file_copy {}/{}", local_bytes_read, std::fs::metadata("random_1gb_file.bin")?.len());
        loop {
            let bytes_read = file.read(&mut buffer.as_mut_slice())?;
            if bytes_read == 0 {
                break;
            }
            log::debug!("Read {} bytes", bytes_read);
            file_copy.write(&buffer.as_mut_slice())?;
        }
        log::info!("Finished reading and copying file");
        //remove the file

        drop(file);
        drop(file_copy);
        std::fs::remove_file("random_1gb_file.bin")?;
        if std::fs::metadata("random_1gb_file_copy.bin")?.len() == 0 {
            std::fs::remove_file("random_1gb_file_copy.bin")?;
            break 'outer;
        }
        std::fs::rename("random_1gb_file_copy.bin", "random_1gb_file.bin")?;
    }
    Ok(())
}


fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", env::var("RUST_LOG").unwrap_or("info".to_string()));
    env_logger::init();
    let args = Args::parse();



    Ok(())
}