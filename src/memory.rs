use core::{mem, slice};

use super::{Cb64, Record};

#[derive(Debug)]
#[repr(u32)]
pub enum MemoryRangeKind {
    Ram = 1,
    Reserved = 2,
    Acpi = 3,
    Nvs = 4,
    Unusable = 5,
    VendorReserved = 6,
    Table = 16,
}

#[derive(Debug)]
#[repr(packed)]
pub struct MemoryRange {
    pub start: Cb64,
    pub size: Cb64,
    pub kind: MemoryRangeKind,
}

#[derive(Debug)]
#[repr(packed)]
pub struct Memory {
    pub record: Record,
}

impl Memory {
    pub fn ranges(&self) -> &[MemoryRange] {
        let address = (self as *const Memory as usize) + mem::size_of::<Record>();
        let size = (self.record.size as usize) - mem::size_of::<Record>();
        unsafe {
            slice::from_raw_parts(
                address as *const MemoryRange,
                size / mem::size_of::<MemoryRange>()
            )
        }
    }
}
