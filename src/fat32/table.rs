use super::FsError;
use crate::device::BlockDevice;

pub const EOF: u32 = 0x0FFF_FFF8;

pub fn next_cluster<D: BlockDevice>(
    dev: &mut D,
    fat_lba: u32,
    cluster: u32,
    tmp: &mut [u8; 512],
) -> Result<u32, FsError> {
    let off = cluster * 4;
    let sec = fat_lba + (off / 512);
    let idx = (off % 512) as usize;
    dev.read_sector(sec, tmp).map_err(|_| FsError::Io)?;
    let val = u32::from_le_bytes(tmp[idx..idx + 4].try_into().unwrap()) & 0x0FFF_FFFF;
    Ok(val)
}
