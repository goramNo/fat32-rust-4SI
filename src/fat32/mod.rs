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
    /// Lit un cluster de répertoire et récupère les entrées courtes valides.
    ///
    /// # Safety
    /// Lit des structures packées depuis un buffer de 512*n octets avec
    /// `read_unaligned`. On suppose que le buffer vient bien du device FAT32.

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

    pub fn read_dir_chain(
        &mut self,
        start_cl: u32,
        buf: &mut [u8],
        out: &mut [dirent::ShortDirEntry],
    ) -> Result<usize, FsError> {
        let mut total = 0usize;
        let mut cl = start_cl;
        let mut fat_buf = [0u8; 512];

        loop {
            let got = self.read_dir_once(cl, buf, &mut out[total..])?;
            total += got;
            if total >= out.len() {
                break;
            }

            let next = self.next_cluster(cl, &mut fat_buf)?;
            if next >= table::EOF || next < 2 {
                break;
            }
            cl = next;
        }

        Ok(total)
    }

    pub fn read_root_dir(
        &mut self,
        buf: &mut [u8],
        out: &mut [dirent::ShortDirEntry],
    ) -> Result<usize, FsError> {
        let root = self.bpb.fat32.root_cluster;
        self.read_dir_chain(root, buf, out)
    }

    pub fn read_file(
        &mut self,
        entry: &dirent::ShortDirEntry,
        buf: &mut [u8],
    ) -> Result<usize, FsError> {
        let mut cl = entry.first_cluster();
        let mut fat_buf = [0u8; 512];
        let mut off = 0usize;
        let size = entry.file_size as usize;
        let spc = self.bpb.common.sectors_per_cluster as u32;
        let bytes_per_cluster = (spc as usize) * 512;

        while cl >= 2 && cl < table::EOF && off < buf.len() && off < size {
            let mut tmp = [0u8; 4096];
            let need = core::cmp::min(bytes_per_cluster, tmp.len());
            self.read_cluster(cl, &mut tmp[..need])?;

            let remain_file = size - off;
            let remain_buf = buf.len() - off;
            let to_copy = core::cmp::min(
                remain_file,
                core::cmp::min(remain_buf, bytes_per_cluster),
            );

            buf[off..off + to_copy].copy_from_slice(&tmp[..to_copy]);
            off += to_copy;

            if off >= size || off >= buf.len() {
                break;
            }

            let next = self.next_cluster(cl, &mut fat_buf)?;
            if next >= table::EOF || next < 2 {
                break;
            }
            cl = next;
        }

        Ok(off)
    }
}
