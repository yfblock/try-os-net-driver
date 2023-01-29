use alloc::{boxed::Box, vec::Vec};
use core::sync::atomic::{AtomicUsize, Ordering};
use nvme_driver::{DmaAllocator, IrqController, NvmeInterface};
use core::ptr::write_volatile;

use crate::mm::StepByOne;
use crate::sync::UPSafeCell;
use crate::mm::FrameTracker;
use crate::mm::VirtAddr;
use crate::mm::kernel_token;
use crate::mm::PageTable;
use crate::mm::frame_dealloc;
use crate::mm::PhysAddr;
use crate::mm::frame_alloc;
use crate::mm::PhysPageNum;

use super::BlockDevice;

use lazy_static::lazy_static;

lazy_static! {
    static ref QUEUE_FRAMES: UPSafeCell<Vec<FrameTracker>> = unsafe { UPSafeCell::new(Vec::new()) };
}

pub struct DmaAllocatorImpl;
impl DmaAllocator for DmaAllocatorImpl {
    fn dma_alloc(size: usize) -> usize{
        let mut ppn_base = PhysPageNum(0);
        for i in 0..(size / 0x1000) {
            let frame = frame_alloc().unwrap();
            if i == 0 {
                ppn_base = frame.ppn;
            }
            assert_eq!(frame.ppn.0, ppn_base.0 + i);
            QUEUE_FRAMES.exclusive_access().push(frame);
        }
        let pa: PhysAddr = ppn_base.into();
        pa.0
    }

    fn dma_dealloc(addr: usize, size: usize) -> usize{
        let addr = PhysAddr::from(addr);
        let mut ppn_base: PhysPageNum = addr.into();
        for _ in 0..(size / 0x1000) {
            frame_dealloc(ppn_base);
            ppn_base.step();
        }
        0
    }

    fn phys_to_virt(phys: usize) -> usize{
        phys
    }

    fn virt_to_phys(virt: usize) -> usize{
        PageTable::from_token(kernel_token())
            .translate_va(VirtAddr::from(virt))
            .unwrap()
            .0
    }
}

pub struct IrqControllerImpl;

impl IrqController for IrqControllerImpl {
    fn enable_irq(_irq: usize){

    }

    fn disable_irq(_irq: usize){

    }
}


// 虚拟IO设备
// pub struct VirtIOBlock(pub NvmeInterface::<DmaAllocatorImpl, IrqControllerImpl>);

impl BlockDevice for VirtIOBlock {
    fn read_block(&self, sector_offset: usize, buf: &mut [u8]) {
        assert_eq!(buf.len(), 512);
        // 读取文件
        self.0.exclusive_access().read_block(sector_offset, buf)
    }

    fn write_block(&self, sector_offset: usize, buf: &[u8]) {
        assert_eq!(buf.len(), 512);
        self.0.exclusive_access().write_block(sector_offset, buf)
    }
}

pub struct VirtIOBlock(UPSafeCell<NvmeInterface::<DmaAllocatorImpl, IrqControllerImpl>>);

impl VirtIOBlock {
    #[allow(unused)]
    pub fn new() -> Self {
        println!("start log nvme block device");
        config_pci();
        println!("log nvme block device");
        unsafe {
            Self(UPSafeCell::new(
                NvmeInterface::<DmaAllocatorImpl, IrqControllerImpl>::new(0x40000000)
            ))
        }
    }
}

// config pci
pub fn config_pci(){
    let ptr = 0x30008010 as *mut u32;
    unsafe { write_volatile(ptr, 0xffffffff); }
    let ptr = 0x30008010 as *mut u32;
    unsafe { write_volatile(ptr, 0x4); }
    let ptr = 0x30008010 as *mut u32;
    unsafe { write_volatile(ptr, 0x40000000); }
    let ptr = 0x30008004 as *mut u32;
    unsafe { write_volatile(ptr, 0x100006); }
    let ptr = 0x3000803c as *mut u32;
    unsafe { write_volatile(ptr, 0x21); }
}

// pub fn init() {
//     // 初始化 pci
//     config_pci();

//     unsafe {
//         // 创建存储设备
//         DEVICE.call_once(|| {
//             let device = Box::new(VirtIOBlock(
//                 NvmeInterface::<DmaAllocatorImpl, IrqControllerImpl>::new(0x40000000)
//             ));
//             Mutex::new(device)
//         });
//     }
// }