use alloc::{boxed::Box, vec};
use lose_net_stack::{IPv4, packets::udp::UDPPacket, MacAddress, results::Packet};

use crate::{fs::File, mm::UserBuffer};

use super::{NET_DEVICE, LOSE_NET_STACK};

pub struct UDP{
    pub target: IPv4,
    pub sport: u16,
    pub dport: u16
}

impl UDP {
    pub fn new(target: IPv4, sport: u16, dport: u16) -> Self {
        Self {
            target,
            sport,
            dport
        }
    }
}

impl File for UDP {
    fn readable(&self) -> bool {
        true
    }

    fn writable(&self) -> bool {
        true
    }

    fn read(&self, mut buf: crate::mm::UserBuffer) -> usize {
        let mut recv_buf = vec![0u8; 1024];
        loop {
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
                    let mut left = 0;
                    for i in 0..buf.buffers.len() {
                        let buffer_i_len = buf.buffers[i].len().min(udp_packet.data_len - left);
                        
                        buf.buffers[i][..buffer_i_len].copy_from_slice(&udp_packet.data[left..(left + buffer_i_len)]);

                        left += buffer_i_len;
                        if left == udp_packet.data_len {
                            break;
                        }
                    }
                    return left;
                }
                _ => {}
            }
        }
    }

    fn write(&self, buf: crate::mm::UserBuffer) -> usize {
        let lose_net_stack = LOSE_NET_STACK.exclusive_access();

        let mut data = vec![0u8; buf.len()];
        
        let mut left = 0;
        for i in 0..buf.buffers.len() {
            data[left..(left + buf.buffers[i].len())].copy_from_slice(buf.buffers[i]);
            left += buf.buffers[i].len();
        }

        let len = data.len();

        let mut t = NET_DEVICE.exclusive_access();
        let udp_packet = UDPPacket::new(
            lose_net_stack.ip, 
            lose_net_stack.mac, 
            self.sport, 
            self.target, 
            MacAddress::new([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]), 
            self.dport, 
            len, 
            data.as_ref()
        );
        t.send(&udp_packet.build_data()).expect("can't send to net device");
        len
    }
}

pub fn hexdump(data: &[u8]) {
    const PRELAND_WIDTH: usize = 70;
    println!("[kernel] {:-^1$}", " hexdump ", PRELAND_WIDTH);
    for offset in (0..data.len()).step_by(16) {
        print!("[kernel] ");
        for i in 0..16 {
            if offset + i < data.len() {
                print!("{:02x} ", data[offset + i]);
            } else {
                print!("{:02} ", "");
            }
        }

        print!("{:>6}", ' ');

        for i in 0..16 {
            if offset + i < data.len() {
                let c = data[offset + i];
                if c >= 0x20 && c <= 0x7e {
                    print!("{}", c as char);
                } else {
                    print!(".");
                }
            } else {
                print!("{:02} ", "");
            }
        }
        
        println!("");
    }
    println!("[kernel] {:-^1$}", " hexdump end ", PRELAND_WIDTH);
}
