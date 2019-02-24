extern crate coreboot_table;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::slice;

use coreboot_table::{
    Mapper, PhysicalAddress, VirtualAddress,
    Table,
};

struct FileMapper {
    file: File,
}

impl FileMapper {
    fn new(path: &str) -> io::Result<FileMapper> {
        Ok(FileMapper {
            file: File::open(path)?
        })
    }
}

impl Mapper for FileMapper {
    unsafe fn map_aligned(&mut self, address: PhysicalAddress, size: usize) -> Result<VirtualAddress, &'static str> {
        extern "C" {
            fn memalign(alignment: usize, size: usize) -> usize;
        }

        let page_size = self.page_size();
        let ptr = memalign(page_size, size);
        if ptr == 0 {
            return Err("Failed to allocate memory");
        }

        let data = slice::from_raw_parts_mut(
            ptr as *mut u8,
            size
        );

        self.file.seek(SeekFrom::Start(address.0 as u64)).map_err(|_| "Failed to seek file")?;
        self.file.read(data).map_err(|_| "Failed to read file")?;

        Ok(VirtualAddress(ptr))
    }

    unsafe fn unmap_aligned(&mut self, address: VirtualAddress) -> Result<(), &'static str> {
        extern "C" {
            fn free(ptr: usize);
        }

        free(address.0);

        Ok(())
    }

    fn page_size(&self) -> usize {
        4096
    }
}

fn main() {
    let mut mapper = FileMapper::new("res/example.bin").unwrap();

    coreboot_table::tables(|table| {
        match table {
            Table::Framebuffer(framebuffer) => println!("{:?}", framebuffer),
            Table::Memory(memory) => println!("{:?}", memory.ranges()),
            Table::Other(other) => println!("{:?}", other),
        }
        Ok(())
    }, &mut mapper).unwrap();
}
