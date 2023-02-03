#![no_std]
#![no_main]

use alloc::string::String;
use user_lib::{connect, write, read};

#[macro_use]
extern crate user_lib;
#[macro_use]
extern crate alloc;

#[no_mangle]
pub fn main() -> i32 {
    println!("udp test open!");
    
    let udp_fd = connect((10 << 24 | 0 << 16 | 2 << 8 | 2), 2000, 26099);

    if udp_fd < 0 {
        println!("failed to create udp connection.");
        return -1;
    }

    // let buf = "Hello rCoreOS user program!";

    // write(user_fd, buf);

    let mut buf = vec![0u8; 1024];

    let len = read(udp_fd as usize, &mut buf);

    if len < 0 {
        println!("can't receive udp packet");
        return -1;
    }

    let recv_str = String::from_utf8_lossy(&buf[..len as usize]);

    println!("{}", recv_str);

    0
}
