use crate::{BLOCK_SZ, BlockDevice, Inode};
use alloc::{sync::Arc, vec};
use spin::{Mutex, Once};
use fatfs::{NullTimeProvider, LossyOemCpConverter};

static FILE_SYSTEM: Once<FileSystem> = Once::new();
// static FILE_SYSTEM: Lazy<FileSystem<DiskCursor, NullTimeProvider, LossyOemCpConverter>> = Lazy::new(); 

pub struct FileSystem(fatfs::FileSystem<DiskCursor, NullTimeProvider, LossyOemCpConverter>);

unsafe impl Sync for FileSystem {}

unsafe impl Send for FileSystem {}

///An easy file system on block
pub struct EasyFileSystem {}

/// An easy fs over a block device
impl EasyFileSystem {
    /// A data block of block size
    // pub fn create(
    //     block_device: Arc<dyn BlockDevice>,
    //     total_blocks: u32,
    //     inode_bitmap_blocks: u32,
    // ) -> Arc<Mutex<Self>> {
    //     todo!()
    // }
    /// Open a block device as a filesystem
    pub fn open(block_device: Arc<dyn BlockDevice>) -> Arc<Mutex<Self>> {
        let c = DiskCursor {
            sector: 0,
            offset: 0,
            block_device: block_device
        };
        FILE_SYSTEM.call_once(|| FileSystem(
            fatfs::FileSystem::new(c, fatfs::FsOptions::new()).expect("open fs fai")
        ));
        Arc::new(Mutex::new(Self {}))
    }
    /// Get the root inode of the filesystem
    pub fn root_inode(_efs: &Arc<Mutex<Self>>) -> Inode {
        Inode::Dir(Arc::new(Mutex::new(FILE_SYSTEM.get().unwrap().0.root_dir())))
    }
}


#[derive(Debug)]
pub enum DiskCursorIoError {
    UnexpectedEof,
    WriteZero,
}
impl fatfs::IoError for DiskCursorIoError {
    fn is_interrupted(&self) -> bool {
        false
    }

    fn new_unexpected_eof_error() -> Self {
        Self::UnexpectedEof
    }

    fn new_write_zero_error() -> Self {
        Self::WriteZero
    }
}

pub struct DiskCursor {
    sector: u64,
    offset: usize,
    block_device: Arc<dyn BlockDevice>
}

unsafe impl Sync for DiskCursor {
    
}

unsafe impl Send for DiskCursor {
    
}

impl DiskCursor {
    fn get_position(&self) -> usize {
        (self.sector * 0x200) as usize + self.offset
    }

    fn set_position(&mut self, position: usize) {
        self.sector = (position / 0x200) as u64;
        self.offset = position % 0x200;
    }

    fn move_cursor(&mut self, amount: usize) {
        self.set_position(self.get_position() + amount)
    }
}

impl fatfs::IoBase for DiskCursor {
    type Error = DiskCursorIoError;
}

impl fatfs::Read for DiskCursor {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, DiskCursorIoError> {
        // 由于读取扇区内容还需要考虑跨 cluster，因此 read 函数只读取一个扇区
        // 防止读取较多数据时超出限制
        // 读取所有的数据的功能交给 read_exact 来实现

        // 获取硬盘设备读取器（驱动？）
        let block_device = &self.block_device;

        // 如果 start 不是 0 或者 len 不是 512
        let read_size = if self.offset != 0 || buf.len() < 512 {
            // let mut data = [0u8; 512];
            let mut data = vec![0u8; 512];
            block_device.read_block(self.sector as usize, &mut data);

            let start = self.offset;
            let end = (self.offset + buf.len()).min(512);
            
            buf.copy_from_slice(&data[start..end]);
            end-start
        } else {
            block_device.read_block(self.sector as usize, &mut buf[0..512]);
            512
        };

        self.move_cursor(read_size);
        Ok(read_size)
    }
}

impl fatfs::Write for DiskCursor {
    fn write(&mut self, buf: &[u8]) -> Result<usize, DiskCursorIoError> {
        // 由于写入扇区还需要考虑申请 cluster，因此 write 函数只写入一个扇区
        // 防止写入较多数据时超出限制
        // 写入所有的数据的功能交给 write_all 来实现

        // 获取硬盘设备写入器（驱动？）
        let block_device = &self.block_device;

        // 如果 start 不是 0 或者 len 不是 512
        let write_size = if self.offset != 0 || buf.len() < 512 {
            let mut data = vec![0u8; 512];
            block_device.read_block(self.sector as usize, &mut data);

            let start = self.offset;
            let end = (self.offset + buf.len()).min(512);
            
            data[start..end].clone_from_slice(&buf);
            block_device.write_block(self.sector as usize, &mut data);

            end-start
        } else {
            block_device.write_block(self.sector as usize, &buf[0..512]);
            512
        };

        self.move_cursor(write_size);
        Ok(write_size)
    }

    fn flush(&mut self) -> Result<(), DiskCursorIoError> {
        Ok(())
    }
}

impl fatfs::Seek for DiskCursor {
    fn seek(&mut self, pos: fatfs::SeekFrom) -> Result<u64, DiskCursorIoError> {
        match pos {
            fatfs::SeekFrom::Start(i) => {
                self.set_position(i as usize);
                Ok(i)
            }
            fatfs::SeekFrom::End(i) => {
                todo!("Seek from end")
            }
            fatfs::SeekFrom::Current(i) => {
                let new_pos = (self.get_position() as i64) + i;
                self.set_position(new_pos as usize);
                Ok(new_pos as u64)
            }
        }
    }
}
