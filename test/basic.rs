#![no_std]
#![no_main]

extern crate fat32_nostd;

use fat32_nostd::mock::MockBlockDevice;
use fat32_nostd::fat32::Fat32;

#[test]
fn test_mount_empty() {
    let mut dev = MockBlockDevice::new(4);

    // BPB minimal
    dev.sectors[0][510] = 0x55;
    dev.sectors[0][511] = 0xAA;

    let res = Fat32::mount(dev);
    assert!(res.is_ok());
}
