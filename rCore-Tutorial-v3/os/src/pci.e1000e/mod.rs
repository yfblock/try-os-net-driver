mod pci_impl;
pub mod e1000;
pub mod plic;

use core::sync::atomic::{fence, Ordering};

use alloc::{format, vec::Vec};
use device_tree::util::SliceRead;
use device_tree::{DeviceTree, Node};
use isomorphic_drivers::provider::Provider;
use pci::{PCIDevice, Location, scan_bus, BAR};
use pci_impl::*;

use crate::config::PAGE_SIZE;
use crate::mm::{frame_alloc, frame_dealloc, PhysPageNum, PhysAddr, StepByOne, FrameTracker, frame_alloc_trackers};
use crate::sync::UPSafeCell;

use lazy_static::lazy_static;

// use log to output
// #[no_mangle]
// extern "C" fn main(_hartid: usize, device_tree_paddr: usize) {
//     opensbi_rt::println!("\nHi !\nTEST START");
//     log::set_max_level(LevelFilter::Debug);
//     info!("log initialized");
//     init_dt(device_tree_paddr);
//     info!("TEST END");
// }

/// Enable the pci device and its interrupt
/// Return assigned MSI interrupt number when applicable
unsafe fn enable(loc: Location) -> Option<usize> {

    let ops = &PortOpsImpl;
    let am = PCI_ACCESS;

    // 23 and lower are used
    static mut MSI_IRQ: u32 = 23;

    let orig = am.read16(ops, loc, PCI_COMMAND);
    // IO Space | MEM Space | Bus Mastering | Special Cycles | PCI Interrupt Disable
    am.write32(ops, loc, PCI_COMMAND, (orig | 0x40f) as u32);

    // find MSI cap
    let mut msi_found = false;
    let mut cap_ptr = am.read8(ops, loc, PCI_CAP_PTR) as u16;
    let mut assigned_irq = None;
    while cap_ptr > 0 {
        let cap_id = am.read8(ops, loc, cap_ptr);
        if cap_id == PCI_CAP_ID_MSI {
            let orig_ctrl = am.read32(ops, loc, cap_ptr + PCI_MSI_CTRL_CAP);
            // The manual Volume 3 Chapter 10.11 Message Signalled Interrupts
            // 0 is (usually) the apic id of the bsp.
            am.write32(ops, loc, cap_ptr + PCI_MSI_ADDR, 0xfee00000 | (0 << 12));
            MSI_IRQ += 1;
            let irq = MSI_IRQ;
            assigned_irq = Some(irq as usize);
            // we offset all our irq numbers by 32
            if (orig_ctrl >> 16) & (1 << 7) != 0 {
                // 64bit
                am.write32(ops, loc, cap_ptr + PCI_MSI_DATA_64, irq + 32);
            } else {
                // 32bit
                am.write32(ops, loc, cap_ptr + PCI_MSI_DATA_32, irq + 32);
            }

            // enable MSI interrupt, assuming 64bit for now
            am.write32(ops, loc, cap_ptr + PCI_MSI_CTRL_CAP, orig_ctrl | 0x10000);
            println!(
                "MSI control {:#b}, enabling MSI interrupt {}",
                orig_ctrl >> 16,
                irq
            );
            msi_found = true;
        }
        println!("PCI device has cap id {} at {:#X}", cap_id, cap_ptr);
        cap_ptr = am.read8(ops, loc, cap_ptr + 1) as u16;
    }

    if !msi_found {
        // Use PCI legacy interrupt instead
        // IO Space | MEM Space | Bus Mastering | Special Cycles
        am.write32(ops, loc, PCI_COMMAND, (orig | 0xf) as u32);
        println!("MSI not found, using PCI interrupt");
    }

    println!("pci device enable done");

    assigned_irq
}

pub fn init_driver(dev: &PCIDevice) {
    let name = format!("enp{}s{}f{}", dev.loc.bus, dev.loc.device, dev.loc.function);
    match (dev.id.vendor_id, dev.id.device_id) {
        (0x8086, 0x100e) | (0x8086, 0x100f) | (0x8086, 0x10d3) => {
            // 0x100e
            // 82540EM Gigabit Ethernet Controller
            // 0x100f
            // 82545EM Gigabit Ethernet Controller (Copper)
            // 0x10d3
            // 82574L Gigabit Network Connection
            if let Some(BAR::Memory(addr, len, _, _)) = dev.bars[0] {
                let addr = if addr == 0 { E1000_BASE as u64 } else { addr };
                let irq = unsafe { enable(dev.loc) };
                e1000::init(name, irq.unwrap(), addr as usize, len as usize, 1);
                return;
            }
        }
        _ => {}
    }
}


pub fn pci_init() {
    let pci_iter = unsafe { scan_bus(&PortOpsImpl, PCI_ACCESS) };
    for dev in pci_iter {
        println!(
            "pci: {:02x}:{:02x}.{} {:#x} {:#x} ({} {}) irq: {}:{:?}",
            dev.loc.bus,
            dev.loc.device,
            dev.loc.function,
            dev.id.vendor_id,
            dev.id.device_id,
            dev.id.class,
            dev.id.subclass,
            dev.pic_interrupt_line,
            dev.interrupt_pin,
        );
        init_driver(&dev);
    }
}

pub fn find_device(vendor: u16, product: u16) -> Option<Location> {
    let pci_iter = unsafe { scan_bus(&PortOpsImpl, PCI_ACCESS) };
    for dev in pci_iter {
        if dev.id.vendor_id == vendor && dev.id.device_id == product {
            return Some(dev.loc);
        }
    }
    None
}


pub fn init() {

    const BAR_LEN: usize = 40;
    println!("{:-^1$}", "PCI INIT", BAR_LEN);
    pci_init();
    plic::init();
    println!("{:-^1$}", "PCI INIT SUCCESS", BAR_LEN);
}

lazy_static! {
    static ref QUEUE_FRAMES: UPSafeCell<Vec<FrameTracker>> = unsafe { UPSafeCell::new(Vec::new()) };
}

pub struct ProviderImpl;

impl Provider for ProviderImpl {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_dma(size: usize) -> (usize, usize) {
        let trakcers = frame_alloc_trackers(size / PAGE_SIZE);
        let ppn_base = trakcers.as_ref().unwrap().first().unwrap().ppn;
        QUEUE_FRAMES.exclusive_access().append(&mut trakcers.unwrap());
        let pa: PhysAddr = ppn_base.into();
        (pa.0, pa.0)
    }

    fn dealloc_dma(vaddr: usize, size: usize) {
        let addr = PhysAddr::from(vaddr);
        let mut ppn_base: PhysPageNum = addr.into();
        for _ in 0..(size / PAGE_SIZE) {
            frame_dealloc(ppn_base);
            ppn_base.step();
        }
    }
}