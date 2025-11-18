use crate::device::{BlockDevice, DeviceError};

pub struct MockBlockDevice {
    pub sectors: Vec<[u8; 512]>,
}

impl MockBlockDevice {
    pub fn new(nb: usize) -> Self {
        Self {
            sectors: vec![[0u8; 512]; nb],
        }
    }
}

impl BlockDevice for MockBlockDevice {
    fn read_sector(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), DeviceError> {
        let lba = lba as usize;
        if lba >= self.sectors.len() { return Err(DeviceError::Io); }
        if buf.len() < 512 { return Err(DeviceError::ShortBuffer); }
        buf[..512].copy_from_slice(&self.sectors[lba]);
        Ok(())
    }
}
