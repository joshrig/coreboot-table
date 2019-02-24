#[derive(Debug)]
#[repr(packed)]
pub struct Header {
    pub signature: [u8; 4],
    pub header_bytes: u32,
    pub header_checksum: u32,
    pub table_bytes: u32,
    pub table_checksum: u32,
    pub table_entries: u32,
}

impl Header {
    pub fn is_valid(&self) -> bool {
        //TODO: Check checksums
        &self.signature == b"LBIO"
    }
}
