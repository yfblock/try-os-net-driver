// remove std lib
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(default_alloc_error_handler)]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(asm_sym)]
#![feature(stdsimd)]
#![allow(unaligned_references)]

extern crate alloc;

#[macro_use]
mod console;
mod panic;
mod sbi;
mod fs;
mod memory;
mod mutex;
mod test_async;
mod task;
mod interrupt;
mod block;
mod uart;
// mod loopback;
mod utils;
mod plic;

use core::{arch::{asm, riscv64::wfi}, ptr::NonNull};

use fdt::{Fdt, node::FdtNode, standard_nodes::Compatible};
use riscv::register::{medeleg, mideleg, mhartid};
use virtio_drivers::{VirtIOHeader, MmioTransport, Transport};

/// 汇编入口函数
/// 
/// 分配栈 并调到rust入口函数
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> ! {
    const STACK_SIZE: usize = 4096;

    #[link_section = ".bss.stack"]
    static mut STACK: [u8; STACK_SIZE] = [0u8; STACK_SIZE];

    core::arch::asm!(
        "   la  sp, {stack} + {stack_size}
            mv  tp, a0
            j   rust_main
        ",
        stack_size = const STACK_SIZE,
        stack      =   sym STACK,
        options(noreturn),
    )
}

/// rust 入口函数
/// 
/// 进行操作系统的初始化，
#[no_mangle]
pub extern "C" fn rust_main(hart_id: usize, _device_tree_addr: usize) -> ! {
    // 让其他核心进入等待
    if hart_id != 0 {
        support_hart_resume(hart_id, 0);
    }

    init_dt(_device_tree_addr);
    
    memory::init();
    block::init();
    interrupt::init();
    uart::init();

    plic::init();
//    fs::ls_dir("var");
    // loopback::net_main(); 

    loop {
        unsafe {
            wfi();
        }
    }
    // test_async::init();

    // 调用rust api关机
    panic!("正常关机")
}


/// 辅助核心进入的函数
/// 
/// 目前让除 0 核之外的其他内核进入该函数进行等待
#[allow(unused)]
extern "C" fn support_hart_resume(hart_id: usize, _param: usize) {
    loop {
        // 使用wfi 省电
        unsafe { asm!("wfi") }
    }
}


fn init_dt(dtb: usize) {
    println!("device tree @ {:#x}", dtb);
    // Safe because the pointer is a valid pointer to unaliased memory.
    let fdt = unsafe { Fdt::from_ptr(dtb as *const u8).unwrap() };
    walk_dt(fdt);
}

fn walk_dt(fdt: Fdt) {
    for node in fdt.all_nodes() {
        if let Some(compatible) = node.compatible() {
            if compatible.all().any(|s| s == "virtio,mmio") {
                virtio_probe(node);
            }
        }
    }
}

fn virtio_probe(node: FdtNode) {
    if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
        let paddr = reg.starting_address as usize;
        let size = reg.size.unwrap();
        let vaddr = paddr;
        println!("walk dt addr={:#x}, size={:#x}", paddr, size);
        println!(
            "Device tree node {}: {:?}",
            node.name,
            node.compatible().map(Compatible::first),
        );
        let header = NonNull::new(vaddr as *mut VirtIOHeader).unwrap();
        match unsafe { MmioTransport::new(header) } {
            Err(e) => println!("Error creating VirtIO MMIO transport: {}", e),
            Ok(transport) => {
                println!(
                    "[mmio] Detected virtio MMIO device with vendor id {:#X}, device type {:?}, version {:?}",
                    transport.vendor_id(),
                    transport.device_type(),
                    transport.version(),
                );
                // virtio_device(transport);
            }
        }
    }
}
