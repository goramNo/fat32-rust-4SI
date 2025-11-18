use crate::device::{BlockDevice, DeviceError};

pub struct MockBlockDevice<const N: usize> {
    pub sectors: [[u8; 512]; N],
}

impl<const N: usize> MockBlockDevice<N> {
    pub fn new() -> Self {
        Self {
            sectors: [[0u8; 512]; N],
        }
    }
}

impl<const N: usize> BlockDevice for MockBlockDevice<N> {
    fn read_sector(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), DeviceError> {
        let lba = lba as usize;
        if lba >= N { return Err(DeviceError::Io); }
        if buf.len() < 512 { return Err(DeviceError::ShortBuffer); }

        buf[..512].copy_from_slice(&self.sectors[lba]);
        Ok(())
    }
}
