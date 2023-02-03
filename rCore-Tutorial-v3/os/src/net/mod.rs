pub mod syscall;
pub mod udp;

use core::arch::riscv64::wfi;

use lose_net_stack::{LoseStack, IPv4, MacAddress};
use virtio_drivers::{VirtIONet, VirtIOHeader};

use crate::{drivers::block::virtio_blk::VirtioHal, trap::hexdump, sync::UPSafeCell};

lazy_static::lazy_static! {
    static ref NET_DEVICE:UPSafeCell<VirtIONet<'static, VirtioHal>> = unsafe {
        UPSafeCell::new(VirtIONet::<VirtioHal>::new(
            &mut *(0x1000_8000 as *mut VirtIOHeader)
        ).expect("failed to create net driver"))
    };

    static ref LOSE_NET_STACK: UPSafeCell<LoseStack> = unsafe {
        UPSafeCell::new(LoseStack::new(
            IPv4::new(10, 0, 2, 15),
            MacAddress::new([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]) 
        ))
    };
}


// net related function
pub const SYS_SOCKET: usize = 41;
pub const SYS_CONNECT: usize = 29;

pub fn init() {

}