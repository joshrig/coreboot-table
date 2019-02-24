use super::record::Record;

#[derive(Debug)]
#[repr(packed)]
pub struct Forward {
    pub record: Record,
    pub forward: u64,
}
