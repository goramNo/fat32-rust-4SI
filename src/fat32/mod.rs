pub mod bpb;
pub mod table;
pub mod dirent;

use crate::device::BlockDevice;
use core::{cmp, mem, ptr};

#[derive(Debug, Clone, Copy)]
pub enum FsError {
    BadSig,
    Unsupported,
    Io,
    Parse,
}

pub struct Fat32<D: BlockDevice> {
    dev: D,
    pub bpb: bpb::BootSector,
    first_data_lba: u32,
    fat_lba: u32,
}

impl<D: BlockDevice> Fat32<D> {
    pub fn mount(mut dev: D) -> Result<Self, FsError> {
        let mut sec = [0u8; 512];
        dev.read_sector(0, &mut sec).map_err(|_| FsError::Io)?;
        let bpb = bpb::BootSector::parse(&sec)?;
        let fat_lba = bpb.common.reserved_sector_count as u32;
        let first_data_lba =
            fat_lba + (bpb.common.num_fats as u32 * bpb.sectors_per_fat32());
        Ok(Self {
            dev,
            bpb,
            first_data_lba,
            fat_lba,
        })
    }

    pub fn first_sector_of_cluster(&self, cl: u32) -> u32 {
        (cl - 2) * self.bpb.common.sectors_per_cluster as u32 + self.first_data_lba
    }

    pub fn next_cluster(&mut self, cl: u32, tmp: &mut [u8; 512]) -> Result<u32, FsError> {
        table::next_cluster(&mut self.dev, self.fat_lba, cl, tmp)
    }

    pub fn read_cluster(&mut self, cl: u32, buf: &mut [u8]) -> Result<(), FsError> {
        let spc = self.bpb.common.sectors_per_cluster as u32;
        let need = (spc as usize) * 512;
        if buf.len() < need {
            return Err(FsError::Parse);
        }
        let first = self.first_sector_of_cluster(cl);
        for i in 0..spc {
            let off = (i as usize) * 512;
            self.dev
                .read_sector(first + i, &mut buf[off..off + 512])
                .map_err(|_| FsError::Io)?;
        }
        Ok(())
    }

    pub fn read_dir_once(
        &mut self,
        cl: u32,
        buf: &mut [u8],
        out: &mut [dirent::ShortDirEntry],
    ) -> Result<usize, FsError> {
        self.read_cluster(cl, buf)?;

        let entry_size = mem::size_of::<dirent::ShortDirEntry>();
        let max_entries = cmp::min(out.len(), buf.len() / entry_size);

        let mut count = 0usize;
        let mut offset = 0usize;

        while count < max_entries && offset + entry_size <= buf.len() {
            let ptr_entry =
                unsafe { buf.as_ptr().add(offset) as *const dirent::ShortDirEntry };
            let e = unsafe { ptr::read_unaligned(ptr_entry) };

            if e.is_unused() {
                break;
            }
            if !e.is_deleted() && !e.is_lfn() {
                out[count] = e;
                count += 1;
            }

            offset += entry_size;
        }

        Ok(count)
    }
}
