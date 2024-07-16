use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use anyhow::bail;
use rand::{Rng, thread_rng};
use rand::distributions::{Alphanumeric, DistString};

fn truncate_file_int(file_path: &str, target_size: u64) -> anyhow::Result<()> {
    //1 open file
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(false)
        .open(file_path)?;

    let file_size = file.seek(SeekFrom::End(0))?;

    if file_size < target_size {
        bail!("File size is already smaller than target size {} vs {}", file_size, target_size);
    }
    if file_size == target_size {
        log::debug!("File size is already equal to target size {}", file_size);
        return Ok(());
    }
    log::debug!("Truncating file {} to target size {}", file_path, target_size);

    //2 seek to target size
    file.seek(SeekFrom::Start(target_size))?;

    //3 truncate file
    file.set_len(target_size)?;

    Ok(())
}

pub fn truncate_file(file_path: &str, target_size: u64) -> anyhow::Result<()> {
    match truncate_file_int(file_path, target_size) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Error truncating file {}: {}", file_path, e);
            Err(e)
        }
    }
}

fn generate_zero_file_int(file_path: &str, len: u64) -> anyhow::Result<()> {
    //1 open file
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_path)?;

    let buf = vec![0u8; 1000 * 1000];
    let mut bytes_left = len;
    while bytes_left > 0 {
        let bytes_to_write = std::cmp::min(buf.len() as u64, bytes_left);
        file.write_all(&buf[0..bytes_to_write as usize])?;
        bytes_left -= bytes_to_write;
    }
    Ok(())
}

pub fn generate_zero_file(file_path: &str, len: u64) -> anyhow::Result<()> {
    match generate_zero_file_int(file_path, len) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Error generating zero file {}: {}", file_path, e);
            Err(e)
        }
    }
}

fn generate_random_file_int(file_path: &str, len: u64, is_ascii: bool) -> anyhow::Result<()> {
    //1 open file
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_path)?;

    let mut buf = vec![0u8; 1000 * 1000];
    let mut bytes_left = len;
    let mut thread_rng = thread_rng();
    while bytes_left > 0 {
        if !is_ascii {
            thread_rng.fill(&mut buf[..]);
        } else {
            let str = Alphanumeric.sample_string(&mut rand::thread_rng(), buf.len());
            buf.copy_from_slice(str.as_bytes());
        }
        let bytes_to_write = std::cmp::min(buf.len() as u64, bytes_left);
        file.write_all(&buf[0..bytes_to_write as usize])?;
        bytes_left -= bytes_to_write;
    }
    Ok(())
}

pub fn generate_random_file(file_path: &str, len: u64, is_ascii: bool) -> anyhow::Result<()> {
    match generate_random_file_int(file_path, len, is_ascii) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Error generating random file {}: {}", file_path, e);
            Err(e)
        }
    }
}