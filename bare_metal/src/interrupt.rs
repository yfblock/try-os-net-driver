use core::arch::asm;

use riscv::register::{sstatus, stvec, utvec::TrapMode, self, scause};

use crate::sbi;

#[naked]
unsafe fn trap_entry() {
    asm!("
    .align 4
    call trap_handler
    ", options(noreturn));
}

#[no_mangle]
fn trap_handler() {

    println!("trap {:?}", scause::read().cause());

    panic!("trap")
}

pub fn init() {
    unsafe {
        stvec::write(trap_entry as usize, TrapMode::Direct);
        // register::sie::set_stimer();
        register::sie::set_sext();
        // register::sie::set_ssoft();
        // register::sie::set_uext();
        sstatus::set_sie();
        
        // sbi::set_timer(register::time::read() + 100);

        // asm!("ebreak");
    }
}