//!An easy file system isolated from the kernel
#![no_std]
#![deny(missing_docs)]
extern crate alloc;
mod block_dev;
mod efs;
mod vfs;
/// Use a block size of 512 bytes
pub const BLOCK_SZ: usize = 512;
pub use block_dev::BlockDevice;
pub use efs::EasyFileSystem;
pub use vfs::Inode;
