#![no_std]

pub mod device;
pub mod fat32;
pub mod mock;
pub mod allocator;

pub fn version() -> &'static str {
    "0.0.1"
}
