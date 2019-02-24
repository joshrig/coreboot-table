#![no_std]

use core::{mem, ptr, slice};

pub use self::cb64::Cb64;
pub use self::mapper::{Mapper, PhysicalAddress, VirtualAddress};

use self::framebuffer::Framebuffer;
use self::header::Header;
use self::memory::Memory;
use self::record::{Record, RecordKind};

mod cb64;
mod framebuffer;
mod header;
mod mapper;
mod memory;
mod record;

#[derive(Debug)]
pub enum Table<'a> {
    Framebuffer(&'a Framebuffer),
    Memory(&'a Memory),
    Other(&'a Record),
}

pub fn tables<F, M>(mut callback: F, mapper: &mut M) -> Result<(), &'static str>
    where F: FnMut(Table) -> Result<(), &'static str>, M: Mapper
{
    let page_size = mapper.page_size();

    // First, we need to find the header somewhere in low memory
    let low_memory = 1024 * 1024;
    for header_page in 0..(low_memory / page_size) {
        let header_physical = PhysicalAddress(header_page * page_size);
        let header_address = unsafe { mapper.map(header_physical, page_size)? };

        {
            let mut i = 0;
            while i + mem::size_of::<Header>() <= page_size {
                let header = unsafe { ptr::read((header_address.0 + i) as *const Header) };

                if header.is_valid() {
                    unsafe { mapper.unmap(header_address)? };

                    let table_physical = PhysicalAddress(header_physical.0 + i + header.header_bytes as usize);
                    let table_size = header.table_bytes as usize;
                    let table_address = unsafe { mapper.map(table_physical, table_size)? };
                    let table_entries = header.table_entries as usize;

                    {
                        let mut j = 0;
                        let mut entries = 0;
                        while j + mem::size_of::<Record>() <= table_size && entries < table_entries {
                            let record_address = table_address.0 + j;
                            let record = unsafe { &*(record_address as *const Record) };

                            callback(match record.kind {
                                RecordKind::Framebuffer => Table::Framebuffer(
                                    unsafe { &*(record_address as *const Framebuffer) }
                                ),
                                RecordKind::Memory => Table::Memory(
                                    unsafe { &*(record_address as *const Memory) }
                                ),
                                _ => Table::Other(record),
                            });

                            j += record.size as usize;
                            entries += 1;
                        }
                    }

                    unsafe { mapper.unmap(table_address)? };

                    return Ok(());
                }

                i += 4;
            }
        }

        unsafe { mapper.unmap(header_address)? };
    }

    // Header not found
    Err("Header not found")
}
