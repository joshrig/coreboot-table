#![no_std]

use core::{mem, ptr};

pub use self::cb64::Cb64;
pub use self::cmos::{Cmos, CmosEntry, CmosEnum, CmosRecord};
pub use self::forward::Forward;
pub use self::framebuffer::Framebuffer;
pub use self::header::Header;
pub use self::mapper::{Mapper, PhysicalAddress, VirtualAddress};
pub use self::memory::{Memory, MemoryRange, MemoryRangeKind};
pub use self::record::{Record, RecordKind};

mod cb64;
mod cmos;
mod forward;
mod framebuffer;
mod header;
mod mapper;
mod memory;
mod record;

#[derive(Debug)]
pub enum Table<'a> {
    Cmos(&'a Cmos),
    Framebuffer(&'a Framebuffer),
    Memory(&'a Memory),
    Other(&'a Record),
}

pub fn tables<F: FnMut(Table) -> Result<(), &'static str>, M: Mapper>(callback: F, mapper: &mut M) -> Result<(), &'static str> {
    let mut env = Env {
        callback,
        mapper
    };
    env.tables()
}

struct Env<'m, F: FnMut(Table) -> Result<(), &'static str>, M: Mapper>  {
    callback: F,
    mapper: &'m mut M,
}

impl<'m, F: FnMut(Table) -> Result<(), &'static str>, M: Mapper> Env<'m, F, M> {
    fn forward(&mut self, forward: &Forward) -> Result<(), &'static str> {
        let page_size = self.mapper.page_size();

        let header_physical = PhysicalAddress(forward.forward as usize);
        let header_address = unsafe { self.mapper.map(header_physical, page_size)? };

        let header = unsafe { ptr::read((header_address.0) as *const Header) };

        unsafe { self.mapper.unmap(header_address)? };

        if header.is_valid() {
            self.header(header, PhysicalAddress(header_physical.0))
        } else {
            Err("Forward header invalid")
        }
    }

    fn header(&mut self, header: Header, header_physical: PhysicalAddress) -> Result<(), &'static str> {
        let mut result = Ok(());

        let table_physical = PhysicalAddress(header_physical.0 + header.header_bytes as usize);
        let table_size = header.table_bytes as usize;
        let table_address = unsafe { self.mapper.map(table_physical, table_size)? };
        let table_entries = header.table_entries as usize;
        {
            let mut i = 0;
            let mut entries = 0;
            while i + mem::size_of::<Record>() <= table_size && entries < table_entries {
                let record_address = table_address.0 + i;
                let record = unsafe { &*(record_address as *const Record) };

                result = match record.kind {
                    RecordKind::CmosOptionTable => {
                        (self.callback)(Table::Cmos(
                            unsafe { &*(record_address as *const Cmos) }
                        ))
                    },
                    RecordKind::Forward => {
                        let forward = unsafe { &*(record_address as *const Forward) };
                        self.forward(forward)
                    },
                    RecordKind::Framebuffer => (self.callback)(Table::Framebuffer(
                        unsafe { &*(record_address as *const Framebuffer) }
                    )),
                    RecordKind::Memory => (self.callback)(Table::Memory(
                        unsafe { &*(record_address as *const Memory) }
                    )),
                    _ => (self.callback)(Table::Other(record)),
                };

                if ! result.is_ok() {
                    break;
                }

                i += record.size as usize;
                entries += 1;
            }
        }

        unsafe { self.mapper.unmap(table_address)? };

        return result;
    }

    pub fn tables(&mut self) -> Result<(), &'static str> {
        let page_size = self.mapper.page_size();

        // First, we need to find the header somewhere in low memory
        let low_memory = 1024 * 1024;
        for header_page in 0..(low_memory / page_size) {
            let header_physical = PhysicalAddress(header_page * page_size);
            let header_address = unsafe { self.mapper.map(header_physical, page_size)? };

            {
                let mut i = 0;
                while i + mem::size_of::<Header>() <= page_size {
                    let header = unsafe { ptr::read((header_address.0 + i) as *const Header) };

                    if header.is_valid() {
                        unsafe { self.mapper.unmap(header_address)? };

                        return self.header(header, PhysicalAddress(header_physical.0 + i));
                    }

                    i += 4;
                }
            }

            unsafe { self.mapper.unmap(header_address)? };
        }

        // Header not found
        Err("Header not found")
    }
}
