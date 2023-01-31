use core::{ptr::{self, write_volatile}, arch::{asm, riscv64::fence_i}};

use riscv::register;


const PLIC_BASE: usize = 0x0c00_0000;
const UART0_IRQ: u32 = 10;
const VIRTIO0_IRQ: u32 = 1;


const PLIC_PRIORITY: usize = PLIC_BASE;
const PLIC_PENDING: usize = PLIC_BASE + 0x1000;

fn PLIC_MENABLE(hart_id: usize) -> usize {
    PLIC_BASE + 0x2000 + hart_id * 0x100
}

fn PLIC_SENABLE(hart_id: usize) -> usize {
    PLIC_BASE + 0x2080 + hart_id * 0x100
}

fn PLIC_MPRIORITY(hart_id: usize) -> usize {
    PLIC_BASE + 0x200000 + hart_id * 0x2000
}

fn PLIC_SPRIORITY(hart_id: usize) -> usize {
    PLIC_BASE + 0x201000 + hart_id * 0x2000
}

fn PLIC_MCLAIM(hart_id: usize) -> usize {
    PLIC_BASE + 0x200004 + hart_id * 0x2000
}

fn PLIC_SCLAIM(hart_id: usize) -> usize {
    PLIC_BASE + 0x201004 + hart_id * 0x2000
}

pub fn init() {
    unsafe {
        register::sie::set_sext();
        // write_volatile(0x1000_0001 as *mut u8, 3);
    }
    
    // set desired IRQ priorities non-zero (otherwise disable)
    write(PLIC_BASE + (UART0_IRQ * 4) as usize, 1);

    // 24 号中断
    write(PLIC_BASE + (24 * 4) as usize, 1);

    for i in 1..0x35 {
        write(PLIC_BASE + (i * 4) as usize, 1);
        unsafe { fence_i(); }
    }
    plic_init_hart();
}

pub fn plic_init_hart() {
    let hart_id = unsafe{ cpuid() } as usize;

    // Set UART's enable bit for this hart's S-mode. 
    write(PLIC_SENABLE(hart_id), 1 << UART0_IRQ);

    unsafe { fence_i(); }

    write(PLIC_SENABLE(hart_id), 1 << 24 );
    unsafe { fence_i(); }

    write(PLIC_SENABLE(hart_id) + 4, 0xffffffff);
    unsafe { fence_i(); }

    // Set this hart's S-mode pirority threshold to 0. 
    write(PLIC_SPRIORITY(hart_id), 0);
}

/// Ask the PLIC what interrupt we should serve. 
pub fn plic_claim() -> Option<u32> {
    let hart_id = unsafe {
        cpuid()
    } as usize;
    let interrupt = read(PLIC_SCLAIM(hart_id));
    if interrupt == 0 {
        None
    } else {
        Some(interrupt)
    }
}

/// Tell the PLIC we've served the IRQ
pub fn plic_complete(interrupt: u32) {
    let hart_id = unsafe {
        cpuid()
    } as usize;
    write(PLIC_SCLAIM(hart_id), interrupt);
}


fn write(addr: usize, val: u32) {
    unsafe {
        ptr::write(addr as *mut u32, val);
    }
}

fn read(addr: usize) -> u32 {
    unsafe {
        ptr::read(addr as *const u32)
    }
}

fn cpuid() -> u8 {
    let mut cpu_id;
    unsafe {
        asm!("mv {0}, tp", out(reg) cpu_id);
    }
    cpu_id
}