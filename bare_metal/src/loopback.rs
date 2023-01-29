#![no_std]
#![allow(unused_mut)]
#![allow(clippy::collapsible_if)]

use core::str;
use log::{debug, error, info};

use smoltcp::iface::{Config, Interface, SocketSet};
use smoltcp::phy::{Loopback, Medium};
use smoltcp::socket::tcp;
use smoltcp::time::{Duration, Instant};
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr};

mod mock {
    use core::cell::Cell;
    use smoltcp::time::{Duration, Instant};

    #[derive(Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct Clock(Cell<Instant>);

    impl Clock {
        pub fn new() -> Clock {
            Clock(Cell::new(Instant::from_millis(0)))
        }

        pub fn advance(&self, duration: Duration) {
            self.0.set(self.0.get() + duration)
        }

        pub fn elapsed(&self) -> Instant {
            self.0.get()
        }
    }
}

pub fn net_main() {
    let clock = mock::Clock::new();
    let mut device = Loopback::new(Medium::Ethernet);

    // Create interface
    let mut config = Config::new();
    config.hardware_addr = Some(EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]).into());

    let mut iface = Interface::new(config, &mut device);
    iface.update_ip_addrs(|ip_addrs| {
        ip_addrs
            .push(IpCidr::new(IpAddress::v4(127, 0, 0, 1), 8))
            .unwrap();
    });

    // Create sockets
    let server_socket = {
        // It is not strictly necessary to use a `static mut` and unsafe code here, but
        // on embedded systems that smoltcp targets it is far better to allocate the data
        // statically to verify that it fits into RAM rather than get undefined behavior
        // when stack overflows.
        static mut TCP_SERVER_RX_DATA: [u8; 1024] = [0; 1024];
        static mut TCP_SERVER_TX_DATA: [u8; 1024] = [0; 1024];
        let tcp_rx_buffer = tcp::SocketBuffer::new(unsafe { &mut TCP_SERVER_RX_DATA[..] });
        let tcp_tx_buffer = tcp::SocketBuffer::new(unsafe { &mut TCP_SERVER_TX_DATA[..] });
        tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer)
    };

    let client_socket = {
        static mut TCP_CLIENT_RX_DATA: [u8; 1024] = [0; 1024];
        static mut TCP_CLIENT_TX_DATA: [u8; 1024] = [0; 1024];
        let tcp_rx_buffer = tcp::SocketBuffer::new(unsafe { &mut TCP_CLIENT_RX_DATA[..] });
        let tcp_tx_buffer = tcp::SocketBuffer::new(unsafe { &mut TCP_CLIENT_TX_DATA[..] });
        tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer)
    };

    let mut sockets: [_; 2] = Default::default();
    let mut sockets = SocketSet::new(&mut sockets[..]);
    let server_handle = sockets.add(server_socket);
    let client_handle = sockets.add(client_socket);

    let mut did_listen = false;
    let mut did_connect = false;
    let mut done = false;
    while !done && clock.elapsed() < Instant::from_millis(10_000) {
        iface.poll(clock.elapsed(), &mut device, &mut sockets);

        let mut socket = sockets.get_mut::<tcp::Socket>(server_handle);
        if !socket.is_active() && !socket.is_listening() {
            if !did_listen {
                debug!("listening");
                socket.listen(1234).unwrap();
                did_listen = true;
            }
        }

        if socket.can_recv() {
            debug!(
                "got {:?}",
                socket.recv(|buffer| { (buffer.len(), str::from_utf8(buffer).unwrap()) })
            );
            socket.close();
            done = true;
        }

        let mut socket = sockets.get_mut::<tcp::Socket>(client_handle);
        let cx = iface.context();
        if !socket.is_open() {
            if !did_connect {
                debug!("connecting");
                socket
                    .connect(cx, (IpAddress::v4(127, 0, 0, 1), 1234), 65000)
                    .unwrap();
                did_connect = true;
            }
        }

        if socket.can_send() {
            debug!("sending");
            socket.send_slice(b"0123456789abcdef").unwrap();
            socket.close();
        }

        match iface.poll_delay(clock.elapsed(), &sockets) {
            Some(Duration::ZERO) => debug!("resuming"),
            Some(delay) => {
                debug!("sleeping for {} ms", delay);
                clock.advance(delay)
            }
            None => clock.advance(Duration::from_millis(1)),
        }
    }

    if done {
        info!("done")
    } else {
        error!("this is taking too long, bailing out")
    }
}
