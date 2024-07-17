use anyhow::bail;
use rand::distributions::{Alphanumeric, DistString};
use rand::{thread_rng, Rng};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

fn truncate_file_int(file_path: &str, target_size: u64) -> anyhow::Result<()> {
    if target_size == 0 {
        std::fs::remove_file(file_path)?;
        Ok(())
    } else {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(false)
            .open(file_path)?;

        let file_size = file.seek(SeekFrom::End(0))?;

        if file_size < target_size {
            bail!(
                "File size is already smaller than target size {} vs {}",
                file_size,
                target_size
            );
        }
        if file_size == target_size {
            log::debug!("File size is already equal to target size {}", file_size);
            return Ok(());
        }
        log::debug!(
            "Truncating file {} to target size {}",
            file_path,
            target_size
        );

        //2 seek to target size
        file.seek(SeekFrom::Start(target_size))?;

        //3 truncate file
        file.set_len(target_size)?;

        Ok(())
    }
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

fn ranges_overlap(src: (u64, u64), dst: (u64, u64)) -> bool {
    !(src.1 <= dst.0 || src.0 >= dst.1)
}

#[test]
fn test_ranges_overlap_no_overlap() {
    assert!(!ranges_overlap((10, 20), (20, 30)));
    assert!(!ranges_overlap((30, 40), (10, 20)));
    assert!(!ranges_overlap((0, 10), (10, 20)));
    assert!(!ranges_overlap((10, 20), (0, 10)));
    assert!(!ranges_overlap((10, 20), (20, 30)));
    assert!(!ranges_overlap((20, 30), (10, 20)));
    assert!(ranges_overlap((10, 20), (15, 25)));
    assert!(ranges_overlap((15, 25), (10, 20)));
    assert!(ranges_overlap((10, 20), (10, 20)));
    assert!(ranges_overlap((10, 30), (15, 25)));
    assert!(ranges_overlap((15, 25), (10, 30)));
    assert!(ranges_overlap((10, 30), (15, 20)));
    assert!(ranges_overlap((15, 20), (10, 30)));
    assert!(!ranges_overlap((0, 1), (1, 2)));
    assert!(!ranges_overlap((1, 2), (0, 1)));
}

fn copy_chunk_int(file_path: &str, src: (u64, u64), dst: (u64, u64)) -> anyhow::Result<()> {
    if src.1 <= src.0 {
        bail!("Source range is invalid {}-{}", src.0, src.1);
    }
    if dst.1 <= dst.0 {
        bail!("Destination range is invalid {}-{}", dst.0, dst.1);
    }
    if src.1 - src.0 != dst.1 - dst.0 {
        bail!(
            "Source and destination ranges are not the same size {}-{} {}-{}",
            src.0,
            src.1,
            dst.0,
            dst.1
        );
    }

    //check if chunks are overlapping
    if ranges_overlap(src, dst) {
        bail!(
            "Source and destination ranges overlap {}-{} {}-{}",
            src.0,
            src.1,
            dst.0,
            dst.1
        );
    }
    let file_size = std::fs::metadata(file_path)?.len();
    if src.1 > file_size {
        bail!(
            "Source range is out of bounds {}-{} file size {}",
            src.0,
            src.1,
            file_size
        );
    }
    if dst.1 > file_size {
        bail!(
            "Destination range is out of bounds {}-{} file size {}",
            dst.0,
            dst.1,
            file_size
        );
    }

    //open file for read write
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(false)
        .open(file_path)?;

    file.seek(SeekFrom::Start(src.0))?;
    //seek to source start

    //read buffer
    let mut bytes_left = src.1 - src.0;
    let mut buffer = vec![0u8; std::cmp::min(1000 * 1000, bytes_left as usize)];
    while bytes_left > 0 {
        let bytes_read = std::cmp::min(buffer.len() as u64, bytes_left);
        buffer.resize(bytes_read as usize, 0);
        file.read_exact(buffer.as_mut_slice())?;
        bytes_left -= bytes_read;

        //seek to destination start
        file.seek(SeekFrom::Start(dst.0))?;
        file.write_all(&buffer)?;
    }

    Ok(())
}

pub fn copy_chunk(file_path: &str, src: (u64, u64), dst: (u64, u64)) -> anyhow::Result<()> {
    match copy_chunk_int(file_path, src, dst) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Error copying chunk {}: {}", file_path, e);
            Err(e)
        }
    }
}

pub fn output_chunk_int(file_path: &str, data: (u64, u64)) -> anyhow::Result<()> {
    if data.1 <= data.0 {
        bail!("Data range is invalid {}-{}", data.0, data.1);
    }
    let file_size = std::fs::metadata(file_path)?.len();
    if data.1 > file_size {
        bail!(
            "Data range is out of bounds {}-{} file size {}",
            data.0,
            data.1,
            file_size
        );
    }

    //open file for read write
    let mut file = OpenOptions::new()
        .read(true)
        .truncate(false)
        .open(file_path)?;

    file.seek(SeekFrom::Start(data.0))?;
    //read buffer
    let mut bytes_left = data.1 - data.0;
    let mut buffer = vec![0u8; std::cmp::min(1000 * 1000, bytes_left as usize)];
    while bytes_left > 0 {
        let bytes_read = std::cmp::min(buffer.len() as u64, bytes_left);
        buffer.resize(bytes_read as usize, 0);
        file.read_exact(buffer.as_mut_slice())?;
        bytes_left -= bytes_read;
        std::io::stdout().write_all(&buffer)?;
    }
    Ok(())
}
pub fn output_chunk(file_path: &str, data: (u64, u64)) -> anyhow::Result<()> {
    match output_chunk_int(file_path, data) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Error outputting chunk {}: {}", file_path, e);
            Err(e)
        }
    }
}
