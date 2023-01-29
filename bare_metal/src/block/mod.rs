mod virtio;
mod nvme;

use alloc::boxed::Box;
use spin::Once;

// use virtio as block driver
// pub use virtio::init;

// use nvme as block driver
// pub use nvme::init;

use crate::{mutex::Mutex, sbi::shutdown, utils::memory::hexdump};

/// 定义trait
pub trait BlockDevice {
    // 读取扇区
    fn read_block(&mut self, sector_offset: usize, buf: &mut [u8]);
    // 写入扇区
    fn write_block(&mut self, sector_offset: usize, buf: &[u8]);
    // 处理中断
    fn handle_irq(&mut self);
}

pub static mut DEVICE: Once<Mutex<Box<dyn BlockDevice>>> = Once::new();

// pub static mut DEVICE1: Once<Mutex<Box<dyn BlockDevice>>> = Once::new();

pub fn init() {
    // virtio::init();
    nvme::init();

    // println!("start for compare");

    let mut arr = [0u8; 512];
    // let mut arr1 = [0u8; 512];
    let mut device = unsafe{DEVICE.get()}.unwrap().lock();

    device.read_block(0, &mut arr);

    hexdump(&arr);
    // let mut device1 = unsafe{DEVICE1.get()}.unwrap().lock();

    // for i in (0..8192).step_by(2) {
    //     device.read_block(i, &mut arr);
    //     device1.read_block(i, &mut arr1);

    //     assert_eq!(arr, arr1);
    // }
}
