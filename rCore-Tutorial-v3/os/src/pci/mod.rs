mod net;

use core::arch::riscv64::wfi;

use virtio_drivers::{VirtIONet, VirtIOHeader};

use crate::{drivers::block::virtio_blk::VirtioHal, trap::hexdump, sync::UPSafeCell};

use self::net::handle_eth_receive;

lazy_static::lazy_static! {
    static ref NET_DEVICE:UPSafeCell<VirtIONet<'static, VirtioHal>> = unsafe {
        UPSafeCell::new(VirtIONet::<VirtioHal>::new(unsafe {
            &mut *(0x1000_8000 as *mut VirtIOHeader)
        }).expect("failed to create net driver"))
    };
}

pub fn init() {    

    loop {
        unsafe { wfi(); }
        let mut buf = [0u8; 0x100];

        let len = NET_DEVICE.exclusive_access().recv(&mut buf).expect("failed to recv");
        hexdump(&buf[..len]);

        if len > 0 {
            handle_eth_receive(&buf[..len]);
        }
    }
}