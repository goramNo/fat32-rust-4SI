use super::FsError;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct BpbCommon {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sector_count: u16,
    pub num_fats: u8,
    pub root_entry_count: u16,
    pub total_sectors_16: u16,
    pub media: u8,
    pub fat_size_16: u16,
    pub sectors_per_track: u16,
    pub num_heads: u16,
    pub hidden_sectors: u32,
    pub total_sectors_32: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct BpbFat32Ext {
    pub fat_size_32: u32,
    pub ext_flags: u16,
    pub fs_version: u16,
    pub root_cluster: u32,
    pub fs_info: u16,
    pub bk_boot_sector: u16,
    pub reserved: [u8; 12],
    pub drive_number: u8,
    pub reserved1: u8,
    pub boot_signature: u8,
    pub volume_id: u32,
    pub volume_label: [u8; 11],
    pub fs_type: [u8; 8],
}

pub struct BootSector {
    pub common: BpbCommon,
    pub fat32: BpbFat32Ext,
}

impl BootSector {    
    /// Parse un secteur de boot FAT32 (512 octets).
    ///
    /// # Safety
    /// Utilise des accès `read_unaligned` sur des données brutes.
    /// On suppose que le buffer fait au moins 512 octets.
    
    pub fn parse(sec: &[u8]) -> Result<Self, FsError> {
        if sec.len() < 512 { return Err(FsError::Parse); }
        if sec[510] != 0x55 || sec[511] != 0xAA { return Err(FsError::BadSig); }

        let common = unsafe {
            core::ptr::read_unaligned(sec[11..].as_ptr() as *const BpbCommon)
        };
        let fat32 = unsafe {
            core::ptr::read_unaligned(sec[36..].as_ptr() as *const BpbFat32Ext)
        };

        if common.bytes_per_sector as usize != 512 { return Err(FsError::Unsupported); }
        Ok(Self { common, fat32 })
    }

    #[inline]
    pub fn sectors_per_fat32(&self) -> u32 { self.fat32.fat_size_32 }
}
