use libc;
use log::error;
use std::fs;
use std::os::unix::fs::MetadataExt;

#[derive(Debug)]
pub struct BlockDeviceInfo {
    pub path: String,
    pub major: u32,
    pub minor: u32,
    pub size_sectors: u64,
}

pub fn block_dev_info(path: &String) -> Result<BlockDeviceInfo, std::io::Error> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(e) => {
            error!("Failed to get metadata for {}: {}", path, e);
            return Err(e);
        }
    };
    let dev_t: u32 = metadata.rdev() as u32;

    let major = unsafe { libc::major(dev_t as libc::dev_t) };
    let minor = unsafe { libc::minor(dev_t as libc::dev_t) };

    let size_path = format!("/sys/dev/block/{}:{}/size", major, minor);
    let size_str = match fs::read_to_string(&size_path) {
        Ok(size_str) => size_str,
        Err(e) => {
            error!(
                "Failed to read block dev size. Did you pass a block device? {}: {}",
                size_path, e
            );
            return Err(e);
        }
    };
    let size_sectors: u64 = size_str.trim_end().parse().unwrap();

    Ok(BlockDeviceInfo {
        path: path.clone(),
        major,
        minor,
        size_sectors,
    })
}
