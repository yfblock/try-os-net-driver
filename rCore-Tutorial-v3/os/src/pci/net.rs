use core::mem::size_of;

use alloc::vec;

use crate::trap::hexdump;

use super::NET_DEVICE;

#[derive(Debug)]
#[repr(C)]
pub struct eth {
    dhost: [u8; 6], // destination host
    shost: [u8; 6], // source host
    rtype: u16      // packet type, arp or ip
}

const ETH_RTYPE_IP: u16 =  0x0800; // Internet protocol
const ETH_RTYPE_ARP: u16 = 0x0806; // Address resolution protocol

const LOCAL_MAC: [u8; 6] = [0x52, 0x54, 0x00, 0x12, 0x34, 0x56];
const BROADCAST_MAC: [u8; 6] = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
const LOCAL_IP: u32 = ip(10, 0, 2, 15);

#[inline]
pub const fn ip(a1: u8, a2: u8, a3: u8, a4: u8) -> u32 {
    (a1 as u32) << 24 | (a2 as u32) << 16 | (a3 as u32) << 8 | (a4 as u32)
}



pub fn handle_eth_receive(data: &[u8]) {
    let eth_header = unsafe{(data.as_ptr() as usize as *const eth).as_ref()}.unwrap();
    println!("eth header: {:x?}", eth_header);
    
    let rtype = eth_header.rtype.to_be();
    println!("type: {:#04X}", rtype);

    match rtype {
        ETH_RTYPE_IP => {
            println!("handle ip packet");
        }
        ETH_RTYPE_ARP => {
            println!("handle ARP packet");
            handle_arp_receive(&data[size_of::<eth>()..]);
        }
        _ => {}
    }
}

pub fn eth_transmite(send_data: &mut [u8], rtype: u16) {
    let mut data = vec![0u8; size_of::<eth>()];
    data.extend(send_data.iter());
    let mut eth_header = unsafe{(data.as_ptr() as usize as *mut eth).as_mut()}.unwrap();
    eth_header.shost = LOCAL_MAC;
    eth_header.dhost = BROADCAST_MAC;
    eth_header.rtype = rtype.to_be();
    hexdump(&data);

    println!("send done");

    NET_DEVICE.exclusive_access().send(&data).expect("failed to send");
}

// ARP Packet
// refer: https://en.wikipedia.org/wiki/Address_Resolution_Protocol

const ARP_HRD_ETHER: u16 = 1;
const ARP_PTYPE_ETHTYPE_IP: u16 = 0x0800;
const ARP_ETHADDR_LEN: usize = 6;
const ARP_OP_REPLY: u16 = 2;


#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct arp {
    httype: u16, // Hardware type
    pttype: u16, // Protocol type, For IPv4, this has the value 0x0800.
    hlen: u8,    // Hardware length: Ethernet address length is 6.
    plen: u8,    // Protocol length: IPv4 address length is 4.
    op: u16,     // Operation: 1 for request, 2 for reply.
    sha: [u8; 6],// Sender hardware address
    spa: u32,    // Sender protocol address
    tha: [u8; 6],// Target hardware address
    tpa: u32     // Target protocol address
}

pub fn handle_arp_receive(data: &[u8]) {
    hexdump(data);
    let arp_header = unsafe{(data.as_ptr() as usize as *const arp).as_ref()}.unwrap();
    println!("arp header: {:#x?}", arp_header);

    if arp_header.plen == 4 {
        println!("arp protocol: ipv4");
    }

    let op = arp_header.op.to_be();

    if op == 1 {
        println!("arp request");
    } else if op == 2 {
        println!("arp reply");
    }

    println!("sender hardware address: {:?}", arp_header.sha);
    println!("sender protocol address: {:#x}", arp_header.spa.to_be());
    println!("target hardware address: {:?}", arp_header.tha);
    println!("target protocol address: {:#x}", arp_header.tpa.to_be());
    
    arp_tramsmit(2, &arp_header.sha, arp_header.spa.to_be())
    // let rtype = eth_header.rtype.to_be();
    // println!("type: {:#04X}", rtype);
}


pub fn arp_tramsmit(op: u16, dmac: &[u8; 6], dip: u32) {
    let mut data = vec![0u8; size_of::<arp>()];

    let mut arp_header = unsafe{(data.as_ptr() as usize as *mut arp).as_mut()}.unwrap();
    arp_header.httype = ARP_HRD_ETHER.to_be();
    arp_header.pttype = ETH_RTYPE_IP.to_be();
    arp_header.hlen = ARP_ETHADDR_LEN as u8;
    arp_header.plen = 4;    // ipv4
    arp_header.op = op.to_be();
    
    arp_header.sha = LOCAL_MAC;
    arp_header.spa = LOCAL_IP.to_be();

    arp_header.tha = dmac.clone();
    arp_header.tpa = dip.to_be();

    eth_transmite(&mut data, ETH_RTYPE_ARP);
}

/*
arp request and reply data
------------------------------ hexdump -------------------------------
ff ff ff ff ff ff 52 55 0a 00 02 02 08 06 00 01       ......RU........
08 00 06 04 00 01 52 55 0a 00 02 02 0a 00 02 02       ......RU........
00 00 00 00 00 00 0a 00 02 0f                         ..........                  
---------------------------- hexdump end -----------------------------

------------------------------ hexdump -------------------------------
ff ff ff ff ff ff 52 54 00 12 34 56 08 06 00 01       ......RT..4V....
08 00 06 04 00 02 52 54 00 12 34 56 0f 02 00 0a       ......RT..4V....
52 55 0a 00 02 02 0a 00 02 02                         RU........                  
---------------------------- hexdump end -----------------------------




*/