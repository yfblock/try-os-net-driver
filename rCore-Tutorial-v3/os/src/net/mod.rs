pub mod syscall;
pub mod udp;
pub mod socket;

use core::arch::riscv64::wfi;

use lose_net_stack::{LoseStack, IPv4, MacAddress, results::Packet};
use virtio_drivers::{VirtIONet, VirtIOHeader};

use crate::{drivers::block::virtio_blk::VirtioHal, sync::UPSafeCell, net::{socket::{get_socket, push_data}, udp::hexdump}};

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

pub fn net_interrupt_handler() {
    let mut recv_buf = vec![0u8; 1024];

    let len = NET_DEVICE.exclusive_access().recv(&mut recv_buf).expect("can't receive from net dev");
        
    let packet = LOSE_NET_STACK.exclusive_access().analysis(&recv_buf[..len]);
    
    println!("[kernel] receive a packet");
    hexdump(&recv_buf[..len]);

    match packet {
        Packet::ARP(arp_packet) => {
            let lose_stack = LOSE_NET_STACK.exclusive_access();
            let reply_packet = arp_packet.reply_packet(lose_stack.ip, lose_stack.mac).expect("can't build reply");
            let reply_data = reply_packet.build_data();
            NET_DEVICE.exclusive_access().send(&reply_data).expect("can't send net data");
        },

        Packet::UDP(udp_packet) => {
            let target = udp_packet.source_ip;
            let lport = udp_packet.dest_port;
            let rport = udp_packet.source_port;
            
            if let Some(socket_index) = get_socket(target, lport, rport) {
                push_data(socket_index, udp_packet.data.to_vec());
            }
        }
        _ => {}
    }
}