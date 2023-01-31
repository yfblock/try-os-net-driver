use core::num::Wrapping;
use core::ptr;
use core::convert::{ Into, TryInto };
use core::fmt::{self, Write, Error};
use core::sync::atomic::Ordering;

const UART0: usize = 0x1000_0000;

/// receive holding register (for input bytes)
const RHR: usize = 0;
/// transmit holding register (for output bytes)
const THR: usize = 0;
/// interrupt enable register
const IER: usize = 1;
/// FIFO control register
const FCR: usize = 2;
/// interrupt status register 
const ISR: usize = 2; 
/// line control register
const LCR: usize = 3;
/// line status register 
const LSR: usize = 5; 

const IER_RX_ENABLE: usize = 1 << 0;
const IER_TX_ENABLE: usize = 1 << 1;
const FCR_FIFO_ENABLE: usize = 1 << 0;
const FCR_FIFO_CLEAR: usize = 3 << 1; // clear the content of the two FIFOs
const LCR_EIGHT_BITS: usize = 3 << 0;
const LCR_BAUD_LATCH: usize = 1 << 7; // special mode to set baud rate
const LSR_RX_READY: usize = 1 << 0; // input is waiting to be read from RHR
const LSR_TX_IDLE: usize = 1 << 5; // THR can accept another character to send

const UART_BASE_ADDR: usize = UART0;

const UART_BUF_SIZE:usize = 32;

pub fn init() {
    // // disable interrupts
    // write_reg(UART_BASE_ADDR + IER, 0x00);

    // // special mode to set baud rate. 
    // write_reg(UART_BASE_ADDR + LCR, LCR_BAUD_LATCH as u8);

    // // LSB for baud rate of 38.4K
    // write_reg(UART_BASE_ADDR, 0x03);

    // // MSB for baud rate of 38.4k 
    // write_reg(UART_BASE_ADDR + 1, 0x00);

    // // leave set-baud mode, 
    // // and set word length to 8 bits, no parity. 
    // write_reg(UART_BASE_ADDR + LCR, LCR_EIGHT_BITS as u8);

    // // reset and enable FIFOs. 
    // write_reg(UART_BASE_ADDR + FCR, FCR_FIFO_ENABLE as u8 | FCR_FIFO_CLEAR as u8);

    // enable transmit and receive interrupts. 
    write_reg(UART_BASE_ADDR + IER, IER_TX_ENABLE as u8 | IER_RX_ENABLE as u8);
}


fn write_reg(addr: usize, val: u8) {
    unsafe{
        ptr::write(addr as *mut u8, val);
    }
}

fn read_reg(addr: usize) -> u8 {
    unsafe {
        ptr::read(addr as *const u8)
    }
}
