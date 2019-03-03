use core::{mem, slice};

use super::{Record, RecordKind};

unsafe fn text<'a>(ptr: *const u8, max: usize) -> &'a [u8] {
    let mut i = 0;
    while i < max {
        if *ptr.add(i) == 0 {
            break;
        }
        i += 1;
    }
    slice::from_raw_parts(
        ptr,
        i
    )
}

#[derive(Debug)]
#[repr(packed)]
pub struct CmosEntry {
    pub record: Record,
    pub bit: u32,
    pub length: u32,
    pub config: u32,
    pub config_id: u32,
    // name: [u8; 32]
}

impl CmosEntry {
    pub fn name(&self) -> &[u8] {
        unsafe {
            text(
                (self as *const CmosEntry as usize + mem::size_of::<CmosEntry>()) as *const u8,
                self.record.size as usize - mem::size_of::<CmosEntry>()
            )
        }
    }
}

#[derive(Debug)]
#[repr(packed)]
pub struct CmosEnum {
    pub record: Record,
    pub config_id: u32,
    pub value: u32,
    // text: [u8; 32]
}

impl CmosEnum {
    pub fn text(&self) -> &[u8] {
        unsafe {
            text(
                (self as *const CmosEnum as usize + mem::size_of::<CmosEnum>()) as *const u8,
                self.record.size as usize - mem::size_of::<CmosEnum>()
            )
        }
    }
}

#[derive(Debug)]
pub enum CmosRecord<'a> {
    Entry(&'a CmosEntry),
    Enum(&'a CmosEnum),
    Other(&'a Record),
}

#[derive(Debug)]
#[repr(packed)]
pub struct Cmos {
    pub record: Record,
    pub header_length: u32,
}

impl Cmos {
    pub fn records(&self) -> CmosRecords {
        CmosRecords {
            table: self,
            i: self.header_length as usize,
        }
    }
}

pub struct CmosRecords<'a> {
    table: &'a Cmos,
    i: usize,
}

impl<'a> Iterator for CmosRecords<'a> {
    type Item = CmosRecord<'a>;
    fn next(&mut self) -> Option<CmosRecord<'a>> {
        if self.i + mem::size_of::<Record>() <= self.table.record.size as usize {
            let address = (self.table as *const Cmos as usize) + self.i;
            let record = unsafe { &*(address as *const Record) };
            self.i += record.size as usize;
            Some(match record.kind {
                RecordKind::Option => CmosRecord::Entry(unsafe {
                    &*(address as *const CmosEntry)
                }),
                RecordKind::OptionEnum => CmosRecord::Enum(unsafe {
                    &*(address as *const CmosEnum)
                }),
                _ => CmosRecord::Other(record),
            })
        } else {
            None
        }
    }
}
