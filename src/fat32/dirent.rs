// noms courts + flags
pub const ATTR_LFN: u8 = 0x0F;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct ShortDirEntry {
    pub name: [u8; 11],
    pub attr: u8,
    pub nt_res: u8,
    pub crt_time_tenth: u8,
    pub crt_time: u16,
    pub crt_date: u16,
    pub lst_acc_date: u16,
    pub fst_clus_hi: u16,
    pub wrt_time: u16,
    pub wrt_date: u16,
    pub fst_clus_lo: u16,
    pub file_size: u32,
}

impl ShortDirEntry {
    #[inline] pub fn is_unused(&self)  -> bool { self.name[0] == 0x00 }
    #[inline] pub fn is_deleted(&self) -> bool { self.name[0] == 0xE5 }
    #[inline] pub fn is_lfn(&self)     -> bool { self.attr == ATTR_LFN }
    #[inline] pub fn first_cluster(&self) -> u32 {
        ((self.fst_clus_hi as u32) << 16) | self.fst_clus_lo as u32
    }
    // nom court "NAME.EXT" sans espaces
    pub fn short_name<'a>(&'a self) -> (&'a [u8], &'a [u8]) {
        let name = trim_space(&self.name[0..8]);
        let ext  = trim_space(&self.name[8..11]);
        (name, ext)
    }
}

// util mini sans alloc
#[inline]
fn trim_space(slice: &[u8]) -> &[u8] {
    let mut end = slice.len();
    while end > 0 && slice[end-1] == b' ' { end -= 1; }
    &slice[..end]
}
