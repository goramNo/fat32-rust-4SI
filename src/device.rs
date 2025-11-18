pub trait BlockDevice {
    fn read_sector(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), DeviceError>;
}

#[derive(Debug, Clone, Copy)]
pub enum DeviceError {
    Io,
    ShortBuffer,
}
