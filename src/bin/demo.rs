use fat32_nostd::fat32::{Fat32, dirent};
use fat32_nostd::mock::MockBlockDevice;
use fat32_nostd::fat32::table;

fn main() {
    let mut dev = MockBlockDevice::<16>::new();

    // BPB
    {
        let s = &mut dev.sectors[0];
        s[11..13].copy_from_slice(&512u16.to_le_bytes());
        s[13] = 1;
        s[14..16].copy_from_slice(&1u16.to_le_bytes());
        s[16] = 1;
        s[36..40].copy_from_slice(&1u32.to_le_bytes());
        s[44..48].copy_from_slice(&2u32.to_le_bytes());
        s[510] = 0x55;
        s[511] = 0xAA;
    }

    // FAT : 2 -> EOF, 3 -> EOF
    {
        let s = &mut dev.sectors[1];
        let off2 = 2 * 4;
        s[off2..off2+4].copy_from_slice(&table::EOF.to_le_bytes());
        let off3 = 3 * 4;
        s[off3..off3+4].copy_from_slice(&table::EOF.to_le_bytes());
    }

    // Root dir dans cluster 2 -> secteur 2
    {
        let s = &mut dev.sectors[2];

        let mut e = dirent::ShortDirEntry::default();
        e.name = *b"FILE    TXT";
        e.attr = 0x20;
        e.fst_clus_lo = 3;
        let data = b"Hello FAT32";
        e.file_size = data.len() as u32;

        let bytes: [u8; core::mem::size_of::<dirent::ShortDirEntry>()] =
            unsafe { core::mem::transmute(e) };
        s[0..bytes.len()].copy_from_slice(&bytes);

        s[32] = 0x00;
    }

    // Data du fichier dans cluster 3 -> secteur 3
    {
        let s = &mut dev.sectors[3];
        let data = b"Hello FAT32";
        s[..data.len()].copy_from_slice(data);
    }

    let mut fs = Fat32::mount(dev).expect("mount ok");

    let mut buf_cluster = [0u8; 512];
    let mut entries = [dirent::ShortDirEntry::default(); 16];

    let n = fs.read_root_dir(&mut buf_cluster, &mut entries).expect("read_root");
    println!("Nb d'entrées: {}", n);

    let e0 = &entries[0];
    let (name, ext) = e0.short_name();
    let name = String::from_utf8_lossy(name);
    let ext  = String::from_utf8_lossy(ext);
    println!("ENTRY: {}.{}", name, ext);

    let mut file_buf = [0u8; 64];
    let read = fs.read_file(e0, &mut file_buf).expect("read_file");
    let content = &file_buf[..read];
    let s = String::from_utf8_lossy(content);
    println!("CONTENU: {}", s);
}
