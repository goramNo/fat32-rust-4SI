use fat32_nostd::fat32::{Fat32, dirent};
use fat32_nostd::mock::MockBlockDevice;
use fat32_nostd::fat32::table;

fn main() {
    // disque de 8 secteurs
    let mut dev = MockBlockDevice::<8>::new();

    // BPB dans secteur 0
    {
        let s = &mut dev.sectors[0];
        // 512 bytes/sector
        s[11..13].copy_from_slice(&512u16.to_le_bytes());
        // 1 sector/cluster
        s[13] = 1;
        // reserved sectors = 1 (FAT à LBA 1)
        s[14..16].copy_from_slice(&1u16.to_le_bytes());
        // 1 FAT
        s[16] = 1;
        // FAT size (sectors)
        s[36..40].copy_from_slice(&1u32.to_le_bytes());
        // root cluster = 2
        s[44..48].copy_from_slice(&2u32.to_le_bytes());
        // signature 0x55AA
        s[510] = 0x55;
        s[511] = 0xAA;
    }

    // FAT dans secteur 1 : cluster 2 -> EOF
    {
        let s = &mut dev.sectors[1];
        let off = 2 * 4; // entrée n°2
        s[off..off+4].copy_from_slice(&table::EOF.to_le_bytes());
    }

    // Root dir dans cluster 2 -> secteur 2
    {
        let s = &mut dev.sectors[2];

        // entrée 0 : fichier "FILE    TXT"
        let mut e = dirent::ShortDirEntry::default();
        e.name = *b"FILE    TXT";
        e.attr = 0x20; // fichier
        e.fst_clus_lo = 3; // cluster de début (peu importe ici)
        e.file_size = 1234;

        let bytes: [u8; core::mem::size_of::<dirent::ShortDirEntry>()] =
            unsafe { core::mem::transmute(e) };
        s[0..bytes.len()].copy_from_slice(&bytes);

        // entrée 1 : fin de dir (0x00)
        s[32] = 0x00;
    }

    // Monte le FS
    let mut fs = Fat32::mount(dev).expect("mount ok");

    let mut buf = [0u8; 512];
    let mut entries = [dirent::ShortDirEntry::default(); 16];

    let n = fs.read_root_dir(&mut buf, &mut entries).expect("read_root");

    println!("Nb d'entrées: {}", n);
    for e in entries.iter().take(n) {
        let (name, ext) = e.short_name();
        let name = String::from_utf8_lossy(name);
        let ext  = String::from_utf8_lossy(ext);
        if ext.is_empty() {
            println!("FILE: {}", name);
        } else {
            println!("FILE: {}.{}", name, ext);
        }
    }
}
