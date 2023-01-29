use alloc::{string::String, vec};
use alloc::sync::Arc;
use alloc::vec::Vec;
use fatfs::{Dir, NullTimeProvider, LossyOemCpConverter, File, Seek, SeekFrom, Write, Read};
use spin::Mutex;

use crate::efs::DiskCursor;
/// Virtual filesystem layer over easy-fs
pub enum Inode {
    /// File 
    File(Arc<Mutex<File<'static, DiskCursor, NullTimeProvider, LossyOemCpConverter>>>),
    /// Dir
    Dir(Arc<Mutex<Dir<'static, DiskCursor, NullTimeProvider, LossyOemCpConverter>>>)
}

unsafe impl Sync for Inode {}
unsafe impl Send for Inode {}

impl Inode {
    /// Find inode under current inode by name
    pub fn find(&self, name: &str) -> Option<Arc<Inode>> {
        match self {
            Inode::File(_) => None,
            Inode::Dir(f) => {
                // if let Ok(dir) = f.lock().open_dir(name) {
                //     Some(Arc::new(Inode::Dir(Arc::new(Mutex::new(dir)))))
                // } else 
                if let Ok(file) = f.lock().open_file(name) {
                    Some(Arc::new(Inode::File(Arc::new(Mutex::new(file)))))
                } else {
                    None
                }
            },
        }
    }
    /// Create inode under current inode by name
    /// 
    /// create file node or dir node?
    pub fn create(&self, name: &str) -> Option<Arc<Inode>> {
        match self {
            Inode::File(_) => None,
            Inode::Dir(dir_node) => {
                if let Ok(file) = dir_node.lock().create_file(name) {
                    Some(Arc::new(Inode::File(Arc::new(Mutex::new(file)))))
                } else {
                    None
                }
            },
        }
        // release efs lock automatically by compiler
    }
    /// List inodes under current inode
    pub fn ls(&self) -> Vec<String> {
        match self {
            Inode::File(_) => vec![],
            Inode::Dir(dir_node) => {
                let mut arr = vec![];
                for i in dir_node.lock().iter() {
                    if let Ok(i) = i {
                        arr.push(i.file_name());
                    }
                }
                arr
            },
        }
    }
    /// Read data from current inode
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        match self {
            Inode::File(f) =>  {
                let mut f = f.lock();
                f.seek(SeekFrom::Start(offset as u64));
                // f.read_exact(buf);
                // buf.len()
                f.read(buf).map_or(0, |f| f)
            },
            Inode::Dir(_) => 0
        }
    }
    /// Write data to current inode
    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        match self {
            Inode::File(f) =>  {
                let mut f = f.lock();
                f.seek(SeekFrom::Start(offset as u64));
                f.write_all(buf);
                buf.len()
            },
            Inode::Dir(_) => 0
        }
    }
    /// Clear the data in current inode
    pub fn clear(&self) {
        match self {
            Inode::File(f) =>  {
                f.lock().seek(SeekFrom::Start(0));
                f.lock().truncate();
            },
            Inode::Dir(_) => {}
        }
    }
}

