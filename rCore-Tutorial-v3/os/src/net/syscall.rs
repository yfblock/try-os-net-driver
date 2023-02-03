use alloc::sync::Arc;
use lose_net_stack::IPv4;

use crate::task::{current_user_token, current_task};

use super::udp::UDP;


// syscall connect with target addr、source port and target port. 
// return socket fd allocated.
pub fn sys_connect(raddr: u32, lport: u16, rport: u16) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    let fd = inner.alloc_fd();
    let udp_node = UDP::new(IPv4::from_u32(raddr), lport, rport);
    inner.fd_table[fd] = Some(Arc::new(udp_node));
    println!("SYS_CONNECT syscall");
    fd as isize    
}